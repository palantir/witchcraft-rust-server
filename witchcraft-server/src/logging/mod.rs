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
use crate::logging::api::RequestLogV2;
use crate::shutdown_hooks::ShutdownHooks;
use conjure_error::Error;
pub use logger::Appender;
use refreshable::Refreshable;
use std::sync::Arc;
use witchcraft_metrics::MetricRegistry;
use witchcraft_server_config::install::InstallConfig;
use witchcraft_server_config::runtime::LoggingConfig;

#[allow(warnings)]
#[rustfmt::skip]
pub mod api;
mod cleanup;
mod format;
mod logger;
mod metric;
mod service;
mod trace;

pub const UID_MDC_KEY: &str = "\0witchcraft-uid";
pub const SID_MDC_KEY: &str = "\0witchcraft-sid";
pub const TOKEN_ID_MDC_KEY: &str = "\0witchcraft-token-id";
pub const TRACE_ID_MDC_KEY: &str = "\0witchcraft-trace-id";
pub const REQUEST_ID_KEY: &str = "_requestId";
pub const SAMPLED_KEY: &str = "_sampled";

pub struct Loggers {
    pub request_logger: Appender<RequestLogV2>,
}

pub fn early_init() {
    service::early_init()
}

pub async fn init(
    metrics: &Arc<MetricRegistry>,
    install: &InstallConfig,
    runtime: &Refreshable<LoggingConfig, Error>,
    hooks: &mut ShutdownHooks,
) -> Result<Loggers, Error> {
    metric::init(metrics, install, hooks).await?;
    service::init(metrics, install, runtime, hooks).await?;
    trace::init(metrics, install, runtime, hooks).await?;
    let request_logger = logger::appender(install, metrics, hooks).await?;

    cleanup::cleanup_logs().await;

    Ok(Loggers { request_logger })
}
