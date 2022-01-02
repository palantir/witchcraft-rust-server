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
//! Health checks.
pub use api::{health_check_result, CheckType, HealthCheckResult, HealthState};
pub use registry::HealthCheckRegistry;

#[allow(warnings)]
pub(crate) mod api;
pub(crate) mod config_reload;
pub(crate) mod panics;
mod registry;
pub(crate) mod service_dependency;

/// A health check.
pub trait HealthCheck {
    /// Returns the check's type.
    ///
    /// The type must be SCREAMING_SNAKE_CASE.
    fn type_(&self) -> CheckType;

    /// Performs the check, returning its result.
    ///
    /// For convenience, a result builder with the check's type already set is passed in.
    fn result(&self, builder: health_check_result::BuilderStage1) -> HealthCheckResult;
}
