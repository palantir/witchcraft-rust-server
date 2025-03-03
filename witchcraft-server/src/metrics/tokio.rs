// Copyright 2025 Palantir Technologies, Inc.
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

use tokio::runtime::Handle;
use witchcraft_metrics::MetricRegistry;

pub fn register_metrics(metrics: &MetricRegistry, handle: &Handle) {
    let tokio_metrics = handle.metrics();

    #[cfg(all(tokio_unstable, feature = "tokio_unstable"))]
    unstable::register(metrics, &tokio_metrics);

    metrics.gauge("tokio.tasks", move || tokio_metrics.num_alive_tasks());
}

#[cfg(all(tokio_unstable, feature = "tokio_unstable"))]
mod unstable {
    use tokio::runtime::RuntimeMetrics;
    use witchcraft_metrics::{MetricId, MetricRegistry};

    pub fn register(metrics: &MetricRegistry, tokio_metrics: &RuntimeMetrics) {
        metrics.gauge("tokio.blocking.threads", {
            let tokio_metrics = tokio_metrics.clone();
            move || tokio_metrics.num_blocking_threads()
        });

        metrics.gauge("tokio.blocking.threads.idle", {
            let tokio_metrics = tokio_metrics.clone();
            move || tokio_metrics.num_idle_blocking_threads()
        });

        metrics.gauge("tokio.tasks.polls", {
            let tokio_metrics = tokio_metrics.clone();
            move || {
                (0..tokio_metrics.num_workers())
                    .map(|worker| tokio_metrics.worker_poll_count(worker))
                    .sum::<u64>()
            }
        });

        for bucket in 0..tokio_metrics.poll_time_histogram_num_buckets() {
            let range = tokio_metrics.poll_time_histogram_bucket_range(bucket);
            metrics.gauge(
                MetricId::new("tokio.tasks.poll-duration-bucket")
                    .with_tag("ge", range.start.as_micros().to_string())
                    .with_tag("lt", range.end.as_micros().to_string()),
                {
                    let tokio_metrics = tokio_metrics.clone();
                    move || {
                        (0..tokio_metrics.num_workers())
                            .map(|worker| {
                                tokio_metrics.poll_time_histogram_bucket_count(worker, bucket)
                            })
                            .sum::<u64>()
                    }
                },
            );
        }
    }
}
