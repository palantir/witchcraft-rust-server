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
use crate::logging::api::{Annotation, Endpoint, Span, TraceLogV1};
use crate::logging::logger::{self, Appender, Payload};
use crate::shutdown_hooks::ShutdownHooks;
use conjure_error::Error;
use conjure_object::{SafeLong, Utc};
use refreshable::Refreshable;
use std::time::{Duration, SystemTime};
use witchcraft_metrics::MetricRegistry;
use witchcraft_server_config::install::InstallConfig;
use witchcraft_server_config::runtime::LoggingConfig;
use zipkin::{Kind, Report, Sample, TraceId};

mod ifaddrs;

pub async fn init(
    metrics: &MetricRegistry,
    install: &InstallConfig,
    runtime: &Refreshable<LoggingConfig, Error>,
    hooks: &mut ShutdownHooks,
) -> Result<(), Error> {
    let appender = logger::appender(install, metrics, hooks).await?;
    let sampler = WitchcraftSampler {
        trace_rate: runtime.map(|c| c.trace_rate()),
    };
    let reporter = WitchcraftReporter { appender };

    let mut local_endpoint = zipkin::Endpoint::builder();
    local_endpoint
        .service_name(install.product_name())
        .port(install.port());

    if let Some(ip) = ifaddrs::get_ip() {
        local_endpoint.ip(ip);
    }

    zipkin::set_tracer(sampler, reporter, local_endpoint.build())
        .expect("tracer already initialized");

    Ok(())
}

struct WitchcraftReporter {
    appender: Appender<TraceLogV1>,
}

impl Report for WitchcraftReporter {
    fn report(&self, raw_span: zipkin::Span) {
        let raw_timestamp = raw_span.timestamp().expect("BUG: span missing timestamp");
        let timestamp = time2micros(raw_timestamp);

        let raw_duration = raw_span.duration().expect("BUG: span missing duration");
        let duration = dur2micros(raw_duration);

        let mut span = Span::builder()
            .trace_id(raw_span.trace_id().to_string())
            .id(raw_span.id().to_string())
            .name(raw_span.name().unwrap_or("unknown"))
            .timestamp(timestamp)
            .duration(duration)
            .parent_id(raw_span.parent_id().map(|s| s.to_string()));

        let raw_endpoint = raw_span
            .local_endpoint()
            .expect("BUG: local endpoint missing");
        let endpoint = Endpoint::builder()
            .service_name(raw_endpoint.service_name().unwrap_or("unknown"))
            .ipv4(raw_endpoint.ipv4().map(|v| v.to_string()))
            .ipv6(raw_endpoint.ipv6().map(|v| v.to_string()))
            .build();

        match raw_span.kind() {
            Some(Kind::Client) => {
                span = span
                    .push_annotations(Annotation::new(timestamp, "cs", endpoint.clone()))
                    .push_annotations(Annotation::new(
                        time2micros(raw_timestamp + raw_duration),
                        "cr",
                        endpoint.clone(),
                    ));
            }
            Some(Kind::Server) => {
                span = span
                    .push_annotations(Annotation::new(timestamp, "sr", endpoint.clone()))
                    .push_annotations(Annotation::new(
                        time2micros(raw_timestamp + raw_duration),
                        "ss",
                        endpoint.clone(),
                    ));
            }
            Some(Kind::Producer) => {
                span = span.push_annotations(Annotation::new(timestamp, "ms", endpoint.clone()));
            }
            Some(Kind::Consumer) => {
                span = span.push_annotations(Annotation::new(timestamp, "mr", endpoint.clone()));
            }
            Some(_) => {}
            None => {
                span = span.push_annotations(Annotation::new(timestamp, "lc", endpoint.clone()));
            }
        }

        span = span.extend_annotations(
            raw_span
                .annotations()
                .iter()
                .map(|a| Annotation::new(time2micros(a.timestamp()), a.value(), endpoint.clone())),
        );

        span = span.extend_tags(
            raw_span
                .tags()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string())),
        );

        let _ = self.appender.try_send(Payload {
            value: TraceLogV1::builder()
                .type_("trace.1")
                .time(Utc::now())
                .span(span.build())
                .build(),
            cb: None,
        });
    }
}

fn time2micros(t: SystemTime) -> SafeLong {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map_or(SafeLong::default(), dur2micros)
}

fn dur2micros(d: Duration) -> SafeLong {
    SafeLong::try_from(d.as_micros()).ok().unwrap_or_default()
}

struct WitchcraftSampler {
    trace_rate: Refreshable<f32, Error>,
}

impl Sample for WitchcraftSampler {
    fn sample(&self, _: TraceId) -> bool {
        rand::random::<f32>() < *self.trace_rate.get()
    }
}
