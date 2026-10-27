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
//! Encoding of profiles in the [pprof] format.
//!
//! [pprof]: https://github.com/google/pprof/blob/main/proto/profile.proto
use crate::debug::cpu_profile::profile::{
    Function, Label, Line, Location, Mapping, Profile, Sample, ValueType,
};
use crate::unwind::symbolize::Symbolizer;
use crate::unwind::{LoadedModule, Unwind};
use flate2::write::GzEncoder;
use flate2::Compression;
use prost::Message;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::time::{Duration, SystemTime};

/// A set of identical stacks sampled from a thread.
pub struct StackCount<'a> {
    pub tid: i32,
    pub thread_name: &'a str,
    /// Addresses ready for symbolization, leaf first.
    pub frames: &'a [u64],
    pub count: u64,
}

pub struct ProfileInfo {
    pub start: SystemTime,
    pub duration: Duration,
    pub period: Duration,
    pub dropped: u64,
}

/// Builds a gzipped, symbolized pprof profile.
pub fn build<'a>(
    info: &ProfileInfo,
    stacks: impl IntoIterator<Item = StackCount<'a>>,
    unwind: &Unwind,
) -> Vec<u8> {
    let mut builder = Builder {
        profile: Profile::default(),
        strings: Interner::default(),
        functions: Interner::default(),
        locations: HashMap::new(),
        mappings: HashMap::new(),
        symbolizer: Symbolizer::new(unwind),
        unwind,
    };
    // The empty string must be at index 0.
    builder.string("");

    let samples = builder.value_type("samples", "count");
    let cpu = builder.value_type("cpu", "nanoseconds");
    builder.profile.sample_type = vec![samples, cpu];
    builder.profile.period_type = Some(cpu);
    builder.profile.default_sample_type = cpu.r#type;
    let period = info.period.as_nanos() as i64;
    builder.profile.period = period;
    builder.profile.time_nanos = info
        .start
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as i64);
    builder.profile.duration_nanos = info.duration.as_nanos() as i64;
    if info.dropped > 0 {
        let comment = builder.string(&format!(
            "{} samples were dropped because the sample buffer was full",
            info.dropped
        ));
        builder.profile.comment.push(comment);
    }

    // By convention, the first mapping is the main executable.
    if let Some(main) = unwind.modules().find(|m| m.module.main) {
        builder.mapping(main);
    }

    let thread_name = builder.string("thread_name");
    let thread_id = builder.string("thread_id");
    for stack in stacks {
        let location_id = stack.frames.iter().map(|a| builder.location(*a)).collect();
        let sample = Sample {
            location_id,
            value: vec![stack.count as i64, stack.count as i64 * period],
            label: vec![
                Label {
                    key: thread_name,
                    str: builder.string(stack.thread_name),
                    ..Default::default()
                },
                Label {
                    key: thread_id,
                    num: stack.tid.into(),
                    ..Default::default()
                },
            ],
        };
        builder.profile.sample.push(sample);
    }

    builder.profile.string_table = builder.strings.values;

    let mut encoder = GzEncoder::new(vec![], Compression::default());
    encoder
        .write_all(&builder.profile.encode_to_vec())
        .expect("writing to a Vec can't fail");
    encoder.finish().expect("writing to a Vec can't fail")
}

struct Interner<T> {
    ids: HashMap<T, i64>,
    values: Vec<T>,
}

impl<T> Default for Interner<T> {
    fn default() -> Self {
        Interner {
            ids: HashMap::new(),
            values: vec![],
        }
    }
}

impl<T> Interner<T>
where
    T: Hash + Eq + Clone,
{
    fn intern(&mut self, value: T) -> (i64, bool) {
        if let Some(id) = self.ids.get(&value) {
            return (*id, false);
        }

        let id = self.values.len() as i64;
        self.ids.insert(value.clone(), id);
        self.values.push(value);
        (id, true)
    }
}

struct Builder<'a> {
    profile: Profile,
    strings: Interner<String>,
    // (name, filename) string IDs
    functions: Interner<(i64, i64)>,
    locations: HashMap<u64, u64>,
    // keyed by module start
    mappings: HashMap<u64, u64>,
    symbolizer: Symbolizer<'a>,
    unwind: &'a Unwind,
}

impl Builder<'_> {
    fn string(&mut self, s: &str) -> i64 {
        if let Some(id) = self.strings.ids.get(s) {
            return *id;
        }
        self.strings.intern(s.to_string()).0
    }

    fn value_type(&mut self, type_: &str, unit: &str) -> ValueType {
        ValueType {
            r#type: self.string(type_),
            unit: self.string(unit),
        }
    }

    fn function(&mut self, name: &str, filename: &str) -> u64 {
        let name = self.string(name);
        let filename = self.string(filename);
        let (index, new) = self.functions.intern((name, filename));
        let id = index as u64 + 1;
        if new {
            self.profile.function.push(Function {
                id,
                name,
                system_name: name,
                filename,
                ..Default::default()
            });
        }
        id
    }

    fn location(&mut self, address: u64) -> u64 {
        if let Some(id) = self.locations.get(&address) {
            return *id;
        }

        let id = self.profile.location.len() as u64 + 1;
        let (mapping_id, line) = match self.unwind.module(address) {
            Some(module) => (self.mapping(module), self.symbolize(address)),
            None => (0, vec![]),
        };
        // pprof's mapping flags indicate that every location in the mapping has that information.
        if mapping_id != 0 {
            let mapping = &mut self.profile.mapping[mapping_id as usize - 1];
            mapping.has_functions &= !line.is_empty();
            let has_lines = !line.is_empty()
                && line.iter().all(|l| {
                    l.line != 0 && self.profile.function[l.function_id as usize - 1].filename != 0
                });
            mapping.has_filenames &= has_lines;
            mapping.has_line_numbers &= has_lines;
            mapping.has_inline_frames &= has_lines;
        }
        self.profile.location.push(Location {
            id,
            mapping_id,
            address,
            line,
            ..Default::default()
        });
        self.locations.insert(address, id);

        id
    }

    fn mapping(&mut self, loaded: &LoadedModule) -> u64 {
        let module = &loaded.module;
        if let Some(id) = self.mappings.get(&module.range.start) {
            return *id;
        }

        let id = self.profile.mapping.len() as u64 + 1;
        let filename = self.string(&module.name);
        let build_id = match &module.build_id {
            Some(build_id) => self.string(&hex(build_id)),
            None => 0,
        };
        // The has_* flags are set once all locations have been symbolized.
        self.profile.mapping.push(Mapping {
            id,
            memory_start: module.range.start,
            memory_limit: module.range.end,
            file_offset: module.file_offset,
            filename,
            build_id,
            has_functions: true,
            has_filenames: true,
            has_line_numbers: true,
            has_inline_frames: true,
        });
        self.mappings.insert(module.range.start, id);

        id
    }

    fn symbolize(&mut self, address: u64) -> Vec<Line> {
        self.symbolizer
            .symbolize(address)
            .into_iter()
            .map(|frame| Line {
                function_id: self.function(
                    frame.function.as_deref().unwrap_or("??"),
                    frame.file.as_deref().unwrap_or(""),
                ),
                line: frame.line.map_or(0, i64::from),
                ..Default::default()
            })
            .collect()
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
