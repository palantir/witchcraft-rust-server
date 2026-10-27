// Copyright 2026 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//! The SIGPROF handler and the buffer it writes samples to.
//!
//! This is the only part of the profiler that runs in signal context. The handler does no unwinding - it captures
//! the registers needed to start an unwind along with a copy of the top of the thread's stack into a preallocated
//! slot, which the profiler thread later unwinds.
use crate::unwind::Regs;
use conjure_error::Error;
use memmap2::MmapMut;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::{hint, io, mem, ptr, thread};

/// The number of bytes of stack copied for each sample.
///
/// This is a bit larger than the maximum supported by `perf record --call-graph=dwarf`.
pub const STACK_SIZE: usize = 64 * 1024;

/// The number of samples which can be buffered before the profiler thread drains them.
const SLOTS: usize = 256;

/// The granularity of the stack copy. `process_vm_readv` only performs partial reads at the granularity of its
/// iovecs, so the remote side is split at boundaries no larger than the smallest page size.
const CHUNK_SIZE: usize = 4096;
const MAX_CHUNKS: usize = STACK_SIZE / CHUNK_SIZE + 1;

const FREE: u8 = 0;
const WRITING: u8 = 1;
const READY: u8 = 2;

static BUFFER: AtomicPtr<SampleBuffer> = AtomicPtr::new(ptr::null_mut());
static IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
static INSTALLED: AtomicBool = AtomicBool::new(false);

/// A sample captured by the signal handler.
pub struct Sample {
    pub tid: i32,
    pub regs: Regs,
    stack_len: usize,
    stack: [u8; STACK_SIZE],
}

impl Sample {
    /// The bytes of the stack starting at `regs.sp`.
    pub fn stack(&self) -> &[u8] {
        &self.stack[..self.stack_len]
    }
}

struct Slot {
    state: AtomicU8,
    sample: UnsafeCell<Sample>,
}

struct SampleBuffer {
    pid: libc::pid_t,
    slots: *const Slot,
    cursor: AtomicUsize,
    dropped: AtomicU64,
}

impl SampleBuffer {
    fn slots(&self) -> &[Slot] {
        // SAFETY: slots points to SLOTS slots for the lifetime of the buffer.
        unsafe { std::slice::from_raw_parts(self.slots, SLOTS) }
    }
}

/// Installs the SIGPROF handler if it hasn't been already.
///
/// The handler stays installed for the lifetime of the process and does nothing when no recording is active, so
/// signals from timers that fire while a recording is being torn down are harmless.
pub fn install() -> Result<(), Error> {
    if INSTALLED.load(Ordering::Acquire) {
        return Ok(());
    }

    // SAFETY: sigaction is zero-initializable, and we only replace the default or ignore dispositions.
    unsafe {
        let mut old = mem::zeroed::<libc::sigaction>();
        if libc::sigaction(libc::SIGPROF, ptr::null(), &mut old) != 0 {
            return Err(Error::internal_safe(io::Error::last_os_error()));
        }
        if old.sa_sigaction != libc::SIG_DFL && old.sa_sigaction != libc::SIG_IGN {
            return Err(Error::internal_safe(
                "a SIGPROF handler is already installed",
            ));
        }

        let mut new = mem::zeroed::<libc::sigaction>();
        new.sa_sigaction = handler
            as extern "C" fn(libc::c_int, *mut libc::siginfo_t, *mut libc::c_void)
            as libc::sighandler_t;
        new.sa_flags = libc::SA_SIGINFO | libc::SA_RESTART | libc::SA_ONSTACK;
        libc::sigemptyset(&mut new.sa_mask);
        if libc::sigaction(libc::SIGPROF, &new, ptr::null_mut()) != 0 {
            return Err(Error::internal_safe(io::Error::last_os_error()));
        }
    }

    INSTALLED.store(true, Ordering::Release);
    Ok(())
}

/// Checks that `process_vm_readv` can be used to copy stacks.
///
/// A seccomp filter could forbid it.
pub fn check_process_vm_readv() -> Result<(), Error> {
    let src = [1u8; 8];
    let mut dst = [0u8; 8];
    let local = libc::iovec {
        iov_base: dst.as_mut_ptr().cast(),
        iov_len: dst.len(),
    };
    let remote = libc::iovec {
        iov_base: src.as_ptr() as *mut _,
        iov_len: src.len(),
    };
    // SAFETY: both iovecs point to valid memory of the specified lengths.
    let nread = unsafe { libc::process_vm_readv(libc::getpid(), &local, 1, &remote, 1, 0) };
    if nread < 0 {
        return Err(Error::internal_safe(io::Error::last_os_error()));
    }
    if dst != src {
        return Err(Error::internal_safe("process_vm_readv returned bad data"));
    }

    Ok(())
}

/// Owns the sample buffer for the duration of a recording.
///
/// Only one may exist at a time.
pub struct Buffer {
    buffer: *mut SampleBuffer,
    _slots: MmapMut,
}

impl Buffer {
    pub fn new() -> Result<Self, Error> {
        // The anonymous mapping is lazily zeroed, which is a valid FREE slot.
        let mut slots =
            MmapMut::map_anon(SLOTS * mem::size_of::<Slot>()).map_err(Error::internal_safe)?;
        let buffer = Box::into_raw(Box::new(SampleBuffer {
            pid: std::process::id() as libc::pid_t,
            slots: slots.as_mut_ptr().cast(),
            cursor: AtomicUsize::new(0),
            dropped: AtomicU64::new(0),
        }));

        if BUFFER
            .compare_exchange(ptr::null_mut(), buffer, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            // SAFETY: the buffer was never published.
            drop(unsafe { Box::from_raw(buffer) });
            return Err(Error::internal_safe("a sample buffer is already active"));
        }

        Ok(Buffer {
            buffer,
            _slots: slots,
        })
    }

