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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct ConfigReloadHealthCheck {
    config_ok: Arc<AtomicBool>,
}

impl ConfigReloadHealthCheck {
    pub fn new(config_ok: Arc<AtomicBool>) -> Self {
        ConfigReloadHealthCheck { config_ok }
    }
}

impl HealthCheck for ConfigReloadHealthCheck {
    fn type_(&self) -> CheckType {
        CheckType("CONFIG_RELOAD".to_string())
    }

    fn result(&self, builder: health_check_result::BuilderStage1) -> HealthCheckResult {
        let state = if self.config_ok.load(Ordering::Relaxed) {
            HealthState::Healthy
        } else {
            HealthState::Error
        };
        builder.state(state).build()
    }
}
