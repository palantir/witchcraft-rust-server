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
use super::LOG_INTERVAL;
use crate::logging::api::objects::MetricLogV1;
use conjure_object::{DateTime, Utc};
use std::collections::HashSet;
use witchcraft_log_util::metrics;
use witchcraft_metrics::{Metric, MetricId, Metrics};

/// Skips zero-count meters, histograms, and timers and adds a baseline on their first nonzero report.
#[derive(Default)]
pub(super) struct LazyMetricReporter {
    first_report_time: Option<DateTime<Utc>>,
    reported: HashSet<MetricId>,
}

impl LazyMetricReporter {
    pub(super) fn start_cycle(&mut self, snapshot: &Metrics, now: DateTime<Utc>) {
        self.first_report_time.get_or_insert(now);

        // Retain IDs across same-ID replacements so a baseline cannot overwrite a previous real sample.
        let mut reported = std::mem::take(&mut self.reported);
        self.reported = snapshot
            .iter()
            .filter(|(_, metric)| metrics::supports_lazy_reporting(metric))
            .filter_map(|(id, _)| reported.take(id))
            .collect();
    }

    pub(super) fn prepare_logs(
        &mut self,
        id: &MetricId,
        metric: &Metric,
        now: DateTime<Utc>,
    ) -> [Option<MetricLogV1>; 2] {
        if metrics::should_skip_reporting(metric) {
            return [None, None];
        }

        let mut baseline = None;
        if metrics::supports_lazy_reporting(metric) && !self.reported.contains(id) {
            self.reported.insert(id.clone());
            let baseline_time = now - LOG_INTERVAL;
            if self
                .first_report_time
                .is_some_and(|first_report_time| baseline_time >= first_report_time)
            {
                baseline = metrics::zero_baseline_log(id, metric, baseline_time);
            }
        }

        [baseline, Some(metrics::from_metric(id, metric))]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use witchcraft_metrics::MetricRegistry;

    fn time(seconds: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000 + seconds, 0).unwrap()
    }

    fn values(log: &MetricLogV1) -> serde_json::Value {
        serde_json::to_value(log.values()).unwrap()
    }

    #[test]
    fn startup_does_not_backdate_before_reporting_began() {
        let registry = MetricRegistry::new();
        registry.meter("meter").mark(1);
        let snapshot = registry.metrics();
        let (id, metric) = snapshot.iter().next().unwrap();
        let mut reporter = LazyMetricReporter::default();
        reporter.start_cycle(&snapshot, time(0));

        assert!(matches!(
            reporter.prepare_logs(id, metric, time(15)),
            [None, Some(_)]
        ));

        // Skipping the baseline at startup still initializes the metric, so one is never added later.
        reporter.start_cycle(&snapshot, time(30));
        assert!(matches!(
            reporter.prepare_logs(id, metric, time(45)),
            [None, Some(_)]
        ));
    }

    #[test]
    fn first_use_emits_one_baseline_per_series() {
        let registry = MetricRegistry::new();
        let timers = ["getFoo", "getBar"].map(|endpoint| {
            registry.timer(MetricId::new("server.response").with_tag("endpoint", endpoint))
        });
        let snapshot = registry.metrics();
        let mut reporter = LazyMetricReporter::default();
        reporter.start_cycle(&snapshot, time(0));
        for (id, metric) in &snapshot {
            assert_eq!(reporter.prepare_logs(id, metric, time(15)), [None, None]);
        }

        // A zero-duration observation is still a real observation.
        for timer in timers {
            timer.update(Duration::ZERO);
        }
        reporter.start_cycle(&snapshot, time(30));
        for (id, metric) in &snapshot {
            let [Some(baseline), Some(current)] = reporter.prepare_logs(id, metric, time(45))
            else {
                panic!("expected a baseline and a current metric");
            };
            assert_eq!(baseline.time(), time(15));
            assert_eq!(baseline.tags(), current.tags());
            assert_eq!(values(&baseline)["count"], json!(0));
            assert_eq!(values(&current)["count"], json!(1));
        }

        reporter.start_cycle(&snapshot, time(60));
        for (id, metric) in &snapshot {
            let [None, Some(current)] = reporter.prepare_logs(id, metric, time(75)) else {
                panic!("expected the used metric to keep reporting without another baseline");
            };
            assert_eq!(values(&current)["count"], json!(1));
        }
    }

    #[test]
    fn absence_from_a_cycle_snapshot_resets_baseline() {
        let registry = MetricRegistry::new();
        registry.meter("meter").mark(1);
        let snapshot = registry.metrics();
        let (id, metric) = snapshot.iter().next().unwrap();
        let mut reporter = LazyMetricReporter::default();
        reporter.start_cycle(&snapshot, time(0));
        reporter.prepare_logs(id, metric, time(0));

        registry.remove("meter");
        reporter.start_cycle(&registry.metrics(), time(30));

        registry.meter("meter").mark(1);
        let snapshot = registry.metrics();
        let (id, metric) = snapshot.iter().next().unwrap();
        reporter.start_cycle(&snapshot, time(60));
        let [Some(baseline), Some(current)] = reporter.prepare_logs(id, metric, time(60)) else {
            panic!("expected a fresh baseline after an absent cycle");
        };
        assert_eq!(baseline.time(), time(30));
        assert_eq!(values(&current)["count"], json!(1));
    }

    #[test]
    fn replacement_with_the_same_id_does_not_overwrite_previous_samples() {
        let registry = MetricRegistry::new();
        registry.meter("meter").mark(5);
        let mut reporter = LazyMetricReporter::default();
        let snapshot = registry.metrics();
        reporter.start_cycle(&snapshot, time(0));
        for (id, metric) in &snapshot {
            reporter.prepare_logs(id, metric, time(0));
        }

        registry.remove("meter");
        registry.meter("meter").mark(1);
        let snapshot = registry.metrics();
        let (id, metric) = snapshot.iter().next().unwrap();
        reporter.start_cycle(&snapshot, time(30));
        let [None, Some(current)] = reporter.prepare_logs(id, metric, time(30)) else {
            panic!("a second baseline would overwrite the previous real sample at time(0)");
        };
        assert_eq!(values(&current)["count"], json!(1));
    }

    #[test]
    fn signed_meter_uses_the_exact_zero_convention() {
        let registry = MetricRegistry::new();
        let meter = registry.meter("meter");
        let snapshot = registry.metrics();
        let (id, metric) = snapshot.iter().next().unwrap();
        let mut reporter = LazyMetricReporter::default();
        reporter.start_cycle(&snapshot, time(0));

        meter.mark(-1);
        let [Some(_), Some(current)] = reporter.prepare_logs(id, metric, time(30)) else {
            panic!("negative counts must be reported");
        };
        assert_eq!(values(&current)["count"], json!(-1));

        meter.mark(1);
        assert_eq!(reporter.prepare_logs(id, metric, time(60)), [None, None]);

        meter.mark(1);
        assert!(matches!(
            reporter.prepare_logs(id, metric, time(90)),
            [None, Some(_)]
        ));
    }
}
