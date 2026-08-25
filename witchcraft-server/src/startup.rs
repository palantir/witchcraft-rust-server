// Copyright 2026 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::logging::MetricLogger;
use crate::readiness::ReadinessCheckRegistry;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tokio::task::JoinHandle;
use tokio::{task, time};
use witchcraft_metrics::{MetricId, MetricRegistry};

const READINESS_POLL_INTERVAL: Duration = Duration::from_millis(100);
const STARTUP_TIME_METRIC: &str = "server.startup.time";
const STARTUP_FAILED_METRIC: &str = "server.startup.failed";

pub(crate) struct StartupMetrics {
    timer: StartupTimer,
    metric_logger: MetricLogger,
    failed: bool,
}

impl StartupMetrics {
    pub fn new(start: Instant, metrics: Arc<MetricRegistry>, metric_logger: MetricLogger) -> Self {
        StartupMetrics {
            timer: StartupTimer { start, metrics },
            metric_logger,
            failed: true,
        }
    }

    pub fn initialized(&self) {
        self.timer.record(Stage::Initialized);
    }

    pub fn endpoints_available(&self) {
        self.timer.record(Stage::EndpointsAvailable);
    }

    pub fn monitor_readiness(
        &self,
        handle: &Handle,
        readiness_checks: Arc<ReadinessCheckRegistry>,
    ) {
        drop(self.timer.monitor_readiness(handle, readiness_checks));
    }

    pub fn complete(mut self) {
        self.failed = false;
    }
}

impl Drop for StartupMetrics {
    fn drop(&mut self) {
        if self.failed {
            self.metric_logger
                .gauge(&MetricId::new(STARTUP_FAILED_METRIC), 1);
        }
    }
}

#[derive(Clone)]
struct StartupTimer {
    start: Instant,
    metrics: Arc<MetricRegistry>,
}

impl StartupTimer {
    fn record(&self, stage: Stage) {
        self.record_elapsed(stage, self.start.elapsed());
    }

    fn record_elapsed(&self, stage: Stage, elapsed: Duration) {
        let elapsed_micros = elapsed.as_micros() as u64;
        self.metrics.gauge(
            MetricId::new(STARTUP_TIME_METRIC).with_tag("stage", stage.tag()),
            move || elapsed_micros,
        );
    }

    fn monitor_readiness(
        &self,
        handle: &Handle,
        readiness_checks: Arc<ReadinessCheckRegistry>,
    ) -> JoinHandle<()> {
        let timer = self.clone();
        handle.spawn(async move {
            // Match Java Witchcraft's fixed-delay startup readiness polling interval.
            loop {
                let checks = task::spawn_blocking({
                    let readiness_checks = readiness_checks.clone();
                    move || readiness_checks.run_checks()
                })
                .await;

                if checks.is_ok_and(|checks| checks.values().all(|check| check.successful)) {
                    timer.record(Stage::Ready);
                    return;
                }

                time::sleep(READINESS_POLL_INTERVAL).await;
            }
        })
    }
}

#[derive(Copy, Clone)]
enum Stage {
    Initialized,
    EndpointsAvailable,
    Ready,
}

