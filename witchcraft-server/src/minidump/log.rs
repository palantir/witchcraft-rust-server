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

use crate::minidump::symbol_provider::{Arena, WitchcraftSymbolProvider};
use conjure_error::Error;
use minidump::Minidump;
use minidump_processor::ProcessState;
use minidump_unwind::{CallStack, StackFrame};
use std::fmt::Write;
use std::path::Path;
use witchcraft_log::fatal;

pub async fn log_minidump(p: &Path) -> Result<(), Error> {
    let info = process_minidump(p).await?;

    fatal!(
        "a previous instance of the process crashed",
        safe: {
            info: info,
            minidump: p.to_string_lossy()
        },
    );

    Ok(())
}

pub async fn process_minidump(p: &Path) -> Result<String, Error> {
    let dump = Minidump::read_path(p).map_err(Error::internal_safe)?;

    let arena = Arena::new();
    let symbol_provider = WitchcraftSymbolProvider::new(&arena);

    let state = minidump_processor::process_minidump(&dump, &symbol_provider)
        .await
        .map_err(Error::internal_safe)?;
    let info = format_dump(&state);

    Ok(info)
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
    let mut idx = Some(idx);
    for inline in &frame.inlines {
        format_frame_entry(
            buf,
            &mut idx,
            &inline.function_name,
            inline.source_file_name.as_deref(),
            inline.source_line,
        );
    }

    format_frame_entry(
        buf,
        &mut idx,
        frame.function_name.as_deref().unwrap_or("???"),
        frame.source_file_name.as_deref(),
        frame.source_line,
    );
}

fn format_frame_entry(
    buf: &mut String,
    idx: &mut Option<usize>,
    name: &str,
    file: Option<&str>,
    line: Option<u32>,
) {
    match *idx {
        Some(i) => {
            write!(buf, "{i:4} ").unwrap();
            *idx = None;
        }
        None => write!(buf, "     ").unwrap(),
    }
    writeln!(buf, "{name}").unwrap();
    if let Some(file) = file {
        writeln!(buf, "              at {}:{}", file, line.unwrap_or(0)).unwrap();
    }
}
