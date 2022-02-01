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
//! A highly opinionated embedded application server for RESTy APIs.
//!
//! # Configuration
//!
//! The configuration for a Witchcraft server is split into two files. `install.yml` contains the configuration that is
//! fixed at server startup, and `runtime.yml` contains the configuration that can be updated dynamically at runtime.
//! These are deserialized into Rust types via the [`serde::Deserialize`] trait. Witchcraft's own internal configuration
//! is represented by the [`InstallConfig`] and [`RuntimeConfig`] types. Services that need their own configuration
//! should embed the Witchcraft configuration within their own using the `#[serde(flatten)]` annotation and implement
//! the [`AsRef`] trait:
//!
//! ```
//! use serde::Deserialize;
//! use witchcraft_server::config::install::InstallConfig;
//!
//! #[derive(Deserialize)]
//! #[serde(rename_all = "kebab-case")]
//! struct MyInstallConfig {
//!     shave_yaks: bool,
//!     #[serde(flatten)]
//!     base: InstallConfig,
//! }
//!
//! impl AsRef<InstallConfig> for MyInstallConfig {
//!     fn as_ref(&self) -> &InstallConfig {
//!         &self.base
//!     }
//! }
//! ```
//!
//! The service's custom configuration will then sit next to the standard Witchcraft configuration in `install.yml`:
//!
//! ```yml
//! product-name: my-service
//! product-version: 1.0.0
//! port: 12345
//! shave-yaks: true
//! ```
#![warn(missing_docs)]

#[doc(inline)]
pub use witchcraft_server_config as config;
