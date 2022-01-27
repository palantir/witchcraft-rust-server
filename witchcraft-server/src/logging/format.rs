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
use crate::logging::api::{LogLevel, MetricLogV1, RequestLogV2, ServiceLogV1, TraceLogV1};
use std::marker::PhantomData;
use std::sync::Arc;
use witchcraft_metrics::{Meter, MetricId, MetricRegistry};

pub trait LogFormat: Sized {
    const TYPE: &'static str;
    const FILE_STEM: &'static str;
    const SIZE_LIMIT_GB: u32;
    const TIME_LIMIT_DAYS: u32;

    type Reporter: ReportLog<Self>;
}

pub trait ReportLog<T> {
    fn new(metrics: &MetricRegistry) -> Self;

    fn report(&self, log: &T);
}

impl LogFormat for MetricLogV1 {
    const TYPE: &'static str = "metric.1";
    const FILE_STEM: &'static str = "metric";
    const SIZE_LIMIT_GB: u32 = 1;
    const TIME_LIMIT_DAYS: u32 = 5;

    type Reporter = StandardReporter<Self>;
}

impl LogFormat for RequestLogV2 {
    const TYPE: &'static str = "request.2";
    const FILE_STEM: &'static str = "request";
    const SIZE_LIMIT_GB: u32 = 5;
    const TIME_LIMIT_DAYS: u32 = 30;

    type Reporter = StandardReporter<Self>;
}

impl LogFormat for ServiceLogV1 {
    const TYPE: &'static str = "service.1";
    const FILE_STEM: &'static str = "service";
    const SIZE_LIMIT_GB: u32 = 5;
    const TIME_LIMIT_DAYS: u32 = 30;

    type Reporter = ServiceLogReporter;
}

impl LogFormat for TraceLogV1 {
    const TYPE: &'static str = "trace.1";
    const FILE_STEM: &'static str = "trace";
    const SIZE_LIMIT_GB: u32 = 1;
    const TIME_LIMIT_DAYS: u32 = 5;

    type Reporter = StandardReporter<Self>;
}

pub struct ServiceLogReporter {
    fatal_rate: Arc<Meter>,
    error_rate: Arc<Meter>,
    warn_rate: Arc<Meter>,
    info_rate: Arc<Meter>,
    debug_rate: Arc<Meter>,
    trace_rate: Arc<Meter>,
}

impl ReportLog<ServiceLogV1> for ServiceLogReporter {
    fn new(metrics: &MetricRegistry) -> Self {
        let meter = |level| {
            metrics.meter(
                MetricId::new("logging.sls")
                    .with_tag("type", ServiceLogV1::TYPE)
                    .with_tag("level", level),
            )
        };

        ServiceLogReporter {
            fatal_rate: meter("fatal"),
            error_rate: meter("error"),
            warn_rate: meter("warn"),
            info_rate: meter("info"),
            debug_rate: meter("debug"),
            trace_rate: meter("trace"),
        }
    }

    fn report(&self, log: &ServiceLogV1) {
        let meter = match log.level() {
            LogLevel::Fatal => &self.fatal_rate,
            LogLevel::Error => &self.error_rate,
            LogLevel::Warn => &self.warn_rate,
            LogLevel::Info => &self.info_rate,
            LogLevel::Debug => &self.debug_rate,
            LogLevel::Trace => &self.trace_rate,
        };

        meter.mark(1);
    }
}

pub struct StandardReporter<T> {
    rate: Arc<Meter>,
    _p: PhantomData<T>,
}

impl<T> ReportLog<T> for StandardReporter<T>
where
    T: LogFormat,
{
    fn new(metrics: &MetricRegistry) -> Self {
        StandardReporter {
            rate: metrics.meter(MetricId::new("logging.sls").with_tag("type", T::TYPE)),
            _p: PhantomData,
        }
    }

    fn report(&self, _: &T) {
        self.rate.mark(1);
    }
}
