// Copyright 2021 Palantir Technologies, Inc.
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
use crate::logging::api::{metric_log_v1, MetricLogV1};
use crate::logging::logger::r#async::Closed;
use crate::logging::logger::{self, Appender};
use crate::logging::metric::gauge_reporter::GaugeReporter;
use crate::shutdown::ShutdownHooks;
use conjure_error::Error;
use conjure_object::Utc;
use futures_util::{ready, SinkExt, Stream};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::task;
use tokio::time::{self, Instant};
use witchcraft_log::warn;
use witchcraft_metrics::{Metric, MetricId, MetricRegistry};
use witchcraft_server_config::install::InstallConfig;

mod gauge_reporter;

const LOG_INTERVAL: Duration = Duration::from_secs(30);
const NANOS_PER_MICRO: i64 = 1_000;
const NANOS_PER_MICRO_F64: f64 = NANOS_PER_MICRO as f64;

pub async fn init(
    metrics: &Arc<MetricRegistry>,
    install: &InstallConfig,
    hooks: &mut ShutdownHooks,
) -> Result<(), Error> {
    let appender = logger::appender(install, metrics, hooks).await?;
    task::spawn(log_metrics(appender, metrics.clone()));

    Ok(())
}

/// Periodically records metric values.
///
/// Most of the implementation complexity here is to gracefully deal with very slow gauges. While gauges should always
/// be fast to compute, that isn't always the case and we don't want a poorly behaved gauge to negatively impact the
/// emission of other metrics.
///
/// To deal with this, gauge processing happens asynchronously from the main metric reporting loop. We track which
/// gauges are still processing to avoid running any twice at the same time, and spawn their computation off to separate
/// tasks. We collect and output the results of the gauges during the "idle" time when waiting for the next collection
/// interval. This makes the implementation a bit more complex but avoids having to have multiple owners of the
/// appender.
async fn log_metrics(mut appender: Appender<MetricLogV1>, metrics: Arc<MetricRegistry>) {
    let mut gauge_reporter = GaugeReporter::new();

    let mut next = Instant::now() + LOG_INTERVAL;

    loop {
        idle(&mut gauge_reporter, &mut appender, next).await;

        for (id, metric) in &metrics.metrics() {
            let builder = match metric {
                Metric::Counter(m) => builder(id)
                    .metric_type("counter")
                    .insert_values("count", m.count()),
                Metric::Meter(m) => builder(id)
                    .metric_type("meter")
                    .insert_values("count", m.count())
                    .insert_values("1m", m.one_minute_rate()),
                Metric::Gauge(m) => {
                    if !gauge_reporter.insert(id, m) {
                        warn!(
                            "Gauge is still executing from the last report cycle and will be skipped to avoid \
                             overloading the system",
                            safe: { gaugeName: metric_name(id) }
                        );
                    }
                    continue;
                }
                Metric::Histogram(m) => {
                    let snapshot = m.snapshot();
                    builder(id)
                        .metric_type("histogram")
                        .insert_values("max", snapshot.max())
                        .insert_values("p95", snapshot.value(0.95))
                        .insert_values("p99", snapshot.value(0.99))
                        .insert_values("p999", snapshot.value(0.999))
                        .insert_values("count", m.count())
                }
                Metric::Timer(m) => {
                    let snapshot = m.snapshot();
                    builder(id)
                        .metric_type("timer")
                        .insert_values("max", snapshot.max() / NANOS_PER_MICRO)
                        .insert_values("p95", snapshot.value(0.95) / NANOS_PER_MICRO_F64)
                        .insert_values("p99", snapshot.value(0.99) / NANOS_PER_MICRO_F64)
                        .insert_values("p999", snapshot.value(0.999) / NANOS_PER_MICRO_F64)
                        .insert_values("count", m.count())
                        .insert_values("1m", m.one_minute_rate())
                }
            };

            let metric = finish_log(id, builder);
            if let Err(Closed) = Pin::new(&mut appender).feed(metric).await {
                break;
            }
        }

        next += LOG_INTERVAL;
    }
}

fn metric_name(id: &MetricId) -> String {
    let mut name = id.name().to_string();

    if id.tags().iter().next().is_some() {
        name.push('[');
        let mut first = true;
        for (key, value) in id.tags() {
            if !first {
                name.push(',');
            }
            first = false;
            name.push_str(key);
            name.push(':');
            name.push_str(value);
        }
        name.push(']');
    }

    name
}

fn builder(id: &MetricId) -> metric_log_v1::BuilderStage3 {
    MetricLogV1::builder()
        .type_("metric.1")
        .time(Utc::now())
        .metric_name(id.name())
}

fn finish_log(id: &MetricId, builder: metric_log_v1::BuilderStage4) -> MetricLogV1 {
    builder
        .tags(
            id.tags()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        )
        .build()
}

async fn idle(
    gauge_reporter: &mut GaugeReporter,
    appender: &mut Appender<MetricLogV1>,
    timeout: Instant,
) {
    IdleFuture {
        gauge_reporter,
        appender,
        sleep: time::sleep_until(timeout),
    }
    .await
}

#[pin_project]
struct IdleFuture<'a> {
    gauge_reporter: &'a mut GaugeReporter,
    appender: &'a mut Appender<MetricLogV1>,
    #[pin]
    sleep: time::Sleep,
}

impl Future for IdleFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if this.sleep.poll(cx).is_ready() {
            return Poll::Ready(());
        }

        while !this.gauge_reporter.is_empty() {
            if let Err(Closed) = ready!(Pin::new(&mut *this.appender).poll_ready(cx)) {
                break;
            }

            let result = match ready!(Pin::new(&mut *this.gauge_reporter).poll_next(cx)) {
                Some(r) => r,
                None => break,
            };

            if let Ok(log) = result {
                if let Err(Closed) = Pin::new(&mut *this.appender).start_send(log) {
                    break;
                }
            }
        }

        Poll::Pending
    }
}
