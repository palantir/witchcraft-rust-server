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
use super::HealthState;
use crate::health::api::{self, CheckType, HealthStatus};
use crate::health::{HealthCheck, HealthCheckResult};
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tokio::task::{self, JoinHandle};
use tokio::time;

const CHECK_RUN_INTERVAL: Duration = Duration::from_secs(30);
const STALENESS_THRESHOLD: Duration = Duration::from_secs(5 * 60);
const HEALTH_CHECK_COMPUTATION_STALENESS_TYPE: &str = "HEALTH_CHECK_COMPUTATION_STALENESS";
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

struct InstalledCheck {
    check: Arc<dyn HealthCheck + Sync + Send>,
    result: Arc<ArcSwap<TimestampedResult>>,
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
    /// If a check already exists with the same type, it will be replaced.
    ///
    /// # Panics
    ///
    /// Panics if the check's type is not `SCREAMING_SNAKE_CASE`.
    pub fn register<T>(&self, check: T)
    where
        T: HealthCheck + 'static + Sync + Send,
    {
        let type_ = check.type_();
        self.check_type(type_);

        self.checks.lock().insert(
            CheckType(type_.to_string()),
            self.make_check(Arc::new(check)),
        );
    }

    /// Registers a new check if one does not already exist with the same type.
    ///
    /// The registered check will be returned.
    ///
    /// # Panics
    ///
    /// Panics if the check's type is not `SCREAMING_SNAKE_CASE`.
    pub fn register_if_absent<T>(&self, check: T) -> Arc<dyn HealthCheck + Sync + Send>
    where
        T: HealthCheck + 'static + Sync + Send,
    {
        let type_ = check.type_();
        self.check_type(type_);

        match self.checks.lock().entry(CheckType(type_.to_string())) {
            Entry::Occupied(e) => e.get().check.clone(),
            Entry::Vacant(e) => e.insert(self.make_check(Arc::new(check))).check.clone(),
        }
    }

    fn check_type(&self, type_: &str) {
        assert!(
            TYPE_PATTERN.is_match(type_),
            "{type_} must be `SCREAMING_SNAKE_CASE`",
        );
    }

    fn make_check(&self, check: Arc<dyn HealthCheck + Sync + Send>) -> InstalledCheck {
        let _guard = self.handle.enter();

        let result = Arc::new(ArcSwap::new(Arc::new(TimestampedResult::new(
            computing_for_the_first_time(),
        ))));
        let handle = task::spawn(run_check(check.clone(), result.clone()));

        InstalledCheck {
            check,
            result,
            handle,
        }
    }

    pub(crate) fn run_checks(&self) -> HealthStatus {
        let threshold = Instant::now() - STALENESS_THRESHOLD;
        let mut stale_checks = vec![];

        let status = HealthStatus::builder().extend_checks(self.checks.lock().iter().map(
            |(type_, check)| {
                let result = check.result.load();
                if result.time < threshold {
                    stale_checks.push(type_.clone());
                }

                let result = api::HealthCheckResult::builder()
                    .type_(type_.clone())
                    .state(result.result.state().clone())
                    .message(result.result.message().map(|s| s.to_string()))
                    .params(
                        result
                            .result
                            .params()
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone())),
                    )
                    .build();

                (type_.clone(), result)
            },
        ));

        let staleness_result = if stale_checks.is_empty() {
            api::HealthCheckResult::builder()
                .type_(CheckType(HEALTH_CHECK_COMPUTATION_STALENESS_TYPE.to_string()))
                .state(HealthState::Healthy)
                .message(format!("All healthcheck results have been computed within the last {STALENESS_THRESHOLD:?}"))
                .build()
        } else {
            api::HealthCheckResult::builder()
                .type_(CheckType(HEALTH_CHECK_COMPUTATION_STALENESS_TYPE.to_string()))
                .state(HealthState::Warning)
                .message(
                    format!("Some healthcheck results have not been computed within the last {STALENESS_THRESHOLD:?}"),
                )
                .insert_params("staleHealthChecks", stale_checks)
                .build()
        };

        status
            .insert_checks(
                CheckType(HEALTH_CHECK_COMPUTATION_STALENESS_TYPE.to_string()),
                staleness_result,
            )
            .build()
    }
}

fn computing_for_the_first_time() -> HealthCheckResult {
    HealthCheckResult::builder()
        .state(HealthState::Repairing)
        .message(
            "Healthcheck has not yet been run and is currently computing for the first time"
                .to_string(),
        )
        .build()
}

fn panic_error() -> HealthCheckResult {
    HealthCheckResult::builder()
        .state(HealthState::Error)
        .message("Healthcheck evaluation has panicked".to_string())
        .build()
}

async fn run_check(
    check: Arc<dyn HealthCheck + Sync + Send>,
    state: Arc<ArcSwap<TimestampedResult>>,
) {
    loop {
        // it's okay to use block_in_place here since we know this future is running as its own task.
        let result = task::block_in_place(|| {
            let _span = zipkin::new_trace().with_name(&format!("healthcheck: {}", check.type_()));
            panic::catch_unwind(AssertUnwindSafe(|| check.result()))
        });

        let result = match result {
            Ok(result) => result,
            Err(_) => panic_error(),
        };

        state.store(Arc::new(TimestampedResult::new(result)));

        time::sleep(CHECK_RUN_INTERVAL).await;
    }
}
