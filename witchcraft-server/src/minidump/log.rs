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
use minidump::Minidump;
use minidump_processor::debuginfo::DebugInfoSymbolProvider;
use minidump_processor::{CallStack, ProcessState, StackFrame};
use std::fmt::Write;
use std::path::Path;
use witchcraft_log::fatal;

pub async fn log_minidump(p: &Path) -> Result<(), Error> {
    let dump = Minidump::read_path(p).map_err(Error::internal_safe)?;

    let state = minidump_processor::process_minidump(&dump, &DebugInfoSymbolProvider::default())
        .await
        .map_err(Error::internal_safe)?;

    fatal!(
        "a previous instance of the process crashed",
        safe: {
            info: format_dump(&state),
            minidump: p.to_string_lossy()
        },
    );

    Ok(())
}

fn format_dump(state: &ProcessState) -> String {
    let mut buf = String::new();

    if let Some(info) = &state.exception_info {
        writeln!(buf, "Crash reason: {}", info.reason).unwrap();
        writeln!(buf, "Crash address: {}", info.address).unwrap();
    }

    if let Some(requesting_thread) = state.requesting_thread {
        writeln!(buf).unwrap();
        writeln!(buf, "Crashed thread:").unwrap();
        format_thread(&mut buf, &state.threads[requesting_thread]);
    }

    writeln!(buf, "Threads:").unwrap();
    for (i, thread) in state.threads.iter().enumerate() {
        if state.requesting_thread == Some(i) {
            continue;
        }

        format_thread(&mut buf, thread);
    }

    buf
}

fn format_thread(buf: &mut String, thread: &CallStack) {
    match &thread.thread_name {
        Some(thread_name) => {
            writeln!(buf, "Thread {} - {}", thread.thread_id, thread_name).unwrap()
        }
        None => writeln!(buf, "Thread {}", thread.thread_id).unwrap(),
    }

    for (i, frame) in thread.frames.iter().enumerate() {
        format_frame(buf, frame, i);
    }

    writeln!(buf).unwrap();
}

fn format_frame(buf: &mut String, frame: &StackFrame, idx: usize) {
    writeln!(
        buf,
        "{idx:4}: {}",
        frame.function_name.as_deref().unwrap_or("???")
    )
    .unwrap();
    if let Some(file) = frame.function_name.as_deref() {
        writeln!(
            buf,
            "              at {}:{}",
            file,
            frame.source_line.unwrap_or(0)
        )
        .unwrap();
    }

    for inline in &frame.inlines {
        writeln!(buf, "      {}", inline.function_name).unwrap();
        if let Some(file) = inline.source_file_name.as_deref() {
            writeln!(
                buf,
                "              at {}:{}",
                file,
                inline.source_line.unwrap_or(0)
            )
            .unwrap();
        }
    }
}
