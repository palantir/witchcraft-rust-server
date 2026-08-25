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
use crate::logging::api::objects::{metric_log_v1, MetricLogV1};
use crate::logging::logger::r#async::{AppenderHandle, Closed};
use crate::logging::logger::{self, Appender};
use crate::logging::metric::gauge_reporter::GaugeReporter;
use crate::shutdown_hooks::ShutdownHooks;
use conjure_error::Error;
use conjure_object::Utc;
use futures_sink::Sink;
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

#[derive(Clone)]
pub(crate) struct MetricLogger {
    appender: AppenderHandle<MetricLogV1>,
}

impl MetricLogger {
    pub(crate) fn new(appender: AppenderHandle<MetricLogV1>) -> Self {
        MetricLogger { appender }
    }

    pub fn gauge(&self, id: &MetricId, value: u64) {
        // Startup failures can terminate the process before the first periodic metric report, so they must be queued
        // directly.
        let metric = finish_log(
            id,
            builder(id)
                .metric_type("gauge")
                .insert_values("value", value),
        );
        let _ = self.appender.try_send(metric);
    }
}

pub async fn init(
    metrics: &Arc<MetricRegistry>,
    install: &InstallConfig,
    hooks: &mut ShutdownHooks,
) -> Result<MetricLogger, Error> {
    let appender = logger::appender(install, metrics, hooks).await?;
    let metric_logger = MetricLogger::new(appender.handle());
    task::spawn(log_metrics(appender, metrics.clone()));

    Ok(metric_logger)
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

fn builder(id: &MetricId) -> metric_log_v1::Builder<metric_log_v1::MetricTypeStage> {
    MetricLogV1::builder()
        .type_("metric.1")
        .time(Utc::now())
        .metric_name(id.name())
}

fn finish_log(
    id: &MetricId,
    builder: metric_log_v1::Builder<metric_log_v1::Complete>,
) -> MetricLogV1 {
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
            if let Err(Closed) = ready!(Pin::new(&mut **this.appender).poll_ready(cx)) {
                break;
            }

            let result = match ready!(Pin::new(&mut *this.gauge_reporter).poll_next(cx)) {
                Some(r) => r,
                None => break,
            };

            if let Ok(log) = result {
                if let Err(Closed) = Pin::new(&mut **this.appender).start_send(log) {
                    break;
                }
            }
        }

        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::logger::r#async::AsyncAppender;
    use futures_channel::mpsc;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn sends_gauge_immediately() {
        let metrics = MetricRegistry::new();
        let mut hooks = ShutdownHooks::new();
        let (sender, mut receiver) = mpsc::unbounded();
        let appender = AsyncAppender::new(sender, &metrics, &mut hooks);
        let logger = MetricLogger::new(appender.handle());

        logger.gauge(&MetricId::new("test.gauge").with_tag("tag", "value"), 42);

        let log = receiver.next().await.unwrap();
        assert_eq!(log.metric_name(), "test.gauge");
        assert_eq!(log.metric_type(), "gauge");
        assert_eq!(log.tags().get("tag").map(String::as_str), Some("value"));
        assert_eq!(
            log.values()
                .get("value")
                .unwrap()
                .clone()
                .deserialize_into::<u64>()
                .unwrap(),
            42
        );

        drop(logger);
        drop(appender);
        hooks.await;
    }
}
