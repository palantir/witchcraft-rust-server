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
use conjure_object::log_safety::AssertLogSafe;
use std::path::Path;
use witchcraft_log::fatal;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub async fn log_minidump(p: &Path) -> Result<(), Error> {
    let info = tokio::task::spawn_blocking({
        let p = p.to_owned();
        move || process_minidump(&p)
    })
    .await
    .map_err(Error::internal_safe)??;

    fatal!(
        "a previous instance of the process crashed",
        safe: {
            info: AssertLogSafe(info),
            minidump: AssertLogSafe(p.to_string_lossy())
        },
    );

    Ok(())
}

/// Minidumps can only be processed on Linux, so on other platforms we just report the crash.
#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)))]
pub async fn log_minidump(p: &Path) -> Result<(), Error> {
    fatal!(
        "a previous instance of the process crashed",
        safe: {
            minidump: AssertLogSafe(p.to_string_lossy())
        },
    );

    Ok(())
}

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
pub use process::process_minidump;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod process {
    use crate::unwind::modules;
    use crate::unwind::symbolize::{Frame, Symbolizer};
    use crate::unwind::{Regs, Unwind};
    use conjure_error::Error;
    use minidump::{
        Minidump, MinidumpContext, MinidumpException, MinidumpMiscInfo, MinidumpModuleList,
        MinidumpSystemInfo, MinidumpThreadList, MinidumpThreadNames, Module,
    };
    use std::fmt::Write;
    use std::path::Path;

    struct Thread {
        id: u32,
        name: Option<String>,
        frames: Vec<u64>,
    }

    /// Unwinds and symbolizes the threads of a minidump of a process on this machine.
    pub fn process_minidump(p: &Path) -> Result<String, Error> {
        let dump = Minidump::read_path(p).map_err(Error::internal_safe)?;

        let system_info = dump
            .get_stream::<MinidumpSystemInfo>()
            .map_err(Error::internal_safe)?;
        let misc = dump.get_stream::<MinidumpMiscInfo>().ok();
        let thread_list = dump
            .get_stream::<MinidumpThreadList>()
            .map_err(Error::internal_safe)?;
        let module_list = dump
            .get_stream::<MinidumpModuleList>()
            .map_err(Error::internal_safe)?;
        let thread_names = dump.get_stream::<MinidumpThreadNames>().ok();
        let exception = dump.get_stream::<MinidumpException>().ok();
        let memory = dump.get_memory().unwrap_or_default();

        let mut unwind = Unwind::new();
        for module in module_list.iter() {
            let build_id = module
                .code_identifier()
                .and_then(|id| decode_hex(id.as_str()));
            let path = module.code_file();
            if let Some(module) =
                modules::from_file(Path::new(&*path), module.base_address(), build_id)
            {
                unwind.add_module(module);
            }
        }

        let crashing_thread = exception.as_ref().map(|e| e.get_crashing_thread_id());

        let mut threads = vec![];
        for thread in &thread_list.threads {
            let id = thread.raw.thread_id;
            // The crashing thread's context in the thread list is that of the signal handler.
            let context = match &exception {
                Some(exception) if Some(id) == crashing_thread => {
                    exception.context(&system_info, misc.as_ref())
                }
                _ => thread.context(&system_info, misc.as_ref()),
            };

            let mut frames = vec![];
            if let (Some(context), Some(stack)) = (context, thread.stack_memory(&memory)) {
                if let Some(regs) = regs(&context) {
                    unwind.unwind(&regs, stack.base_address(), stack.bytes(), &mut frames);
                }
            }

            threads.push(Thread {
                id,
                name: thread_names
                    .as_ref()
                    .and_then(|n| n.get_name(id))
                    .map(|n| n.into_owned()),
                frames,
            });
        }

        let mut symbolizer = Symbolizer::new(&unwind);
        let mut buf = String::new();

        if let Some(exception) = &exception {
            let reason = exception.get_crash_reason(system_info.os, system_info.cpu);
            let address = exception.get_crash_address(system_info.os, system_info.cpu);
            writeln!(buf, "Crash reason: {reason}").unwrap();
            writeln!(buf, "Crash address: {address}").unwrap();
        }

        if let Some(thread) = threads.iter().find(|t| Some(t.id) == crashing_thread) {
            writeln!(buf).unwrap();
            writeln!(buf, "Crashed thread:").unwrap();
            format_thread(&mut buf, &mut symbolizer, thread);
        }

        writeln!(buf, "Threads:").unwrap();
        for thread in &threads {
            if Some(thread.id) == crashing_thread {
                continue;
            }

            format_thread(&mut buf, &mut symbolizer, thread);
        }

        Ok(buf)
    }

    #[cfg(target_arch = "x86_64")]
    fn regs(context: &MinidumpContext) -> Option<Regs> {
        Some(Regs {
            pc: context.get_instruction_pointer(),
            sp: context.get_stack_pointer(),
            fp: context.get_register("rbp")?,
        })
    }

    #[cfg(target_arch = "aarch64")]
    fn regs(context: &MinidumpContext) -> Option<Regs> {
        Some(Regs {
            pc: context.get_instruction_pointer(),
            sp: context.get_stack_pointer(),
            fp: context.get_register("fp")?,
            lr: context.get_register("lr")?,
        })
    }

    fn decode_hex(s: &str) -> Option<Vec<u8>> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
            .collect()
    }

    fn format_thread(buf: &mut String, symbolizer: &mut Symbolizer<'_>, thread: &Thread) {
        match &thread.name {
            Some(thread_name) => writeln!(buf, "Thread {} - {}", thread.id, thread_name).unwrap(),
            None => writeln!(buf, "Thread {}", thread.id).unwrap(),
        }

        for (i, address) in thread.frames.iter().enumerate() {
            format_frame(buf, &symbolizer.symbolize(*address), i);
        }

        writeln!(buf).unwrap();
    }

    fn format_frame(buf: &mut String, frames: &[Frame], idx: usize) {
        let mut idx = Some(idx);

        if frames.is_empty() {
            format_frame_entry(buf, &mut idx, "???", None, None);
        }

        // Inlined functions come first, followed by the function they were inlined into.
        for frame in frames {
            format_frame_entry(
                buf,
                &mut idx,
                frame.function.as_deref().unwrap_or("???"),
                frame.file.as_deref(),
                frame.line,
            );
        }
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
}
