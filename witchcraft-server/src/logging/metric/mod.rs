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
use crate::logging::api::objects::MetricLogV1;
use crate::logging::logger;
use crate::logging::logger::r#async::Closed;
use crate::logging::metric::gauge_reporter::GaugeReporter;
use crate::logging::metric::lazy_reporter::LazyMetricReporter;
use crate::shutdown_hooks::ShutdownHooks;
use conjure_error::Error;
use conjure_object::Utc;
use futures_sink::Sink;
use futures_util::{ready, SinkExt, StreamExt};
use std::collections::hash_map::DefaultHasher;
use std::future::{poll_fn, Future};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;
use tokio::task;
use tokio::time::{self, Instant};
use witchcraft_log::warn;
use witchcraft_log_util::metrics;
use witchcraft_metrics::{Metric, MetricId, MetricRegistry};
use witchcraft_server_config::install::InstallConfig;

mod gauge_reporter;
mod lazy_reporter;

const LOG_INTERVAL: Duration = Duration::from_secs(30);
const BUCKET_COUNT: usize = 30;

pub async fn init(
    metrics: &Arc<MetricRegistry>,
    install: &InstallConfig,
    hooks: &mut ShutdownHooks,
) -> Result<(), Error> {
    let appender = logger::appender(install, metrics, hooks).await?;
    task::spawn(run_metric_reporter(
        appender,
        metrics.clone(),
        rand::random(),
    ));

    Ok(())
}

/// Spreads each reporting cycle across stable metric buckets.
///
/// Membership is snapshotted at the start of the cycle; values are read when each bucket is due. Slow gauges run
/// separately, and their results are written while waiting for the next bucket.
async fn run_metric_reporter<S>(mut appender: S, metrics: Arc<MetricRegistry>, salt: u64)
where
    S: Sink<MetricLogV1, Error = Closed> + Unpin,
{
    let mut gauge_reporter = GaugeReporter::new();
    let mut lazy_reporter = LazyMetricReporter::default();
    let mut buckets: [Vec<(MetricId, Metric)>; BUCKET_COUNT] = std::array::from_fn(|_| Vec::new());
    let bucket_period = LOG_INTERVAL / BUCKET_COUNT as u32;
    let phase = Duration::from_nanos(salt % bucket_period.as_nanos() as u64);
    // Give service initialization time to register metrics before the first snapshot.
    let mut next_bucket_deadline = Instant::now() + LOG_INTERVAL + phase;

    for next_bucket in (0..BUCKET_COUNT).cycle() {
        if report_gauges_until(&mut gauge_reporter, &mut appender, next_bucket_deadline)
            .await
            .is_err()
        {
            return;
        }

        if next_bucket == 0 {
            let snapshot = metrics.metrics();
            lazy_reporter.start_cycle(&snapshot, Utc::now());
            for (id, metric) in &snapshot {
                buckets[bucket_index(id, salt)].push((id.clone(), metric.clone()));
            }
        }

        for (id, metric) in buckets[next_bucket].drain(..) {
            if let Metric::Gauge(m) = &metric {
                if !gauge_reporter.insert(&id, m) {
                    warn!(
                        "Gauge is still executing from the last report cycle and will be skipped to avoid \
                         overloading the system",
                        safe: { gaugeName: metrics::safe_metric_name_with_tags(&id) }
                    );
                }
                continue;
            }

            for log in lazy_reporter
                .prepare_logs(&id, &metric, Utc::now())
                .into_iter()
                .flatten()
            {
                if let Err(Closed) = appender.feed(log).await {
                    return;
                }
            }
        }

        // Keep the original phase, skipping elapsed ticks so delays cannot trigger catch-up bursts.
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(next_bucket_deadline);
        let remainder =
            Duration::from_nanos((elapsed.as_nanos() % bucket_period.as_nanos()) as u64);
        next_bucket_deadline = now + bucket_period - remainder;
    }
}

fn bucket_index(id: &MetricId, salt: u64) -> usize {
    let mut hasher = DefaultHasher::new();
    salt.hash(&mut hasher);
    id.hash(&mut hasher);
    (hasher.finish() % BUCKET_COUNT as u64) as usize
}

async fn report_gauges_until<S>(
    gauge_reporter: &mut GaugeReporter,
    appender: &mut S,
    deadline: Instant,
) -> Result<(), Closed>
where
    S: Sink<MetricLogV1, Error = Closed> + Unpin,
{
    let sleep = time::sleep_until(deadline);
    tokio::pin!(sleep);

    poll_fn(|cx| loop {
        // Check for shutdown even when there are no gauges or metrics in the current bucket.
        let appender_ready = appender.poll_ready_unpin(cx);
        if let Poll::Ready(Err(Closed)) = appender_ready {
            return Poll::Ready(Err(Closed));
        }

        if sleep.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Ok(()));
        }

        if gauge_reporter.is_empty() {
            return Poll::Pending;
        }
        ready!(appender_ready)?;

        let Some(result) = ready!(gauge_reporter.poll_next_unpin(cx)) else {
            return Poll::Pending;
        };

        if let Ok(log) = result {
            appender.start_send_unpin(log)?;
        }
    })
    .await
}

