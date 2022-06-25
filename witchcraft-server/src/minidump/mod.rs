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
use conjure_object::Uuid;
use crash_handler::CrashHandler;
use minidumper::{LoopAction, MinidumpBinary, ServerHandler};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs, io, mem, thread};
use witchcraft_log::{debug, error};

mod frame_resolver;
mod log;

const SOCKET_ADDR: &str = "var/data/tmp/minidump.sock";

pub async fn init() -> Result<(), Error> {
    log_dumps().await?;

    fs::create_dir_all(Path::new(SOCKET_ADDR).parent().unwrap()).map_err(Error::internal_safe)?;

    let exe = env::current_exe().map_err(Error::internal_safe)?;
    let child = Command::new(exe)
        .arg("minidump")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Error::internal_safe)?;

    // Ensure that the child's stdin says open until this process exits since that's how it detects the parent exiting.
    mem::forget(child.stdin);

    let client = wait_for_client()?;

    let guard = CrashHandler::attach(unsafe {
        crash_handler::make_crash_event(move |context| {
            let _ = client.request_dump(&context);
            crash_handler::CrashEventResult::Handled(true)
        })
    })
    .map_err(Error::internal_safe)?;
    mem::forget(guard);

    Ok(())
}

fn wait_for_client() -> Result<minidumper::Client, Error> {
    for _ in 0..50 {
        match minidumper::Client::with_name(Path::new(SOCKET_ADDR)) {
            Ok(client) => return Ok(client),
            Err(e) => debug!(
                "error opening minidump client",
                error: Error::internal_safe(e)
            ),
        }

        thread::sleep(Duration::from_millis(100));
    }

    Err(Error::internal_safe("unable to connect to minidump server"))
}

pub fn server() -> Result<(), Error> {
    let shutdown = Arc::new(AtomicBool::new(false));

    thread::spawn({
        let shutdown = shutdown.clone();
        move || {
            let _ = io::stdin().read(&mut [0]);
            shutdown.store(true, Ordering::Relaxed);
        }
    });

    minidumper::Server::with_name(Path::new(SOCKET_ADDR))
        .map_err(Error::internal_safe)?
        .run(Box::new(WitchcraftServerHandler), &shutdown)
        .map_err(Error::internal_safe)
}

struct WitchcraftServerHandler;

impl ServerHandler for WitchcraftServerHandler {
    fn create_minidump_file(&self) -> io::Result<(File, PathBuf)> {
        let dir = Path::new("var/log");
        fs::create_dir_all(dir)?;

        let path = dir.join(format!("{}.dmp.new", Uuid::new_v4()));
        let file = File::create(&path)?;
        Ok((file, path))
    }

    fn on_minidump_created(
        &self,
        _result: Result<MinidumpBinary, minidumper::Error>,
    ) -> LoopAction {
        LoopAction::Continue
    }

    fn on_message(&self, _kind: u32, _buffer: Vec<u8>) {}
}

pub async fn log_dumps() -> Result<(), Error> {
    let mut dir = tokio::fs::read_dir("var/log")
        .await
        .map_err(Error::internal_safe)?;

    while let Some(entry) = dir.next_entry().await.map_err(Error::internal_safe)? {
        let path = entry.path();

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(s) => s,
            None => continue,
        };

        if !file_name.ends_with(".dmp.new") {
            continue;
        }

        let new_file_name = file_name.strip_suffix(".new").unwrap();
        let new_path = path.with_file_name(new_file_name);
        tokio::fs::rename(&path, &new_path)
            .await
            .map_err(Error::internal_safe)?;

        if let Err(e) = log::log_minidump(&new_path).await {
            error!("error logging minidump", safe: { path: new_path.to_string_lossy() }, error: e);
        }
    }

    Ok(())
}