    fn get(&self) -> &SampleBuffer {
        // SAFETY: the buffer is live until we drop it.
        unsafe { &*self.buffer }
    }

    /// Passes each sample written by the signal handler to the callback and frees its slot.
    pub fn drain(&self, mut f: impl FnMut(&Sample)) {
        for slot in self.get().slots() {
            if slot.state.load(Ordering::Acquire) != READY {
                continue;
            }

            // SAFETY: the READY state gives us exclusive access until we mark it FREE.
            f(unsafe { &*slot.sample.get() });
            slot.state.store(FREE, Ordering::Release);
        }
    }

    /// Returns the number of samples dropped because the buffer was full.
    pub fn dropped(&self) -> u64 {
        self.get().dropped.load(Ordering::Relaxed)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        BUFFER.store(ptr::null_mut(), Ordering::SeqCst);
        // Handlers increment IN_FLIGHT before loading BUFFER, so once this is zero none can be using the buffer.
        while IN_FLIGHT.load(Ordering::SeqCst) != 0 {
            hint::spin_loop();
            thread::yield_now();
        }

        // SAFETY: the buffer was created by Box::into_raw and is no longer reachable from the handler.
        drop(unsafe { Box::from_raw(self.buffer) });
    }
}

// Everything below runs in signal context, and must be async-signal-safe: no allocation, locks, or panics.

extern "C" fn handler(_: libc::c_int, info: *mut libc::siginfo_t, ucontext: *mut libc::c_void) {
    // SAFETY: errno is thread local and always valid.
    let errno = unsafe { *libc::__errno_location() };
    IN_FLIGHT.fetch_add(1, Ordering::SeqCst);

    let buffer = BUFFER.load(Ordering::SeqCst);
    // SAFETY: the kernel passes valid info and ucontext pointers to SA_SIGINFO handlers, and the buffer is live while
    // IN_FLIGHT is nonzero.
    unsafe {
        // Ignore anything other than our timers, like a SIGPROF sent with kill.
        if !buffer.is_null() && (*info).si_code == libc::SI_TIMER {
            sample(&*buffer, &*ucontext.cast::<libc::ucontext_t>());
        }
    }

    IN_FLIGHT.fetch_sub(1, Ordering::SeqCst);
    // SAFETY: errno is thread local and always valid.
    unsafe { *libc::__errno_location() = errno };
}

fn sample(buffer: &SampleBuffer, ucontext: &libc::ucontext_t) {
    let Some(slot) = claim(buffer) else {
        buffer.dropped.fetch_add(1, Ordering::Relaxed);
        return;
    };

    // SAFETY: the WRITING state gives us exclusive access to the sample.
    let sample = unsafe { &mut *slot.sample.get() };
    // SAFETY: gettid is always safe to call.
    sample.tid = unsafe { libc::gettid() };
    sample.regs = regs(ucontext);
    sample.stack_len = copy_stack(buffer.pid, sample.regs.sp, &mut sample.stack);

    slot.state.store(READY, Ordering::Release);
}

fn claim(buffer: &SampleBuffer) -> Option<&Slot> {
    let slots = buffer.slots();
    let start = buffer.cursor.fetch_add(1, Ordering::Relaxed);
    (0..SLOTS)
        .map(|i| &slots[start.wrapping_add(i) % SLOTS])
        .find(|slot| {
            slot.state
                .compare_exchange(FREE, WRITING, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
        })
}

#[cfg(target_arch = "x86_64")]
fn regs(ucontext: &libc::ucontext_t) -> Regs {
    let gregs = &ucontext.uc_mcontext.gregs;
    Regs {
        pc: gregs[libc::REG_RIP as usize] as u64,
        sp: gregs[libc::REG_RSP as usize] as u64,
        fp: gregs[libc::REG_RBP as usize] as u64,
    }
}

#[cfg(target_arch = "aarch64")]
fn regs(ucontext: &libc::ucontext_t) -> Regs {
    let mcontext = &ucontext.uc_mcontext;
    Regs {
        pc: mcontext.pc,
        sp: mcontext.sp,
        fp: mcontext.regs[29],
        lr: mcontext.regs[30],
    }
}

/// Copies as much of the stack starting at `sp` into `buf` as is mapped, returning the number of bytes copied.
///
/// `process_vm_readv` stops at the first unreadable chunk rather than faulting, so this can't read past the end of the
/// thread's stack.
fn copy_stack(pid: libc::pid_t, sp: u64, buf: &mut [u8; STACK_SIZE]) -> usize {
    let local = libc::iovec {
        iov_base: buf.as_mut_ptr().cast(),
        iov_len: buf.len(),
    };

    let mut remote = [libc::iovec {
        iov_base: ptr::null_mut(),
        iov_len: 0,
    }; MAX_CHUNKS];
    let mut nchunks = 0;
    let mut addr = sp as usize;
    let end = addr.saturating_add(STACK_SIZE);
    while addr < end {
        let chunk_end = usize::min((addr / CHUNK_SIZE + 1).saturating_mul(CHUNK_SIZE), end);
        // Avoid a panicking index, though there are never more than MAX_CHUNKS.
        let Some(iov) = remote.get_mut(nchunks) else {
            break;
        };
        *iov = libc::iovec {
            iov_base: addr as *mut _,
            iov_len: chunk_end - addr,
        };
        nchunks += 1;
        addr = chunk_end;
    }

    // SAFETY: the local iovec points to the buffer, and process_vm_readv validates the remote iovecs.
    let nread = unsafe {
        libc::process_vm_readv(pid, &local, 1, remote.as_ptr(), nchunks as libc::c_ulong, 0)
    };
    usize::try_from(nread).unwrap_or(0)
}
