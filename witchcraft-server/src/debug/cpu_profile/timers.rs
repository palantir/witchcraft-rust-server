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
//! Per-thread CPU time timers.
//!
//! Like Go's profiler, each thread gets its own timer on its `CLOCK_THREAD_CPUTIME_ID` clock which sends SIGPROF to
//! that thread. Unlike a process-wide `ITIMER_PROF` timer, this samples each thread in proportion to its own CPU time.
//! The clock counts both user and system time, and since signals are delivered on the return to user space, time
//! spent in the kernel is attributed to the user space code that made the system call.
use conjure_error::Error;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use std::{fs, io, mem, ptr};

/// Manages timers for every thread in the process other than the calling thread.
pub struct Timers {
    period: Duration,
    self_tid: libc::pid_t,
    timers: HashMap<libc::pid_t, libc::timer_t>,
    names: HashMap<libc::pid_t, String>,
}

impl Timers {
    pub fn new(period: Duration) -> Self {
        Timers {
            period,
            // SAFETY: gettid is always safe to call.
            self_tid: unsafe { libc::gettid() },
            timers: HashMap::new(),
            names: HashMap::new(),
        }
    }

    /// Creates timers for new threads and deletes the timers of threads which have exited.
    ///
    /// A thread ID reused between scans keeps the dead timer of its previous owner, but that requires the PID space
    /// to wrap within one scan interval.
    pub fn scan(&mut self) -> Result<(), Error> {
        let mut live = HashSet::new();
        for entry in fs::read_dir("/proc/self/task").map_err(Error::internal_safe)? {
            let entry = entry.map_err(Error::internal_safe)?;
            let Some(tid) = entry.file_name().to_str().and_then(|s| s.parse().ok()) else {
                continue;
            };
            if tid == self.self_tid {
                continue;
            }
            live.insert(tid);

            if self.timers.contains_key(&tid) {
                continue;
            }
            // The thread may have exited.
            let Ok(comm) = fs::read_to_string(entry.path().join("comm")) else {
                continue;
            };
            match self.create(tid) {
                Ok(timer) => {
                    self.timers.insert(tid, timer);
                    self.names.insert(tid, comm.trim_end().to_string());
                }
                // The thread exited.
                Err(e) if e.raw_os_error() == Some(libc::EINVAL) => {}
                Err(e) => return Err(Error::internal_safe(e)),
            }
        }

        self.timers.retain(|tid, timer| {
            let live = live.contains(tid);
            if !live {
                // SAFETY: the timer is valid and is removed from the map.
                unsafe { libc::timer_delete(*timer) };
            }
            live
        });

        Ok(())
    }

    /// Returns the names of all threads which have had timers.
    pub fn names(&self) -> &HashMap<libc::pid_t, String> {
        &self.names
    }

    fn create(&self, tid: libc::pid_t) -> io::Result<libc::timer_t> {
        // The kernel's encoding of a thread's CPUCLOCK_SCHED clock, as used by pthread_getcpuclockid.
        let clock = ((!tid) << 3) | 6;

        // SAFETY: sigevent is zero-initializable.
        let mut event = unsafe { mem::zeroed::<libc::sigevent>() };
        event.sigev_notify = libc::SIGEV_THREAD_ID;
        event.sigev_signo = libc::SIGPROF;
        event.sigev_notify_thread_id = tid;

        let mut timer = ptr::null_mut();
        // SAFETY: the event and timer pointers are valid.
        if unsafe { libc::timer_create(clock, &mut event, &mut timer) } != 0 {
            return Err(io::Error::last_os_error());
        }

        let interval = libc::timespec {
            tv_sec: self.period.as_secs() as libc::time_t,
            tv_nsec: self.period.subsec_nanos() as libc::c_long,
        };
        let spec = libc::itimerspec {
            it_interval: interval,
            it_value: interval,
        };
        // SAFETY: the timer is valid.
        if unsafe { libc::timer_settime(timer, 0, &spec, ptr::null_mut()) } != 0 {
            let e = io::Error::last_os_error();
            // SAFETY: the timer is valid.
            unsafe { libc::timer_delete(timer) };
            return Err(e);
        }

        Ok(timer)
    }
}

impl Drop for Timers {
    fn drop(&mut self) {
        for timer in self.timers.values() {
            // SAFETY: the timer is valid.
            unsafe { libc::timer_delete(*timer) };
        }
    }
}
