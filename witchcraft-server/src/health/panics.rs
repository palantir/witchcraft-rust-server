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
use crate::health::{health_check_result, CheckType, HealthCheck, HealthCheckResult, HealthState};
use parking_lot::Mutex;
use std::panic;
use std::sync::Arc;
use std::time::{Duration, Instant};

const THRESHOLD: Duration = Duration::from_secs(5 * 60);

pub struct PanicsHealthCheck {
    last_panic: Arc<Mutex<Option<Instant>>>,
}

impl PanicsHealthCheck {
    pub fn new() -> Self {
        let last_panic = Arc::new(Mutex::new(None));

        let hook = panic::take_hook();
        panic::set_hook({
            let last_panic = last_panic.clone();
            Box::new(move |payload| {
                *last_panic.lock() = Some(Instant::now());
                hook(payload)
            })
        });

        PanicsHealthCheck { last_panic }
    }
}

impl HealthCheck for PanicsHealthCheck {
    fn type_(&self) -> CheckType {
        CheckType("PANICS".to_string())
    }

    fn result(&self, builder: health_check_result::BuilderStage1) -> HealthCheckResult {
        if self
            .last_panic
            .lock()
            .map_or(false, |t| t.elapsed() < THRESHOLD)
        {
            builder
                .state(HealthState::Warning)
                .message(format!("A thread has panicked in the last {:?}", THRESHOLD))
                .build()
        } else {
            builder
                .state(HealthState::Healthy)
                .message(format!(
                    "No thread has panicked in the last {:?}",
                    THRESHOLD
                ))
                .build()
        }
    }
}
