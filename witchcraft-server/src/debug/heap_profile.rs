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
    ffi::{c_char, CString},
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
        Ok(Bytes::from(profile))
    }
}
