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
use crate::health::api::health_check_result::HealthCheckResult;
use crate::health::api::{CheckType, HealthState, HealthStatus};
use crate::health::HealthCheck;
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::task::{self, JoinError, JoinHandle};
use tokio::time::{self, Instant};

const CHECK_RUN_INTERVAL: Duration = Duration::from_secs(30);
const STALENESS_THRESHOLD: Duration = Duration::from_secs(5 * 60);
static HEALTH_CHECK_COMPUTATION_STALENESS_TYPE: Lazy<CheckType> =
    Lazy::new(|| CheckType("HEALTH_CHECK_COMPUTATION_STALENESS".to_string()));
static TYPE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new("^[A-Z_]+$").unwrap());

struct TimestampedResult {
    result: HealthCheckResult,
    time: Instant,
}

impl TimestampedResult {
    fn new(result: HealthCheckResult) -> Self {
        TimestampedResult {
            result,
            time: Instant::now(),
        }
    }
}

struct State {
    check: ArcSwap<Box<dyn HealthCheck + Sync + Send>>,
    result: ArcSwap<TimestampedResult>,
}

struct InstalledCheck {
    state: Arc<State>,
    handle: JoinHandle<()>,
}

impl Drop for InstalledCheck {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// A registry of health checks for the server.
pub struct HealthCheckRegistry {
    checks: Mutex<HashMap<CheckType, InstalledCheck>>,
    handle: Handle,
}

impl HealthCheckRegistry {
    pub(crate) fn new(handle: &Handle) -> Self {
        HealthCheckRegistry {
            checks: Mutex::new(HashMap::new()),
            handle: handle.clone(),
        }
    }

    /// Registers a new health check.
    ///
    /// # Panics
    ///
    /// Panics if the check's type is not SCREAMING_SNAKE_CASE or a check with the same type is already registered.
    pub fn register<T>(&self, check: T)
    where
        T: HealthCheck + 'static + Sync + Send,
    {
        self.register_inner(Box::new(check))
    }

    fn register_inner(&self, check: Box<dyn HealthCheck + Sync + Send>) {
        let type_ = check.type_();

        assert!(
            TYPE_PATTERN.is_match(&type_),
            "{} must be uppercase and underscores only",
            type_,
        );

        match self.checks.lock().entry(type_) {
            Entry::Occupied(_) => panic!(
                "a check has already been registered for type {}",
                check.type_()
            ),
            Entry::Vacant(e) => {
                let _guard = self.handle.enter();

                let state = Arc::new(State {
                    result: ArcSwap::new(Arc::new(TimestampedResult::new(
                        computing_for_the_first_time(check.type_()),
                    ))),
                    check: ArcSwap::new(Arc::new(check)),
                });
                let handle = task::spawn(run_check(state.clone()));

                e.insert(InstalledCheck { state, handle });
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn run_checks(&self) -> HealthStatus {
        let threshold = Instant::now() - STALENESS_THRESHOLD;
        let mut stale_checks = vec![];

        let status = HealthStatus::builder().extend_checks(self.checks.lock().iter().map(
            |(type_, check)| {
                let result = check.state.result.load();
                if result.time < threshold {
                    stale_checks.push(type_.clone());
                }
                (type_.clone(), result.result.clone())
            },
        ));

        let staleness_result = if stale_checks.is_empty() {
            HealthCheckResult::builder()
                .type_(HEALTH_CHECK_COMPUTATION_STALENESS_TYPE.clone())
                .state(HealthState::Healthy)
                .message(format!(
                    "All healthcheck results have been computed within the last {:?}",
                    STALENESS_THRESHOLD
                ))
                .build()
        } else {
            HealthCheckResult::builder()
                .type_(HEALTH_CHECK_COMPUTATION_STALENESS_TYPE.clone())
                .state(HealthState::Warning)
                .message(format!(
                    "Some healthcheck results have not been computed within the last {:?}",
                    STALENESS_THRESHOLD
                ))
                .insert_params("staleHealthChecks", stale_checks)
                .build()
        };

        status
            .insert_checks(
                HEALTH_CHECK_COMPUTATION_STALENESS_TYPE.clone(),
                staleness_result,
            )
            .build()
    }
}

fn computing_for_the_first_time(type_: CheckType) -> HealthCheckResult {
    HealthCheckResult::builder()
        .type_(type_)
        .state(HealthState::Repairing)
        .message(
            "Healthcheck has not yet been run and is currently computing for the first time"
                .to_string(),
        )
        .build()
}

fn general_error(type_: CheckType, error: JoinError) -> HealthCheckResult {
    // panic information has already been logged, so we'll just use a generic message
    let message = match error.try_into_panic() {
        Ok(_) => "Healthcheck evaluation has panicked",
        Err(_) => "Healthcheck evaluation was canceled",
    };

    HealthCheckResult::builder()
        .type_(type_)
        .state(HealthState::Error)
        .message(message.to_string())
        .build()
}

async fn run_check(state: Arc<State>) {
    loop {
        let check = state.check.load();

        let result = task::spawn_blocking({
            let check = check.clone();
            move || {
                let type_ = check.type_();
                let _span = zipkin::new_trace().with_name(&format!("healthcheck: {}", type_));
                check.result(HealthCheckResult::builder().type_(type_))
            }
        })
        .await;

        let result = match result {
            Ok(result) => result,
            Err(e) => general_error(check.type_(), e),
        };

        state.result.store(Arc::new(TimestampedResult::new(result)));

        time::sleep(CHECK_RUN_INTERVAL).await;
    }
}
