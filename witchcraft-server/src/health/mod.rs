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
pub use api::HealthState;
use conjure_object::Any;
pub use registry::HealthCheckRegistry;
use staged_builder::staged_builder;
use std::collections::BTreeMap;

#[allow(warnings)]
pub(crate) mod api;
mod registry;

/// A health check.
pub trait HealthCheck {
    /// Returns the check's type.
    ///
    /// The type must be `SCREAMING_SNAKE_CASE`.
    fn type_(&self) -> &str;

    /// Performs the check, returning its result.
    fn result(&self) -> HealthCheckResult;
}

/// The result of a health check.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[staged_builder]
pub struct HealthCheckResult {
    state: HealthState,
    #[builder(default, into)]
    message: Option<String>,
    // FIXME(sfackler) https://github.com/sfackler/staged-builder/issues/2
    #[builder(default)]
    params: BTreeMap<String, Any>,
}

impl HealthCheckResult {
    /// Health state of the check.
    pub fn state(&self) -> &HealthState {
        &self.state
    }

    /// Text describing the state of the check which should provide enough information for the check to be actionable
    /// when included in an alert.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Additional redacted information on the nature of the health check.
    pub fn params(&self) -> &BTreeMap<String, Any> {
        &self.params
    }
}