#[cfg(test)]
mod test {
    use super::*;
    use futures_util::poll;
    use parking_lot::Mutex;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Context, Waker};

    const SALT: u64 = 1_250_000_000;
    const BUCKET_PERIOD: Duration = Duration::from_secs(1);

    #[derive(Default)]
    struct State {
        logs: Vec<(Instant, MetricLogV1)>,
        blocked: bool,
        closed: bool,
        waker: Option<Waker>,
    }

    #[derive(Clone, Default)]
    struct TestAppender(Arc<Mutex<State>>);

    impl TestAppender {
        fn unblock(&self) {
            let mut state = self.0.lock();
            state.blocked = false;
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }

        fn close(&self) {
            let mut state = self.0.lock();
            state.closed = true;
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }

    impl Sink<MetricLogV1> for TestAppender {
        type Error = Closed;

        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Closed>> {
            let mut state = self.0.lock();
            if state.closed {
                Poll::Ready(Err(Closed))
            } else if state.blocked {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        }

        fn start_send(self: Pin<&mut Self>, log: MetricLogV1) -> Result<(), Closed> {
            let mut state = self.0.lock();
            if state.closed {
                return Err(Closed);
            }
            state.logs.push((Instant::now(), log));
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Closed>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Closed>> {
            TestAppender::close(self.get_mut());
            Poll::Ready(Ok(()))
        }
    }

    fn first_report_delay() -> Duration {
        LOG_INTERVAL + Duration::from_nanos(SALT % BUCKET_PERIOD.as_nanos() as u64)
    }

    fn id_in_bucket(name: &'static str, bucket: usize) -> MetricId {
        (0..)
            .map(|index| MetricId::new(name).with_tag("endpoint", index.to_string()))
            .find(|id| bucket_index(id, SALT) == bucket)
            .unwrap()
    }

    fn registry_with_every_bucket() -> Arc<MetricRegistry> {
        let registry = Arc::new(MetricRegistry::new());
        for bucket in 0..BUCKET_COUNT {
            registry.counter(id_in_bucket("server.response", bucket));
        }
        registry
    }

    #[tokio::test(start_paused = true)]
    async fn spreads_tagged_metrics_and_repeats_every_interval() {
        let appender = TestAppender::default();
        let start = Instant::now();
        let reporter = run_metric_reporter(appender.clone(), registry_with_every_bucket(), SALT);
        tokio::pin!(reporter);
        assert!(poll!(&mut reporter).is_pending());

        time::advance(first_report_delay() - Duration::from_millis(1)).await;
        assert!(poll!(&mut reporter).is_pending());
        assert!(appender.0.lock().logs.is_empty());
        time::advance(Duration::from_millis(1)).await;

        for bucket in 0..BUCKET_COUNT {
            if bucket != 0 {
                time::advance(BUCKET_PERIOD).await;
            }
            assert!(poll!(&mut reporter).is_pending());
            let state = appender.0.lock();
            assert_eq!(state.logs.len(), bucket + 1);
            let (time, log) = &state.logs[bucket];
            assert_eq!(
                *time,
                start + first_report_delay() + BUCKET_PERIOD * bucket as u32
            );
            let id = id_in_bucket("server.response", bucket);
            assert_eq!(
                log.tags().get("endpoint").map(String::as_str),
                Some(id.tags().iter().next().unwrap().1)
            );
            assert_eq!(serde_json::to_value(log.values()).unwrap()["count"], 0);
        }

        for bucket in 0..BUCKET_COUNT {
            time::advance(BUCKET_PERIOD).await;
            assert!(poll!(&mut reporter).is_pending());
            let state = appender.0.lock();
            assert_eq!(state.logs.len(), BUCKET_COUNT + bucket + 1);
            let (first_time, first_log) = &state.logs[bucket];
            let (second_time, second_log) = &state.logs[BUCKET_COUNT + bucket];
            assert_eq!(*second_time - *first_time, LOG_INTERVAL);
            assert_eq!(second_log.tags(), first_log.tags());
        }
    }

    #[tokio::test(start_paused = true)]
    async fn snapshots_membership_but_reads_values_at_emission() {
        let registry = Arc::new(MetricRegistry::new());
        let removed_id = id_in_bucket("removed", 10);
        let removed = registry.counter(removed_id.clone());
        let added_id = id_in_bucket("added", 20);
        let appender = TestAppender::default();
        let reporter = run_metric_reporter(appender.clone(), registry.clone(), SALT);
        tokio::pin!(reporter);
        assert!(poll!(&mut reporter).is_pending());
        time::advance(first_report_delay()).await;
        assert!(poll!(&mut reporter).is_pending());

        removed.add(7);
        registry.remove(removed_id);
        registry.counter(added_id).add(9);
        for _ in 1..BUCKET_COUNT {
            time::advance(BUCKET_PERIOD).await;
            assert!(poll!(&mut reporter).is_pending());
        }
        {
            let state = appender.0.lock();
            assert_eq!(state.logs.len(), 1);
            assert_eq!(state.logs[0].1.metric_name(), "removed");
            assert_eq!(
                serde_json::to_value(state.logs[0].1.values()).unwrap()["count"],
                7
            );
        }

        for _ in 0..=20 {
            time::advance(BUCKET_PERIOD).await;
            assert!(poll!(&mut reporter).is_pending());
        }
        let state = appender.0.lock();
        assert_eq!(state.logs.len(), 2);
        assert_eq!(state.logs[1].1.metric_name(), "added");
        assert_eq!(
            serde_json::to_value(state.logs[1].1.values()).unwrap()["count"],
            9
        );
    }

    #[tokio::test(start_paused = true)]
    async fn runtime_stalls_do_not_cause_catch_up_bursts() {
        let appender = TestAppender::default();
        let reporter = run_metric_reporter(appender.clone(), registry_with_every_bucket(), SALT);
        tokio::pin!(reporter);
        assert!(poll!(&mut reporter).is_pending());
        time::advance(first_report_delay()).await;
        assert!(poll!(&mut reporter).is_pending());
        assert_eq!(appender.0.lock().logs.len(), 1);

        time::advance(Duration::from_secs(300)).await;
        assert!(poll!(&mut reporter).is_pending());
        assert_eq!(appender.0.lock().logs.len(), 2);
        time::advance(BUCKET_PERIOD).await;
        assert!(poll!(&mut reporter).is_pending());
        assert_eq!(appender.0.lock().logs.len(), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn appender_backpressure_does_not_cause_catch_up_bursts() {
        let appender = TestAppender::default();
        let reporter = run_metric_reporter(appender.clone(), registry_with_every_bucket(), SALT);
        tokio::pin!(reporter);
        assert!(poll!(&mut reporter).is_pending());
        time::advance(first_report_delay()).await;
        assert!(poll!(&mut reporter).is_pending());
        appender.0.lock().blocked = true;
        time::advance(BUCKET_PERIOD).await;
        assert!(poll!(&mut reporter).is_pending());
        assert_eq!(appender.0.lock().logs.len(), 1);

        time::advance(Duration::from_secs(300)).await;
        appender.unblock();
        assert!(poll!(&mut reporter).is_pending());
        assert_eq!(appender.0.lock().logs.len(), 2);
        time::advance(BUCKET_PERIOD).await;
        assert!(poll!(&mut reporter).is_pending());
        assert_eq!(appender.0.lock().logs.len(), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn closed_appender_stops_reporting() {
        let appender = TestAppender::default();
        appender.close();
        let reporter = run_metric_reporter(appender, Arc::new(MetricRegistry::new()), SALT);
        tokio::pin!(reporter);
        assert!(poll!(&mut reporter).is_ready());

        let appender = TestAppender::default();
        appender.0.lock().blocked = true;
        let reporter = run_metric_reporter(appender.clone(), registry_with_every_bucket(), SALT);
        tokio::pin!(reporter);
        assert!(poll!(&mut reporter).is_pending());
        time::advance(first_report_delay()).await;
        assert!(poll!(&mut reporter).is_pending());
        appender.close();
        assert!(poll!(&mut reporter).is_ready());
    }

    #[tokio::test(start_paused = true)]
    async fn slow_gauges_do_not_block_buckets_or_run_twice() {
        let registry = Arc::new(MetricRegistry::new());
        let evaluations = Arc::new(AtomicUsize::new(0));
        let (release, released) = std::sync::mpsc::channel::<()>();
        let released = Mutex::new(released);
        let (started, running) = tokio::sync::oneshot::channel();
        let started = Mutex::new(Some(started));
        registry.gauge(id_in_bucket("slow.gauge", 0), {
            let evaluations = evaluations.clone();
            move || {
                evaluations.fetch_add(1, Ordering::SeqCst);
                if let Some(started) = started.lock().take() {
                    let _ = started.send(());
                }
                let _ = released.lock().recv();
                1
            }
        });
        registry.counter(id_in_bucket("counter", 1));

        let appender = TestAppender::default();
        let reporter = run_metric_reporter(appender.clone(), registry, SALT);
        tokio::pin!(reporter);
        assert!(poll!(&mut reporter).is_pending());
        time::advance(first_report_delay()).await;
        assert!(poll!(&mut reporter).is_pending());
        running.await.unwrap();

        time::advance(BUCKET_PERIOD).await;
        assert!(poll!(&mut reporter).is_pending());
        assert!(appender
            .0
            .lock()
            .logs
            .iter()
            .any(|(_, log)| log.metric_name() == "counter"));
        for _ in 1..BUCKET_COUNT {
            time::advance(BUCKET_PERIOD).await;
            assert!(poll!(&mut reporter).is_pending());
        }
        assert_eq!(evaluations.load(Ordering::SeqCst), 1);
        release.send(()).unwrap();
    }
}
