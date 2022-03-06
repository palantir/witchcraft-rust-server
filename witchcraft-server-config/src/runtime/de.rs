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
use serde::Deserialize;
use std::collections::HashMap;
use witchcraft_log::LevelFilter;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RuntimeConfig {
    pub diagnostics: super::DiagnosticsConfig,
    pub health_checks: super::HealthChecksConfig,
    pub logging: Option<super::LoggingConfig>,
    pub service_discovery: Option<super::ServicesConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DiagnosticsConfig {
    pub debug_shared_secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HealthChecksConfig {
    pub shared_secret: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LoggingConfig {
    pub level: Option<LevelFilter>,
    pub loggers: Option<HashMap<String, LevelFilter>>,
    pub trace_rate: Option<f32>,
}
