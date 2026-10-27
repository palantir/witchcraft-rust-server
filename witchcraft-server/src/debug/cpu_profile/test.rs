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
use super::*;
use crate::debug::cpu_profile::profile::{self, Profile};
use flate2::read::GzDecoder;
use prost::Message;
use std::fs::File;
use std::hint::black_box;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs, thread};

// Recordings can't overlap, so tests that record must hold this.
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[inline(never)]
fn spin_inner(stop: &AtomicBool) -> u64 {
    let mut x = 0u64;
    while !stop.load(Ordering::Relaxed) {
        for i in 0..1000 {
            x = black_box(x.wrapping_mul(31).wrapping_add(i));
        }
    }
    x
}

#[inline(never)]
fn spin_outer(stop: &AtomicBool) -> u64 {
    black_box(spin_inner(stop)) + 1
}

/// Spends most of its time in the kernel zeroing memory.
#[inline(never)]
fn read_zeros(stop: &AtomicBool) {
    let mut file = File::open("/dev/zero").unwrap();
    let mut buf = vec![0; 1024 * 1024];
    while !stop.load(Ordering::Relaxed) {
        file.read_exact(&mut buf).unwrap();
        black_box(&buf);
    }
}

struct Spawned {
    stop: Arc<AtomicBool>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl Spawned {
    fn new() -> Self {
        Spawned {
            stop: Arc::new(AtomicBool::new(false)),
            threads: vec![],
        }
    }

    fn spawn(&mut self, name: &str, delay: Duration, f: fn(&AtomicBool)) {
        let stop = self.stop.clone();
        let thread = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                thread::sleep(delay);
                f(&stop)
            })
            .unwrap();
        self.threads.push(thread);
    }
}

impl Drop for Spawned {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        for thread in self.threads.drain(..) {
            thread.join().unwrap();
        }
    }
}

struct Decoded {
    profile: Profile,
}

impl Decoded {
    fn new(data: &[u8]) -> Self {
        let mut buf = vec![];
        GzDecoder::new(data).read_to_end(&mut buf).unwrap();
        Decoded {
            profile: Profile::decode(&*buf).unwrap(),
        }
    }

    fn string(&self, id: i64) -> &str {
        &self.profile.string_table[id as usize]
    }

    fn thread_name(&self, sample: &profile::Sample) -> &str {
        let label = sample
            .label
            .iter()
            .find(|l| self.string(l.key) == "thread_name")
            .unwrap();
        self.string(label.str)
    }

    /// Returns the function names of a sample's stack, leaf first, including inlined frames.
    fn functions(&self, sample: &profile::Sample) -> Vec<&str> {
        sample
            .location_id
            .iter()
            .flat_map(|id| self.profile.location[*id as usize - 1].line.iter())
            .map(|line| {
                let function = &self.profile.function[line.function_id as usize - 1];
                self.string(function.name)
            })
            .collect()
    }

    /// Returns the number of samples from a thread, and the number for which `f` returns true for the stack.
    fn count(&self, thread: &str, f: impl Fn(&[&str]) -> bool) -> (i64, i64) {
        let mut total = 0;
        let mut matching = 0;
        for sample in &self.profile.sample {
            if self.thread_name(sample) != thread {
                continue;
            }
            total += sample.value[0];
            if f(&self.functions(sample)) {
                matching += sample.value[0];
            }
        }
        (total, matching)
    }
}

fn position(functions: &[&str], name: &str) -> Option<usize> {
    functions
        .iter()
        .position(|f| f.contains("cpu_profile") && f.ends_with(name))
}

#[test]
fn record_threads() {
    let _guard = TEST_LOCK.lock();

    let mut spawned = Spawned::new();
    spawned.spawn("spinner", Duration::ZERO, |stop| {
        spin_outer(stop);
    });
    spawned.spawn("zeros", Duration::ZERO, read_zeros);
    // This thread starts after the recording does.
    spawned.spawn("late-spinner", Duration::from_millis(500), |stop| {
        spin_outer(stop);
    });

    let data = record(Duration::from_secs(2)).unwrap();
    drop(spawned);

    if let Ok(path) = std::env::var("CPU_PROFILE_TEST_OUTPUT") {
        fs::write(path, &data).unwrap();
    }

    let profile = Decoded::new(&data);
    let exe = fs::read_link("/proc/self/exe").unwrap();
    assert_eq!(
        profile.string(profile.profile.mapping[0].filename),
        exe.to_str().unwrap()
    );
    assert!(profile
        .profile
        .mapping
        .iter()
        .any(|m| m.build_id != 0 && m.has_functions));

    // The spinner was on-CPU for the whole recording, so should have ~200 samples. Nearly all should be in spin_inner
    // with spin_outer as a caller, which requires DWARF unwinding in builds without frame pointers.
    let unwound = |functions: &[&str]| {
        let Some(inner) = position(functions, "spin_inner") else {
            return false;
        };
        position(&functions[inner + 1..], "spin_outer").is_some()
    };
    let (total, matching) = profile.count("spinner", unwound);
    assert!(total > 100, "{total} samples");
    assert!(matching * 10 > total * 9, "{matching} of {total} unwound");

    // The thread reading /dev/zero spends most of its time in the kernel, but should be sampled as often as the
    // spinner with those samples attributed to its call site.
    let (total, matching) = profile.count("zeros", |functions| {
        position(functions, "read_zeros").is_some()
    });
    assert!(total > 100, "{total} samples");
    assert!(matching * 10 > total * 9, "{matching} of {total} unwound");

    let (total, matching) = profile.count("late-spinner", unwound);
    assert!(total > 50, "{total} samples");
    assert!(matching * 10 > total * 9, "{matching} of {total} unwound");
}

#[test]
fn concurrent_recordings_conflict() {
    let _guard = TEST_LOCK.lock();
    let _in_progress = IN_PROGRESS.lock();

    let error = record(Duration::from_millis(100)).unwrap_err();
    let conjure_error::ErrorKind::Service(error) = error.kind() else {
        panic!("expected a service error");
    };
    assert_eq!(*error.error_code(), conjure_error::ErrorCode::Conflict);
}

#[test]
fn thread_churn() {
    let _guard = TEST_LOCK.lock();

    // Threads constantly starting and exiting exercise timer creation and deletion racing with thread exit.
    let stop = Arc::new(AtomicBool::new(false));
    let churners = (0..8)
        .map(|_| {
            let stop = stop.clone();
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let stop = stop.clone();
                    thread::Builder::new()
                        .name("churn".to_string())
                        .spawn(move || {
                            let start = Instant::now();
                            while !stop.load(Ordering::Relaxed)
                                && start.elapsed() < Duration::from_millis(50)
                            {
                                black_box(0);
                            }
                        })
                        .unwrap()
                        .join()
                        .unwrap();
                }
            })
        })
        .collect::<Vec<_>>();

    let result = record(Duration::from_secs(2));
    stop.store(true, Ordering::Relaxed);
    for churner in churners {
        churner.join().unwrap();
    }

    let profile = Decoded::new(&result.unwrap());
    let (total, _) = profile.count("churn", |_| true);
    assert!(total > 0);
}
