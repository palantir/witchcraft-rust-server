use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use parking_lot::Mutex;
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
use tikv_jemalloc_ctl::{epoch, stats};
use witchcraft_metrics::MetricRegistry;

pub fn register_metrics(metrics: &MetricRegistry) {
    let advance = Arc::new(Mutex::new(Debounced::new(|| {
        let _ = epoch::advance();
    })));

    let advance_for_heap = advance.clone();
    metrics.gauge("process.heap", move || {
        advance_for_heap.lock().call_debounced();
        stats::allocated::read().unwrap_or(0)
    });

    let advance_for_active = advance.clone();
    metrics.gauge("process.heap.active", move || {
        advance_for_active.lock().call_debounced();
        stats::active::read().unwrap_or(0)
    });
    metrics.gauge("process.heap.resident", move || {
        advance.lock().call_debounced();
        stats::resident::read().unwrap_or(0)
    });
}

struct Debounced<F>
where
    F: FnMut(),
{
    function: F,
    loaded: Instant,
}

impl<F> Debounced<F>
where
    F: FnMut(),
{
    fn new(function: F) -> Debounced<F> {
        Debounced {
            function,
            loaded: Instant::now(),
        }
    }

    fn call_debounced(&mut self) {
        let now = Instant::now();
        if now - self.loaded > Duration::from_secs(1) {
            (self.function)();
            self.loaded = now;
        }
    }
}