impl Stage {
    fn tag(self) -> &'static str {
        match self {
            Stage::Initialized => "initialized",
            Stage::EndpointsAvailable => "endpoints-available",
            Stage::Ready => "ready",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::api::objects::MetricLogV1;
    use crate::logging::Appender;
    use crate::readiness::{ReadinessCheck, ReadinessCheckResult};
    use crate::shutdown_hooks::ShutdownHooks;
    use futures_channel::mpsc::{self, UnboundedReceiver};
    use futures_util::StreamExt;
    use serde_json::json;
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use tokio::time::timeout;
    use witchcraft_metrics::Metric;

    #[test]
    fn records_fixed_elapsed_time_for_each_stage() {
        let metrics = Arc::new(MetricRegistry::new());
        let timer = StartupTimer {
            start: Instant::now(),
            metrics: metrics.clone(),
        };

        timer.record_elapsed(Stage::Initialized, Duration::from_micros(1));
        timer.record_elapsed(Stage::EndpointsAvailable, Duration::from_micros(2));
        timer.record_elapsed(Stage::Ready, Duration::from_micros(3));

        assert_eq!(metric_value(&metrics, Stage::Initialized), Some(json!(1)));
        assert_eq!(
            metric_value(&metrics, Stage::EndpointsAvailable),
            Some(json!(2))
        );
        assert_eq!(metric_value(&metrics, Stage::Ready), Some(json!(3)));
    }

    #[tokio::test]
    async fn records_ready_when_all_checks_succeed() {
        let metrics = Arc::new(MetricRegistry::new());
        let timer = StartupTimer {
            start: Instant::now(),
            metrics: metrics.clone(),
        };
        let readiness_checks = Arc::new(ReadinessCheckRegistry::new());
        let ready = Arc::new(AtomicBool::new(false));
        let runs = Arc::new(AtomicUsize::new(0));
        readiness_checks.register(TestReadinessCheck {
            ready: ready.clone(),
            runs: runs.clone(),
        });

        let monitor = timer.monitor_readiness(&Handle::current(), readiness_checks);
        timeout(Duration::from_secs(1), async {
            while runs.load(Ordering::Relaxed) == 0 {
                task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(metric_value(&metrics, Stage::Ready), None);

        ready.store(true, Ordering::Relaxed);
        timeout(Duration::from_secs(1), monitor)
            .await
            .unwrap()
            .unwrap();
        assert!(metric_value(&metrics, Stage::Ready).is_some());

        let runs_at_ready = runs.load(Ordering::Relaxed);
        time::sleep(READINESS_POLL_INTERVAL * 2).await;
        assert_eq!(runs.load(Ordering::Relaxed), runs_at_ready);
    }

    #[tokio::test]
    async fn records_ready_for_empty_registry() {
        let metrics = Arc::new(MetricRegistry::new());
        let timer = StartupTimer {
            start: Instant::now(),
            metrics: metrics.clone(),
        };

        let monitor =
            timer.monitor_readiness(&Handle::current(), Arc::new(ReadinessCheckRegistry::new()));

        timeout(Duration::from_secs(1), monitor)
            .await
            .unwrap()
            .unwrap();
        assert!(metric_value(&metrics, Stage::Ready).is_some());
    }

    #[tokio::test]
    async fn emits_failed_metric_when_startup_is_dropped() {
        let (logger, appender, mut receiver, hooks) = test_metric_logger();

        drop(StartupMetrics::new(
            Instant::now(),
            Arc::new(MetricRegistry::new()),
            logger,
        ));

        assert_failed_metric(receiver.next().await.unwrap());
        drop(appender);
        hooks.await;
    }

    #[tokio::test]
    async fn emits_failed_metric_during_unwind() {
        let (logger, appender, mut receiver, hooks) = test_metric_logger();

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let _startup =
                StartupMetrics::new(Instant::now(), Arc::new(MetricRegistry::new()), logger);
            panic!("startup failed");
        }));

        assert!(result.is_err());
        assert_failed_metric(receiver.next().await.unwrap());
        drop(appender);
        hooks.await;
    }

    #[tokio::test]
    async fn does_not_emit_failed_metric_after_completion() {
        let (logger, appender, mut receiver, hooks) = test_metric_logger();

        StartupMetrics::new(Instant::now(), Arc::new(MetricRegistry::new()), logger).complete();

        assert!(timeout(Duration::from_millis(100), receiver.next())
            .await
            .is_err());
        drop(appender);
        hooks.await;
    }

    fn metric_value(metrics: &MetricRegistry, stage: Stage) -> Option<serde_json::Value> {
        metrics.metrics().iter().find_map(|(id, metric)| {
            let stage_tag = id
                .tags()
                .iter()
                .find(|(key, _)| *key == "stage")
                .map(|(_, value)| value);
            if id.name() != STARTUP_TIME_METRIC || stage_tag != Some(stage.tag()) {
                return None;
            }

            match metric {
                Metric::Gauge(gauge) => Some(serde_json::to_value(gauge.value()).unwrap()),
                _ => panic!("startup time metric was not a gauge"),
            }
        })
    }

    fn test_metric_logger() -> (
        MetricLogger,
        Appender<MetricLogV1>,
        UnboundedReceiver<MetricLogV1>,
        ShutdownHooks,
    ) {
        let metrics = MetricRegistry::new();
        let mut hooks = ShutdownHooks::new();
        let (sender, receiver) = mpsc::unbounded();
        let appender = Appender::new(sender, &metrics, &mut hooks);
        let logger = MetricLogger::new(appender.handle());
        (logger, appender, receiver, hooks)
    }

    fn assert_failed_metric(log: MetricLogV1) {
        assert_eq!(log.metric_name(), STARTUP_FAILED_METRIC);
        assert_eq!(log.metric_type(), "gauge");
        assert_eq!(
            log.values()
                .get("value")
                .unwrap()
                .clone()
                .deserialize_into::<u64>()
                .unwrap(),
            1
        );
    }

    struct TestReadinessCheck {
        ready: Arc<AtomicBool>,
        runs: Arc<AtomicUsize>,
    }

    impl ReadinessCheck for TestReadinessCheck {
        fn type_(&self) -> &str {
            "TEST"
        }

        fn result(&self) -> ReadinessCheckResult {
            self.runs.fetch_add(1, Ordering::Relaxed);
            ReadinessCheckResult::builder()
                .successful(self.ready.load(Ordering::Relaxed))
                .build()
        }
    }
}
