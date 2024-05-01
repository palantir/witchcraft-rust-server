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
    metrics.gauge("process.heap", move || {
        let _ = epoch::advance();
        stats::allocated::read().unwrap_or(0)
    });
    metrics.gauge("process.heap.active", move || {
        let _ = epoch::advance();
        stats::active::read().unwrap_or(0)
    });
    metrics.gauge("process.heap.resident", move || {
        let _ = epoch::advance();
        stats::resident::read().unwrap_or(0)
    });
}
