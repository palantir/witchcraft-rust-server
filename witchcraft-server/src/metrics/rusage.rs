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
use std::io;
use std::mem::MaybeUninit;
use std::time::Duration;

pub struct Rusage(libc::rusage);

impl Rusage {
    pub fn get_self() -> io::Result<Self> {
        unsafe {
            let mut rusage = MaybeUninit::uninit();
            if libc::getrusage(libc::RUSAGE_SELF, rusage.as_mut_ptr()) != 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(Rusage(rusage.assume_init()))
        }
    }

    pub fn user_time(&self) -> Duration {
        Duration::new(
            self.0.ru_utime.tv_sec as u64,
            self.0.ru_utime.tv_usec as u32 * 1000,
        )
    }

    pub fn system_time(&self) -> Duration {
        Duration::new(
            self.0.ru_stime.tv_sec as u64,
            self.0.ru_stime.tv_usec as u32 * 1000,
        )
    }

    pub fn blocks_read(&self) -> u64 {
        self.0.ru_inblock as u64
    }

    pub fn blocks_written(&self) -> u64 {
        self.0.ru_oublock as u64
    }
}
