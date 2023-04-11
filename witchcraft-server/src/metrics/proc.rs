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
use std::mem::MaybeUninit;
use std::{fs, io};
use witchcraft_metrics::MetricRegistry;

pub fn register_metrics(metrics: &MetricRegistry) {
    metrics.gauge("process.threads", || num_threads().unwrap_or(0));

    metrics.gauge("process.filedescriptor", || filedescriptor().unwrap_or(0.));
}

fn num_threads() -> Option<i64> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    parse_num_threads(&stat)
}

fn parse_num_threads(stat: &str) -> Option<i64> {
    // The stat pseudo-file is nominally a sequence of space separated values, but one, comm, is
    // the process name truncated to 16 characters and parenthesized. Since the process name itself
    // can contain both ` ` and `)`, we first split on the last `)` and then split on spaces from
    // there. Fortunately, comm is the only non-numeric value.
    //
    // num_threads is entry 20, which maps out to index 18 after the handling of comm.
    stat.rsplit(')').next()?.split(' ').nth(18)?.parse().ok()
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

    #[allow(clippy::unnecessary_cast)]
    pub fn cur(&self) -> u64 {
        self.0.rlim_cur as u64
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn num_threads() {
        let stat = "9 (cat) R 1 9 1 34816 9 4194304 80 0 0 0 0 0 0 0 20 0 1 0 4977616 2138112 113 \
            18446744073709551615 187650934964224 187650934994048 281474646050032 0 0 0 0 0 0 0 0 0 \
            17 7 0 0 0 0 0 187650935060992 187650935062800 187651580477440 281474646051178 \
            281474646051198 281474646051198 281474646052843 0";

        assert_eq!(parse_num_threads(stat), Some(1));
    }
}
