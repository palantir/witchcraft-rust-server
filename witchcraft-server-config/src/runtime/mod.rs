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
use crate::ConfigError;
use conjure_runtime_config::ServicesConfig;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use staged_builder::{staged_builder, Validate};
use std::collections::HashMap;
use witchcraft_log::LevelFilter;

mod de;

/// The runtime-reloadable configuration for a Witchcraft server.
#[derive(Clone, PartialEq, Debug)]
#[staged_builder]
pub struct RuntimeConfig {
    diagnostics: DiagnosticsConfig,
    health_checks: HealthChecksConfig,
    #[builder(default)]
    logging: LoggingConfig,
    #[builder(default)]
    service_discovery: ServicesConfig,
}

impl<'de> Deserialize<'de> for RuntimeConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = de::RuntimeConfig::deserialize(deserializer)?;
        let mut builder = RuntimeConfig::builder()
            .diagnostics(raw.diagnostics)
            .health_checks(raw.health_checks);
        if let Some(logging) = raw.logging {
            builder = builder.logging(logging);
        }
        if let Some(service_discovery) = raw.service_discovery {
            builder = builder.service_discovery(service_discovery);
        }

        Ok(builder.build())
    }
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
#[derive(Clone, PartialEq, Debug)]
#[staged_builder]
pub struct DiagnosticsConfig {
    #[builder(into)]
    debug_shared_secret: String,
}

impl<'de> Deserialize<'de> for DiagnosticsConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = de::DiagnosticsConfig::deserialize(deserializer)?;
        let builder = DiagnosticsConfig::builder().debug_shared_secret(raw.debug_shared_secret);

        Ok(builder.build())
    }
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
#[derive(Clone, PartialEq, Debug)]
#[staged_builder]
pub struct HealthChecksConfig {
    #[builder(into)]
    shared_secret: String,
}

impl<'de> Deserialize<'de> for HealthChecksConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = de::HealthChecksConfig::deserialize(deserializer)?;
        let builder = HealthChecksConfig::builder().shared_secret(raw.shared_secret);

        Ok(builder.build())
    }
}

impl HealthChecksConfig {
    /// Returns the shared secret used to authorize requests to the server's health check endpoint.
    #[inline]
    pub fn shared_secret(&self) -> &str {
        &self.shared_secret
    }
}

/// Logging configuration.
#[derive(Clone, PartialEq, Debug)]
#[staged_builder]
#[builder(validate)]
pub struct LoggingConfig {
    #[builder(default = LevelFilter::Info)]
    level: LevelFilter,
    #[builder(map(key(type = String, into), value(type = LevelFilter)))]
    loggers: HashMap<String, LevelFilter>,
    #[builder(default = 0.0005)]
    trace_rate: f32,
}

impl Validate for LoggingConfig {
    type Error = ConfigError;

    fn validate(&self) -> Result<(), Self::Error> {
        if !(0.0..=1.0).contains(&self.trace_rate()) {
            return Err(ConfigError(
                "trace-rate must be between 0 and 1, inclusive".to_string(),
            ));
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for LoggingConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = de::LoggingConfig::deserialize(deserializer)?;
        let mut builder = LoggingConfig::builder();
        if let Some(level) = raw.level {
            builder = builder.level(level);
        }
        if let Some(loggers) = raw.loggers {
            builder = builder.loggers(loggers);
        }
        if let Some(trace_rate) = raw.trace_rate {
            builder = builder.trace_rate(trace_rate);
        }

        builder.build().map_err(Error::custom)
    }
}

impl Default for LoggingConfig {
    #[inline]
    fn default() -> Self {
        LoggingConfig::builder().build().unwrap()
    }
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
