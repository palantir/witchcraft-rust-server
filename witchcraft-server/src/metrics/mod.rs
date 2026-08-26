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
#[cfg(unix)]
use crate::metrics::rusage::Rusage;
use std::panic;
#[cfg(unix)]
use std::thread;
use std::time::Instant;
use witchcraft_metrics::MetricRegistry;

#[cfg(all(feature = "jemalloc", not(windows)))]
mod jemalloc;
#[cfg(target_os = "linux")]
mod proc;
#[cfg(unix)]
mod rusage;

pub fn init(metrics: &MetricRegistry) {
    register_uptime_metric(metrics);
    register_panic_metric(metrics);
    #[cfg(unix)]
    register_rusage_metrics(metrics);
    #[cfg(target_os = "linux")]
    proc::register_metrics(metrics);
    #[cfg(all(feature = "jemalloc", not(windows)))]
    jemalloc::register_metrics(metrics);
}

fn register_uptime_metric(metrics: &MetricRegistry) {
    let start = Instant::now();
    metrics.gauge("process.uptime", move || start.elapsed().as_micros() as u64);
}

fn register_panic_metric(metrics: &MetricRegistry) {
    let panics = metrics.counter("process.panics");
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        panics.inc();
        hook(info)
    }));
}

#[cfg(unix)]
fn register_rusage_metrics(metrics: &MetricRegistry) {
    metrics.gauge("process.user-time", || {
        Rusage::get_self().map_or(0, |r| r.user_time().as_micros() as u64)
    });
    metrics.gauge("process.user-time.norm", || {
        Rusage::get_self().map_or(0, |r| {
            r.user_time().as_micros() as u64
                / thread::available_parallelism().map_or(1, |v| v.get()) as u64
        })
    });
    metrics.gauge("process.system-time", || {
        Rusage::get_self().map_or(0, |r| r.system_time().as_micros() as u64)
    });
    metrics.gauge("process.system-time.norm", || {
        Rusage::get_self().map_or(0, |r| {
            r.system_time().as_micros() as u64
                / thread::available_parallelism().map_or(1, |v| v.get()) as u64
        })
    });
    metrics.gauge("process.blocks-read", || {
        Rusage::get_self().map_or(0, |r| r.blocks_read())
    });
    metrics.gauge("process.blocks-written", || {
        Rusage::get_self().map_or(0, |r| r.blocks_written())
    });
}
