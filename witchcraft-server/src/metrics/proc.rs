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
use procinfo::pid;
use std::mem::MaybeUninit;
use std::{fs, io};
use witchcraft_metrics::MetricRegistry;

pub fn register_metrics(metrics: &MetricRegistry) {
    metrics.gauge("process.threads", || {
        pid::stat_self().map_or(0, |s| s.num_threads)
    });

    metrics.gauge("process.filedescriptor", || filedescriptor().unwrap_or(0.));
}

fn filedescriptor() -> io::Result<f32> {
    let mut files = 0;
    for r in fs::read_dir("/proc/self/fd")? {
        r?;
        files += 1;
    }

    let max_files = Rlimit::nofile()?.cur();

    Ok(files as f32 / max_files as f32)
}

struct Rlimit(libc::rlimit);

impl Rlimit {
    pub fn nofile() -> io::Result<Self> {
        unsafe {
            let mut limit = MaybeUninit::uninit();
            if libc::getrlimit(libc::RLIMIT_NOFILE, limit.as_mut_ptr()) != 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(Rlimit(limit.assume_init()))
        }
    }

    pub fn cur(&self) -> u64 {
        self.0.rlim_cur as u64
    }
}
