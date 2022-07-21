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

use crate::minidump::frame_resolver::FrameResolver;
use async_trait::async_trait;
use conjure_error::Error;
use minidump::{Minidump, Module};
use minidump_processor::{
    CallStack, FileError, FileKind, ProcessState, StackFrame, SymbolError, SymbolFile,
    SymbolSupplier, Symbolizer,
};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use symbolic::cfi::CfiCache;
use symbolic::common::ByteView;
use symbolic::debuginfo::Object;
use typed_arena::Arena;
use witchcraft_log::{fatal, warn};

pub async fn log_minidump(p: &Path) -> Result<(), Error> {
    let dump = Minidump::read_path(p).map_err(Error::internal_safe)?;

    let symbolizer = Symbolizer::new(WitchcraftSymbolSupplier);

    let state = minidump_processor::process_minidump(&dump, &symbolizer)
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

// minidump_processor doesn't yet support inline frames internally, so we only provide CFI info for unwinding.
struct WitchcraftSymbolSupplier;

#[async_trait]
impl SymbolSupplier for WitchcraftSymbolSupplier {
    async fn locate_symbols(
        &self,
        module: &(dyn Module + Sync),
    ) -> Result<SymbolFile, SymbolError> {
        let buf = ByteView::open(&*module.code_file()).map_err(SymbolError::LoadError)?;
        let object = Object::parse(&buf).map_err(|_| SymbolError::NotFound)?;

        if module.code_identifier() != object.code_id() {
            warn!(
                "code ID mismatch for module, unwinding will be inaccurate",
                safe: {
                    file: module.code_file(),
                    expected: format_args!("{:?}", module.code_identifier()),
                    actual: format_args!("{:?}", object.code_id()),
                },
            );
            return Err(SymbolError::NotFound);
        }

        let cfi_cache = CfiCache::from_object(&object).map_err(|_| SymbolError::NotFound)?;

        SymbolFile::from_bytes(cfi_cache.as_slice())
    }

    async fn locate_file(
        &self,
        _module: &(dyn Module + Sync),
        _file_kind: FileKind,
    ) -> Result<PathBuf, FileError> {
        Err(FileError::NotFound)
    }
}

fn format_dump(state: &ProcessState) -> String {
    let mut buf = String::new();

    if let Some(crash_reason) = state.crash_reason {
        writeln!(buf, "Crash reason: {}", crash_reason).unwrap();
    }
    if let Some(crash_address) = state.crash_address {
        writeln!(buf, "Crash address: {:#0x}", crash_address).unwrap();
    }

    let arena = Arena::new();
    let mut frame_resolver = FrameResolver::new(&arena);

    if let Some(requesting_thread) = state.requesting_thread {
        writeln!(buf).unwrap();
        writeln!(buf, "Crashed thread:").unwrap();
        format_thread(
            &mut buf,
            &state.threads[requesting_thread],
            &mut frame_resolver,
        );
    }

    writeln!(buf, "Threads:").unwrap();
    for (i, thread) in state.threads.iter().enumerate() {
        if state.requesting_thread == Some(i) {
            continue;
        }

        format_thread(&mut buf, thread, &mut frame_resolver);
    }

    buf
}

fn format_thread(buf: &mut String, thread: &CallStack, frame_resolver: &mut FrameResolver<'_>) {
    match &thread.thread_name {
        Some(thread_name) => {
            writeln!(buf, "Thread {} - {}", thread.thread_id, thread_name).unwrap()
        }
        None => writeln!(buf, "Thread {}", thread.thread_id).unwrap(),
    }

    for (i, frame) in thread.frames.iter().enumerate() {
        format_frame(buf, frame, i, frame_resolver);
    }

    writeln!(buf).unwrap();
}

fn format_frame(
    buf: &mut String,
    frame: &StackFrame,
    idx: usize,
    frame_resolver: &mut FrameResolver<'_>,
) {
    let mut first = true;
    frame_resolver.resolve(frame, |entry| {
        if first {
            write!(buf, "{idx:4}: ").unwrap();
            first = false;
        } else {
            write!(buf, "      ").unwrap();
        }
        writeln!(buf, "{}", entry.name.unwrap_or("???")).unwrap();
        if let Some(file) = entry.file {
            writeln!(buf, "              at {}:{}", file, entry.line.unwrap_or(0)).unwrap();
        }
    });
}
