// Copyright 2024 Palantir Technologies, Inc.
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

use crate::debug::Diagnostic;
use bytes::Bytes;
use conjure_error::Error;
use http::HeaderValue;
use refreshable::Refreshable;
use std::{
    collections::BTreeSet,
    env,
    ffi::{c_char, CString},
    fmt::Write,
    fs,
};
use tempfile::NamedTempFile;
use witchcraft_log::{info, warn};
use witchcraft_server_config::runtime::RuntimeConfig;

#[no_mangle]
#[allow(non_upper_case_globals)]
#[cfg(target_os = "linux")]
static malloc_conf: &c_char = unsafe { &*c"prof:true,prof_active:false".as_ptr() };

#[no_mangle]
#[allow(non_upper_case_globals)]
#[cfg(target_os = "macos")]
static _rjem_malloc_conf: &c_char = unsafe { &*c"prof:true,prof_active:false".as_ptr() };

pub fn init<R>(runtime: &Refreshable<R, Error>)
where
    R: AsRef<RuntimeConfig> + PartialEq + 'static + Sync + Send,
{
    runtime
        .map(|r| r.as_ref().diagnostics().jemalloc().prof_active())
        .subscribe(|active| {
            info!("setting prof.active", safe: { value: active });
            if let Err(e) = unsafe {
                tikv_jemalloc_ctl::raw::write::<bool>(c"prof.active".to_bytes_with_nul(), *active)
            } {
                warn!("error setting prof.active", error: Error::internal_safe(e));
            }
        })
        .leak();

    runtime
        .map(|r| r.as_ref().diagnostics().jemalloc().lg_prof_sample())
        .subscribe(|lg_prof_sample| {
            info!("setting prof.reset", safe: { value: lg_prof_sample });
            if let Err(e) = unsafe {
                tikv_jemalloc_ctl::raw::write::<usize>(
                    c"prof.reset".to_bytes_with_nul(),
                    *lg_prof_sample,
                )
            } {
                warn!("error setting prof.reset", error: Error::internal_safe(e));
            }
        })
        .leak();
}

/// A diagnostic which returns a heap profile.
///
/// Requires jemalloc.
pub struct HeapProfileDiagnostic;

impl Diagnostic for HeapProfileDiagnostic {
    fn type_(&self) -> &str {
        "rust.heap.profile.v1"
    }

    fn content_type(&self) -> HeaderValue {
        HeaderValue::from_static("text/plain")
    }

    fn safe_loggable(&self) -> bool {
        true
    }

    fn result(&self) -> Result<Bytes, Error> {
        let file = NamedTempFile::new_in("var/data/tmp").map_err(Error::internal_safe)?;

        let path_str = CString::new(file.path().as_os_str().as_encoded_bytes())
            .map_err(Error::internal_safe)?;
        unsafe {
            tikv_jemalloc_ctl::raw::write::<*const c_char>(
                c"prof.dump".to_bytes_with_nul(),
                path_str.as_ptr(),
            )
            .map_err(Error::internal_safe)?;
        }

        let profile = fs::read_to_string(file.path()).map_err(Error::internal_safe)?;
        let symbolized_profile = symbolize_profile(&profile);
        Ok(Bytes::from(symbolized_profile))
    }
}

/// Adds symbol mappings to a jeprof profile.
///
/// The raw profile looks like:
///
/// ```raw
/// heap_v2/524288
///   t*: 28106: 56637512 [0: 0]
///   [...]
///   t3: 352: 16777344 [0: 0]
///   [...]
///   t99: 17754: 29341640 [0: 0]
///   [...]
/// @ 0x5f86da8 0x5f5a1dc [...] 0x29e4d4e 0xa200316 0xabb2988 [...]
///   t*: 13: 6688 [0: 0]
///   t3: 12: 6496 [0: 0]
///   t99: 1: 192 [0: 0]
/// [...]
///
/// MAPPED_LIBRARIES:
/// [...]
/// ```
///
/// Where the lines starting with `@` correspond to a call chain represented as a sequence of addresses.
///
/// We parse out the call chain addresses and resolve them to symbols (including inlined functions separated by `--`).
/// They are added to a special `symbols` section at the start of the file along with the binary name:
///
/// ```raw
/// --- symbol
/// binary=/usr/local/bin/my_binary
/// 0x000000000029e4d4e someMethod
/// 0x00000000005f86da8 function1--function2
/// [...]
/// ---
/// --- heap
/// heap_v2/524288
///   t*: 28106: 56637512 [0: 0]
///   [...]
///   t3: 352: 16777344 [0: 0]
///   [...]
///   t99: 17754: 29341640 [0: 0]
///   [...]
/// @ 0x5f86da8 0x5f5a1dc [...] 0x29e4d4e 0xa200316 0xabb2988 [...]
///   t*: 13: 6688 [0: 0]
///   t3: 12: 6496 [0: 0]
///   t99: 1: 192 [0: 0]
/// [...]
///
/// MAPPED_LIBRARIES:
/// [...]
/// ```
///
/// This enables jeprof to work with the profile file directly instead of having to resolve the symbols against a local
/// copy of the binaries. Once symbolized, the `MAPPED_LIBRARIES` section is no longer neccessary but we keep it around
/// since some workflows (e.g. resolving call chains to specific lines in source files) require re-resolution.
///
/// Since we only currently care about handling profile output produced by the same process, we just directly resolve
/// the addresses with the `backtrace` crate rather than parsing the `MAPPED_LIBRARIES` section.
fn symbolize_profile(raw: &str) -> String {
    let mut addrs = BTreeSet::new();

    for line in raw.lines() {
        let Some(raw_addrs) = line.strip_prefix("@ ") else {
            continue;
        };

        addrs.extend(
            raw_addrs
                .split(" ")
                .flat_map(|raw_addr| {
                    raw_addr
                        .strip_prefix("0x")
                        .and_then(|s| usize::from_str_radix(s, 16).ok())
                })
                .map(|addr| addr - 1),
        );
    }

    let mut out = String::new();

    writeln!(out, "--- symbol").unwrap();
    if let Ok(binary) = env::current_exe() {
        writeln!(out, "binary={}", binary.display()).unwrap();
    }

    for addr in addrs {
        let mut symbols = vec![];
        backtrace::resolve(addr as *mut _, |symbol| {
            if let Some(name) = symbol.name() {
                symbols.push(name.to_string());
            }
        });

        if !symbols.is_empty() {
            symbols.reverse();
            writeln!(out, "{addr:#016x} {}", symbols.join("--")).unwrap();
        }
    }

    writeln!(out, "---").unwrap();
    writeln!(out, "--- heap").unwrap();
    out.push_str(raw);

    out
}
