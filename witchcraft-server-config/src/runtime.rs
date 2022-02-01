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
//! Runtime-reloadable configuration.
use conjure_runtime_config::ServicesConfig;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use witchcraft_log::LevelFilter;

/// The runtime-reloadable configuration for a Witchcraft server.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct RuntimeConfig {
    diagnostics: DiagnosticsConfig,
    health_checks: HealthChecksConfig,
    #[serde(default)]
    logging: LoggingConfig,
    #[serde(default)]
    service_discovery: ServicesConfig,
}

impl AsRef<RuntimeConfig> for RuntimeConfig {
    #[inline]
    fn as_ref(&self) -> &RuntimeConfig {
        self
    }
}

impl RuntimeConfig {
    /// Returns the server's diagnostics configuration.
    ///
    /// Required.
    #[inline]
    pub fn diagnostics(&self) -> &DiagnosticsConfig {
        &self.diagnostics
    }

    /// Returns the server's health checks configuration.
    ///
    /// Required.
    #[inline]
    pub fn health_checks(&self) -> &HealthChecksConfig {
        &self.health_checks
    }

    /// Returns the server's logging configuration.
    #[inline]
    pub fn logging(&self) -> &LoggingConfig {
        &self.logging
    }

    /// Returns the server's service discovery configuration.
    #[inline]
    pub fn service_discovery(&self) -> &ServicesConfig {
        &self.service_discovery
    }
}

/// Diagnostics configuration.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct DiagnosticsConfig {
    debug_shared_secret: String,
}

impl DiagnosticsConfig {
    /// Returns the shared secret used to authorize requests to the server's debug endpoint.
    ///
    /// Required.
    #[inline]
    pub fn debug_shared_secret(&self) -> &str {
        &self.debug_shared_secret
    }
}

/// Health checks configuration.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct HealthChecksConfig {
    shared_secret: String,
}

impl HealthChecksConfig {
    /// Returns the shared secret used to authorize requests to the server's health check endpoint.
    #[inline]
    pub fn shared_secret(&self) -> &str {
        &self.shared_secret
    }
}

/// Logging configuration.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct LoggingConfig {
    level: LevelFilter,
    loggers: HashMap<String, LevelFilter>,
    #[serde(deserialize_with = "de_logging_config_tracing_rate")]
    trace_rate: f32,
}

impl Default for LoggingConfig {
    #[inline]
    fn default() -> Self {
        LoggingConfig {
            level: LevelFilter::Info,
            loggers: HashMap::new(),
            trace_rate: 0.0005,
        }
    }
}

#[inline]
fn de_logging_config_tracing_rate<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let rate = f32::deserialize(deserializer)?;
    if !(0.0..=1.0).contains(&rate) {
        return Err(de::Error::invalid_value(
            de::Unexpected::Float(f64::from(rate)),
            &"valid rate",
        ));
    }

    Ok(rate)
}

impl LoggingConfig {
    /// Returns the logging verbosity filter applied globally for all service logs.
    ///
    /// Defaults to [`LevelFilter::Info`].
    #[inline]
    pub fn level(&self) -> LevelFilter {
        self.level
    }

    /// Returns a map of verbosity filter overides applied to specific targets.
    #[inline]
    pub fn loggers(&self) -> &HashMap<String, LevelFilter> {
        &self.loggers
    }

    /// Returns the rate at which new traces will be sampled between 0 and 1, inclusive.
    ///
    /// This only applies to fresh traces - Witchcraft will respect sampling decisions made by upstream services for a
    /// given request.
    ///
    /// Defaults to 0.05%.
    #[inline]
    pub fn trace_rate(&self) -> f32 {
        self.trace_rate
    }
}
