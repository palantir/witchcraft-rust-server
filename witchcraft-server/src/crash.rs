// Copyright 2022 Palantir Technologies, Inc.
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
use conjure_error::Error;
use conjure_object::Utc;
use core::fmt::{self, Write};
use libc::close;
use libc::read;
#[cfg(target_os = "linux")]
use libc::O_RDONLY;
use libc::{
    c_int, c_void, open, sigaction, siginfo_t, write, O_CREAT, O_TRUNC, O_WRONLY, SA_RESETHAND,
    SA_SIGINFO, SIGABRT, SIGBUS, SIGFPE, SIGILL, SIGSEGV,
};
use std::fs::{self, File};
use std::io;
use std::io::Read;
use std::mem;
use std::ptr;
use std::str;
#[cfg(target_os = "linux")]
use unwind::{get_context, Cursor, RegNum};
use witchcraft_log::fatal;

pub fn init() -> Result<(), Error> {
    notify_crash()?;
    install_handler()?;

    Ok(())
}

fn notify_crash() -> Result<(), Error> {
    let path = "var/log/crash.log";

    let mut crash_log = match File::open(path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(Error::internal_safe(e)),
    };

    // Only grab the first 512 KB of the crash log just in case it's huge. We'll save it on disk for later retrieval
    // if necessary. The log cleanup logic will get rid of it eventually if not.
    let mut contents = vec![];
    crash_log
        .by_ref()
        .take(512 * 1024)
        .read_to_end(&mut contents)
        .map_err(Error::internal_safe)?;
    if crash_log.read(&mut [0]).map_err(Error::internal_safe)? != 0 {
        contents.extend_from_slice(b"<truncated>");
    }
    let contents = String::from_utf8_lossy(&contents);

    let now = Utc::now();
    let new_path = format!("var/log/crash-{}.log", now.to_rfc3339());

    fatal!(
        "the previous instance of this process crashed",
        safe: {
            info: contents,
            path: new_path,
        },
    );

    fs::rename(path, new_path).map_err(Error::internal_safe)?;

    Ok(())
}

fn install_handler() -> Result<(), Error> {
    unsafe {
        let mut new = mem::zeroed::<sigaction>();
        new.sa_sigaction = handler as usize;
        new.sa_flags = SA_RESETHAND | SA_SIGINFO;

        for signal in [SIGILL, SIGABRT, SIGFPE, SIGSEGV, SIGBUS] {
            let ret = sigaction(signal, &new, ptr::null_mut());
            if ret != 0 {
                return Err(Error::internal_safe(io::Error::last_os_error()));
            }
        }
    }

    Ok(())
}

/// **WARNING:** All of the code in here is extremely sensitive!
///
/// The set of things you are allowed to do in a signal handler is extremely limited (see `man signal-safety`). In
/// particular, it is very not-okay to allocate or deallocate anything.
///
/// The handler here is written directly against the raw C functions which are declared async-signal safe to ensure that
/// we know exactly what code is running here. In addition, libunwind is documented to be async-signal safe when
/// operating over the local address space.
unsafe extern "C" fn handler(sig: c_int, info: *mut siginfo_t, ucontext: *mut c_void) {
    let _ = handler_inner(sig, info, ucontext);
}

unsafe fn handler_inner(
    sig: c_int,
    info: *mut siginfo_t,
    _ucontext: *mut c_void,
) -> Result<(), ()> {
    let ret = open(
        "var/log/crash.log\0".as_bytes().as_ptr().cast(),
        O_WRONLY | O_CREAT | O_TRUNC,
    );
    if ret < 0 {
        return Err(());
    }
    let mut file = RawFile(ret);

    writeln!(file, "Signal: {}", sig).map_err(|_| ())?;
    writeln!(file, "Code: {}", (*info).si_code).map_err(|_| ())?;
    #[cfg(target_os = "linux")]
    write_stacktrace(&mut file)?;
    #[cfg(target_os = "linux")]
    write_proc_maps(&mut file)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn write_stacktrace(file: &mut RawFile) -> Result<(), ()> {
    writeln!(file).map_err(|_| ())?;
    writeln!(file, "Backtrace:").map_err(|_| ())?;

    get_context!(context);
    let mut cursor = Cursor::local(context).map_err(|_| ())?;
    let cursor_backup = cursor.clone();
    // skip past the signal handler if we find it
    while !cursor.is_signal_frame().unwrap_or(false) {
        if !cursor.step().unwrap_or(false) {
            cursor = cursor_backup;
            break;
        }
    }

    let mut name = [0; 512];
    while cursor.step().map_err(|_| ())? {
        let ip = cursor.register(RegNum::IP).map_err(|_| ())?;
        let mut offset = 0;

        let info_result = cursor.procedure_info();
        let name_result = cursor
            .procedure_name_raw(&mut name, &mut offset)
            .or_else(|e| {
                if e == unwind::Error::NOMEM {
                    Ok(())
                } else {
                    Err(e)
                }
            });
        match (info_result, name_result) {
            (Ok(info), Ok(())) if ip == info.start_ip() + offset => {
                let name = name
                    .iter()
                    .position(|b| *b == 0)
                    .and_then(|i| str::from_utf8(&name[..i]).ok())
                    .unwrap_or("????");
                writeln!(
                    file,
                    "{:#016x} - {} ({:#016x}) + {:#x}",
                    ip,
                    rustc_demangle::demangle(name),
                    info.start_ip(),
                    offset,
                )
                .map_err(|_| ())?;
            }
            _ => writeln!(file, "{:#016x} - ????", ip).map_err(|_| ())?,
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn write_proc_maps(file: &mut RawFile) -> Result<(), ()> {
    let ret = unsafe { open("/proc/self/maps\0".as_bytes().as_ptr().cast(), O_RDONLY) };
    if ret < 0 {
        return Err(());
    }
    let maps = RawFile(ret);

    writeln!(file).map_err(|_| ())?;
    writeln!(file, "/proc/self/maps:").map_err(|_| ())?;

    let mut buf = [0; 1024];
    loop {
        let nread = maps.read(&mut buf)?;
        if nread == 0 {
            break;
        }

        file.write_all(&buf[..nread])?;
    }

    Ok(())
}

struct RawFile(c_int);

impl Drop for RawFile {
    fn drop(&mut self) {
        unsafe {
            close(self.0);
        }
    }
}

impl RawFile {
    fn write_all(&self, mut buf: &[u8]) -> Result<(), ()> {
        while !buf.is_empty() {
            let written = unsafe { write(self.0, buf.as_ptr().cast(), buf.len()) };
            if written <= 0 {
                return Err(());
            }
            buf = &buf[written as usize..];
        }
        Ok(())
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize, ()> {
        let nread = unsafe { read(self.0, buf.as_mut_ptr().cast(), buf.len()) };
        if nread < 0 {
            Err(())
        } else {
            Ok(nread as usize)
        }
    }
}

impl fmt::Write for RawFile {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_all(s.as_bytes()).map_err(|()| fmt::Error)
    }
}
