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
use crate::debug::Diagnostic;
use crate::minidump;
use crate::minidump::log;
use bytes::Bytes;
use conjure_error::Error;
use http::HeaderValue;
use minidump_writer::minidump_writer::MinidumpWriter;
use std::ffi::OsString;
use std::fs;
use std::io::Cursor;
use std::os::unix::prelude::{OsStrExt, OsStringExt};
use std::os::unix::process;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tokio::runtime::Handle;

/// A diagnostic which returns a stack trace of every thread in the server.
///
/// It is only supported on Linux.
pub struct ThreadDumpDiagnostic;

impl Diagnostic for ThreadDumpDiagnostic {
    fn type_(&self) -> &str {
        "rust.thread.dump.v1"
    }

    fn content_type(&self) -> HeaderValue {
        HeaderValue::from_static("text/plain")
    }

    fn safe_loggable(&self) -> bool {
        true
    }

    fn result(&self) -> Result<Bytes, Error> {
        let target_file = NamedTempFile::new_in("var/data/tmp").map_err(Error::internal_safe)?;

        let client = minidump::connect()?;
        client
            .send_message(0, target_file.path().as_os_str().as_bytes())
            .map_err(Error::internal_safe)?;
        // send_message doesn't wait for a response, so send a ping after to synchronize
        client.ping().map_err(Error::internal_safe)?;

        let info = Handle::current().block_on(log::process_minidump(target_file.path()))?;

        Ok(Bytes::from(info))
    }
}

pub fn handle_request(buf: Vec<u8>) {
    let target_file = PathBuf::from(OsString::from_vec(buf));
    let ppid = process::parent_id();
    let Ok(minidump) = MinidumpWriter::new(ppid as _, ppid as _).dump(&mut Cursor::new(vec![])) else {
        // We are in the child process so we unfortunately can't really log
        return;
    };
    let _ = fs::write(target_file, minidump);
}
