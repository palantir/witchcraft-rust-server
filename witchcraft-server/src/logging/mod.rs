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

//! Logging APIs
use crate::extensions::AuditLogEntry;
use crate::logging::api::{AuditLogV3, EventLogV2, RequestLogV2};
use crate::shutdown_hooks::ShutdownHooks;
use conjure_error::Error;
use futures::executor::block_on;
use futures_channel::oneshot;
use lazycell::AtomicLazyCell;
pub(crate) use logger::{Appender, Payload};
use refreshable::Refreshable;
use std::sync::Arc;
use tokio::sync::Mutex;
use witchcraft_metrics::MetricRegistry;
use witchcraft_server_config::install::InstallConfig;
use witchcraft_server_config::runtime::LoggingConfig;

/// Conjure-generated type definitions for log formats.
#[allow(warnings)]
#[rustfmt::skip]
pub mod api;
mod cleanup;
mod format;
mod logger;
pub mod mdc;
mod metric;
mod service;
mod trace;

pub(crate) static AUDIT_LOGGER: AtomicLazyCell<Arc<Mutex<Appender<AuditLogV3>>>> =
    AtomicLazyCell::NONE;
pub(crate) static EVENT_LOGGER: AtomicLazyCell<Arc<Mutex<Appender<EventLogV2>>>> =
    AtomicLazyCell::NONE;

pub(crate) const REQUEST_ID_KEY: &str = "_requestId";
pub(crate) const SAMPLED_KEY: &str = "_sampled";

pub(crate) struct Loggers {
    pub request_logger: Arc<Appender<RequestLogV2>>,
    pub audit_logger: Arc<Mutex<Appender<AuditLogV3>>>,
    pub event_logger: Arc<Mutex<Appender<EventLogV2>>>,
}

pub(crate) fn early_init() {
    service::early_init()
}

pub(crate) async fn init(
    metrics: &Arc<MetricRegistry>,
    install: &InstallConfig,
    runtime: &Refreshable<LoggingConfig, Error>,
    hooks: &mut ShutdownHooks,
) -> Result<Loggers, Error> {
    metric::init(metrics, install, hooks).await?;
    service::init(metrics, install, runtime, hooks).await?;
    trace::init(metrics, install, runtime, hooks).await?;
    let request_logger = logger::appender(install, metrics, hooks).await?;
    let request_logger = Arc::new(request_logger);
    let audit_logger = logger::appender(install, metrics, hooks).await?;
    let audit_logger = Arc::new(Mutex::new(audit_logger));
    let event_logger = logger::appender(install, metrics, hooks).await?;
    let event_logger = Arc::new(Mutex::new(event_logger));

    AUDIT_LOGGER
        .fill(audit_logger.clone())
        .ok()
        .expect("Audit logger already initialized");
    EVENT_LOGGER
        .fill(event_logger.clone())
        .ok()
        .expect("Event logger already initialized");

    cleanup::cleanup_logs().await;

    Ok(Loggers {
        request_logger,
        audit_logger,
        event_logger,
    })
}

/// Write the provided v3 audit log entry to the audit log using the global audit logger.
/// Returns an error if the global audit logger is not initialized.
///
/// The returned future completes once the audit log has been successfully written.
pub async fn audit_log(entry: AuditLogEntry) -> Result<(), Error> {
    let audit_logger = AUDIT_LOGGER
        .borrow()
        .ok_or_else(|| Error::internal_safe("Audit logger not initialized"))?;

    let (tx, rx) = oneshot::channel();

    audit_logger
        .lock()
        .await
        .try_send(Payload {
            value: entry.0,
            cb: Some(tx),
        })
        .map_err(|_| Error::internal_safe("Audit logger is closed or not ready"))?;

    match rx.await {
        Ok(true) => Ok(()),
        Ok(false) => Err(Error::internal_safe("Error writing audit log")),
        Err(error) => Err(Error::internal_safe(error)),
    }
}

/// Blocking variant of [audit_log] that only returns once the audit log has been
/// successfully written or if the audit log has failed.
pub fn audit_log_blocking(entry: AuditLogEntry) -> Result<(), Error> {
    block_on(audit_log(entry))
}

/// Writes the provided V2 event log entry using the standard logging appender.
/// Returns an error if the global event logger is not initialized.
///
/// The returned future completes once the event log has been successfully written.
pub async fn event_log(entry: EventLogV2) -> Result<(), Error> {
    let event_logger = EVENT_LOGGER
        .borrow()
        .ok_or_else(|| Error::internal_safe("Event logger not initialized"))?;

    let (tx, rx) = oneshot::channel();

    event_logger
        .lock()
        .await
        .try_send(Payload {
            value: entry,
            cb: Some(tx),
        })
        .map_err(|_| Error::internal_safe("Event logger is closed or not ready"))?;

    match rx.await {
        Ok(true) => Ok(()),
        Ok(false) => Err(Error::internal_safe("Error writing event log")),
        Err(error) => Err(Error::internal_safe(error)),
    }
}

/// Blocking variant of [event_log] that only returns once the event log has been
/// successfully written or if the event log has failed.
pub fn event_log_blocking(entry: EventLogV2) -> Result<(), Error> {
    block_on(event_log(entry))
}
