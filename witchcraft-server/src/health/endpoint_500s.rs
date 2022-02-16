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
use crate::endpoint::WitchcraftEndpoint;
use crate::health::{HealthCheck, HealthCheckResult, HealthState};
use conjure_object::Any;
use http::StatusCode;
use parking_lot::Mutex;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

const GRACE_PERIOD: Duration = Duration::from_secs(2 * 60);
const EVALUATION_PERIOD: Duration = Duration::from_secs(10 * 60);
const BROKEN_THRESHOLD: usize = 50;

enum Unhealthy {
    Healthy,
    Errors { first: Instant, last: Instant },
}

impl Unhealthy {
    fn tick(&mut self, now: Instant) {
        if let Unhealthy::Errors { last, .. } = *self {
            if now > last + GRACE_PERIOD {
                *self = Unhealthy::Healthy;
            }
        }
    }

    fn error(&mut self, now: Instant) {
        self.tick(now);

        match self {
            Unhealthy::Healthy => {
                *self = Unhealthy::Errors {
                    first: now,
                    last: now,
                };
            }
            Unhealthy::Errors { last, .. } => *last = now,
        }
    }

    fn check(&mut self, now: Instant) -> bool {
        self.tick(now);

        match self {
            Unhealthy::Healthy => false,
            Unhealthy::Errors { first, .. } => now > *first + EVALUATION_PERIOD,
        }
    }
}

#[allow(clippy::enum_variant_names)]
enum Broken {
    Healthy,
    Errors { count: usize },
    Broken,
}

impl Broken {
    fn error(&mut self) {
        match self {
            Broken::Healthy | Broken::Broken => {}
            Broken::Errors {
                count: BROKEN_THRESHOLD,
            } => *self = Broken::Broken,
            Broken::Errors { count } => *count += 1,
        }
    }

    fn success(&mut self) {
        match self {
            Broken::Healthy | Broken::Broken => {}
            Broken::Errors { .. } => *self = Broken::Healthy,
        }
    }

    fn check(&self) -> bool {
        match self {
            Broken::Healthy | Broken::Errors { .. } => false,
            Broken::Broken => true,
        }
    }
}

struct State {
    unhealthy: Unhealthy,
    broken: Broken,
}

pub struct EndpointHealth {
    state: Mutex<State>,
}

impl EndpointHealth {
    pub fn new() -> Self {
        EndpointHealth {
            state: Mutex::new(State {
                unhealthy: Unhealthy::Healthy,
                broken: Broken::Errors { count: 0 },
            }),
        }
    }

    pub fn mark(&self, status: StatusCode) {
        let now = Instant::now();
        let mut state = self.state.lock();
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            state.unhealthy.error(now);
            state.broken.error();
        } else if !status.is_server_error() {
            state.broken.success();
        }
    }
}

/// An endpoint which monitors endpoints, reporting two categories of errors:
///
/// * Unhealthy endpoints consistently returned internal server errors over an extended time window.
/// * Broken endpoints have only returned internal server errors since the server started.
pub struct Endpoint500sHealthCheck {
    endpoints: HashMap<String, Arc<EndpointHealth>>,
}

impl Endpoint500sHealthCheck {
    pub fn new(endpoints: &[Box<dyn WitchcraftEndpoint + Sync + Send>]) -> Self {
        Endpoint500sHealthCheck {
            endpoints: endpoints
                .iter()
                .filter_map(|e| {
                    e.health().map(|health| {
                        (format!("{}.{}", e.service_name(), e.name()), health.clone())
                    })
                })
                .collect(),
        }
    }

    fn unhealthy_endpoints(&self) -> BTreeSet<&str> {
        let now = Instant::now();
        self.endpoints
            .iter()
            .filter(|(_, v)| v.state.lock().unhealthy.check(now))
            .map(|(k, _)| &**k)
            .collect()
    }

    fn broken_endpoints(&self) -> BTreeSet<&str> {
        self.endpoints
            .iter()
            .filter(|(_, v)| v.state.lock().broken.check())
            .map(|(k, _)| &**k)
            .collect()
    }

    fn message(
        &self,
        unhealthy_endpoints: &BTreeSet<&str>,
        broken_endpoints: &BTreeSet<&str>,
    ) -> String {
        let mut message = String::new();
        if !unhealthy_endpoints.is_empty() {
            writeln!(
                message,
                "There have been HTTP 500s returned by endpoints in every {GRACE_PERIOD:?} rolling period in the last \
                 {EVALUATION_PERIOD:?}. This indicates a non-transient error.",
            ).unwrap();
        }

        if !broken_endpoints.is_empty() {
            message.push_str("At least one endpoint is consistently failing since startup.");
        }

        message
    }
}

impl HealthCheck for Endpoint500sHealthCheck {
    fn type_(&self) -> &str {
        "ENDPOINT_FIVE_HUNDREDS"
    }

    fn result(&self) -> HealthCheckResult {
        let unhealthy_endpoints = self.unhealthy_endpoints();
        let broken_endpoints = self.broken_endpoints();

        if unhealthy_endpoints.is_empty() && broken_endpoints.is_empty() {
            return HealthCheckResult::builder()
                .state(HealthState::Healthy)
                .build();
        }

        HealthCheckResult::builder()
            .state(HealthState::Warning)
            .message(self.message(&unhealthy_endpoints, &broken_endpoints))
            .params(BTreeMap::from([
                (
                    "unhealthyEndpoints".to_string(),
                    Any::new(unhealthy_endpoints).unwrap(),
                ),
                (
                    "brokenEndpoints".to_string(),
                    Any::new(broken_endpoints).unwrap(),
                ),
            ]))
            .build()
    }
}
