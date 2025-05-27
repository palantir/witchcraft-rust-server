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
use crate::logging::api::ServiceLogV1;
use crate::logging::logger::{self, Appender};
use crate::shutdown_hooks::ShutdownHooks;
use arc_swap::ArcSwap;
use conjure_error::Error;
use conjure_serde::json;
use once_cell::sync::OnceCell;
use refreshable::{Refreshable, Subscription};
use std::io::Write as _;
use std::sync::Arc;
use std::{io, panic};
use witchcraft_log::bridge::{self, BridgedLogger};
use witchcraft_log::error;
use witchcraft_log::{LevelFilter, Log, Metadata, Record};
use witchcraft_log_util::filter::Filter;
use witchcraft_log_util::service;
use witchcraft_metrics::MetricRegistry;
use witchcraft_server_config::install::InstallConfig;
use witchcraft_server_config::runtime::LoggingConfig;

static STATE: OnceCell<LoggerState> = OnceCell::new();

pub fn early_init() {
    witchcraft_log::set_max_level(LevelFilter::Info);
    bridge::set_max_level(LevelFilter::Info);
    witchcraft_log::set_logger(&ServiceLogger).expect("logger already initialized");
    log::set_logger(&BridgedLogger).expect("logger already initialized");
    log_panics();
}

pub async fn init(
    metrics: &MetricRegistry,
    install: &InstallConfig,
    runtime: &Refreshable<LoggingConfig, Error>,
    hooks: &mut ShutdownHooks,
) -> Result<(), Error> {
    let appender = logger::appender(install, metrics, hooks).await?;
    let filter = Arc::new(ArcSwap::new(Arc::new(Filter::builder().build())));
    let subscription = runtime.subscribe({
        let filter = filter.clone();
        move |config| {
            let mut builder = Filter::builder().level(config.level());
            for (target, level) in config.loggers() {
                builder = builder.target_level(target, *level);
            }

            let new_filter = builder.build();
            let max_level = new_filter.max_level();
            witchcraft_log::set_max_level(max_level);
            bridge::set_max_level(max_level);
            filter.store(Arc::new(new_filter));
        }
    });

    let logger = LoggerState {
        appender,
        filter,
        _subscription: subscription,
    };
    STATE.set(logger).ok().expect("logger already initialized");

    Ok(())
}

struct LoggerState {
    appender: Appender<ServiceLogV1>,
    filter: Arc<ArcSwap<Filter>>,
    _subscription: Subscription<LoggingConfig, Error>,
}

struct ServiceLogger;

impl Log for ServiceLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        match STATE.get() {
            Some(state) => state.filter.load().enabled(metadata),
            None => true,
        }
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = service::from_record(record);

        match STATE.get() {
            Some(state) => {
                let _ = state.appender.try_send(message);
            }
            None => {
                let mut buf = json::to_vec(&message).unwrap();
                buf.push(b'\n');
                let _ = io::stdout().write_all(&buf);
            }
        }
    }

    fn flush(&self) {
        // We flush via a different mode.
    }
}

fn log_panics() {
    panic::set_hook(Box::new(|info| {
        let error = if let Some(message) = info.payload().downcast_ref::<&'static str>() {
            Error::internal_safe(*message)
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            Error::internal(&**message)
        } else {
            Error::internal_safe("Box<Any>")
        };

        match info.location() {
            Some(location) => error!(
                "thread panicked",
                safe: {
                    // NB: these override the log's file and line params
                    file: location.file(),
                    line: location.line(),
                },
                error: error,
            ),
            None => error!("thread panicked", error: error),
        }
    }));
}
