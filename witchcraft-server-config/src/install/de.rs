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
use std::path::PathBuf;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InstallConfig {
    pub product_name: String,
    pub product_version: String,
    pub port: u16,
    pub management_port: Option<u16>,
    pub keystore: Option<super::KeystoreConfig>,
    pub client_auth_truststore: Option<super::ClientAuthTruststoreConfig>,
    pub context_path: Option<String>,
    pub use_console_log: Option<bool>,
    pub server: Option<super::ServerConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct KeystoreConfig {
    pub key_path: Option<PathBuf>,
    pub cert_path: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClientAuthTruststoreConfig {
    pub path: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    pub processors: Option<usize>,
    pub min_threads: Option<usize>,
    pub max_threads: Option<usize>,
    pub max_connections: Option<usize>,
    pub io_threads: Option<usize>,
    #[serde(default, with = "humantime_serde")]
    pub idle_thread_timeout: Option<Duration>,
    pub shutdown_timeout: Option<Duration>,
    pub gzip: Option<bool>,
    pub http2: Option<bool>,
    #[serde(default, with = "humantime_serde")]
    pub idle_connection_timeout: Option<Duration>,
}
