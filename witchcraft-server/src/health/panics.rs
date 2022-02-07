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
use crate::health::{HealthCheck, HealthCheckResult, HealthState};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A health check which reports a warning state if a panic has occurred since the server started.
pub struct PanicsHealthCheck {
    panicked: Arc<AtomicBool>,
}

impl PanicsHealthCheck {
    pub fn new() -> Self {
        let panicked = Arc::new(AtomicBool::new(false));

        let hook = panic::take_hook();
        panic::set_hook({
            let panicked = panicked.clone();
            Box::new(move |payload| {
                panicked.store(true, Ordering::Relaxed);
                hook(payload)
            })
        });

        PanicsHealthCheck { panicked }
    }
}

impl HealthCheck for PanicsHealthCheck {
    fn type_(&self) -> &str {
        "PANICS"
    }

    fn result(&self) -> HealthCheckResult {
        if self.panicked.load(Ordering::Relaxed) {
            // We don't want to page on panic, just fail blue-green rollouts
            HealthCheckResult::builder()
                .state(HealthState::Warning)
                .message("A thread has panicked".to_string())
                .build()
        } else {
            HealthCheckResult::builder()
                .state(HealthState::Healthy)
                .message("No thread has panicked".to_string())
                .build()
        }
    }
}
