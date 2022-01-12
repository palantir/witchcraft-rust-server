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
use crate::logging;
use crate::logging::api::{LogLevel, ServiceLogV1, SessionId, TokenId, TraceId, UserId};
use crate::logging::logger::{self, Appender};
use crate::shutdown::ShutdownHooks;
use arc_swap::ArcSwap;
use conjure_error::{Error, ErrorKind};
use conjure_object::Utc;
use refreshable::{Refreshable, Subscription};
use sequence_trie::SequenceTrie;
use serde::Deserialize;
use std::fmt::Write;
use std::sync::Arc;
use std::{error, panic, thread};
use witchcraft_log::bridge::{self, BridgedLogger};
use witchcraft_log::{error, mdc};
use witchcraft_log::{Level, LevelFilter, Log, Metadata, Record};
use witchcraft_metrics::MetricRegistry;
use witchcraft_server_config::install::InstallConfig;
use witchcraft_server_config::runtime::LoggingConfig;

pub async fn init(
    metrics: &MetricRegistry,
    install: &InstallConfig,
    runtime: &Refreshable<LoggingConfig, Error>,
    hooks: &mut ShutdownHooks,
) -> Result<(), Error> {
    let appender = logger::appender(install, metrics, hooks).await?;
    let levels = Arc::new(ArcSwap::new(Arc::new(Levels::empty())));
    let subscription = runtime.subscribe_ok({
        let levels = levels.clone();
        move |config| {
            let new_levels = Levels::new(config);
            let max_level = new_levels.max_level();
            witchcraft_log::set_max_level(max_level);
            bridge::set_max_level(max_level);
            levels.store(Arc::new(new_levels));
        }
    });

    let logger = ServiceLogger {
        appender,
        levels,
        _subscription: subscription,
    };
    let logger = Box::leak(Box::new(logger));
    witchcraft_log::set_logger(logger).expect("logger already initialized");
    log::set_logger(&BridgedLogger).expect("logger already initialized");
    log_panics();

    Ok(())
}

pub struct ServiceLogger {
    appender: Appender<ServiceLogV1>,
    levels: Arc<ArcSwap<Levels>>,
    _subscription: Subscription<LoggingConfig, Error>,
}

impl Log for ServiceLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.levels.load().enabled(metadata)
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = match record.level() {
            Level::Fatal => LogLevel::Fatal,
            Level::Error => LogLevel::Error,
            Level::Warn => LogLevel::Warn,
            Level::Info => LogLevel::Info,
            Level::Debug => LogLevel::Debug,
            Level::Trace => LogLevel::Trace,
        };

        let mut message = ServiceLogV1::builder()
            .type_("service.1")
            .level(level)
            .time(Utc::now())
            .message(record.message())
            .safe(true)
            .origin(record.target().to_string())
            .thread(thread::current().name().map(ToString::to_string));

        let mdc = mdc::snapshot();
        for (key, value) in mdc.safe_iter() {
            match key {
                logging::UID_MDC_KEY => {
                    if let Ok(uid) = String::deserialize(value.clone()) {
                        message = message.uid(UserId(uid));
                    }
                }
                logging::SID_MDC_KEY => {
                    if let Ok(sid) = String::deserialize(value.clone()) {
                        message = message.sid(SessionId(sid));
                    }
                }
                logging::TOKEN_ID_MDC_KEY => {
                    if let Ok(token_id) = String::deserialize(value.clone()) {
                        message = message.token_id(TokenId(token_id));
                    }
                }
                logging::TRACE_ID_MDC_KEY => {
                    if let Ok(trace_id) = String::deserialize(value.clone()) {
                        message = message.trace_id(TraceId(trace_id));
                    }
                }
                key => message = message.insert_params(key, value),
            }
        }
        message = message
            .extend_unsafe_params(mdc.unsafe_iter().map(|(k, v)| (k.to_string(), v.clone())));

        if let Some(file) = record.file() {
            message = message.insert_params("file", file);
        }
        if let Some(line) = record.line() {
            message = message.insert_params("line", line);
        }
        if let Some(error) = record.error() {
            if let ErrorKind::Service(s) = error.kind() {
                message = message
                    .insert_params("errorInstanceId", s.error_instance_id())
                    .insert_params("errorCode", s.error_code())
                    .insert_params("errorName", s.error_name());
            }

            let mut stacktrace = String::new();
            for trace in error.backtraces() {
                writeln!(stacktrace, "{:?}", trace).unwrap();
            }
            message = message.stacktrace(stacktrace);

            let mut causes = vec![];
            let mut cause = Some(error.cause() as &dyn error::Error);
            while let Some(e) = cause {
                causes.push(e.to_string());
                cause = e.source();
            }
            if error.cause_safe() {
                message = message.insert_params("errorCause", causes);
            } else {
                message = message.insert_unsafe_params("errorCause", causes);
            }
            for (key, value) in &error.safe_params() {
                message = message.insert_params(key, value);
            }
            for (key, value) in &error.unsafe_params() {
                message = message.insert_unsafe_params(key, value);
            }
        }
        for (key, value) in record.safe_params() {
            message = message.insert_params(*key, value);
        }
        for (key, value) in record.unsafe_params() {
            message = message.insert_unsafe_params(*key, value);
        }

        let _ = self.appender.try_send(message.build());
    }

    fn flush(&self) {
        // We flush via a different mode.
    }
}

struct Levels {
    trie: SequenceTrie<String, LevelFilter>,
}

impl Levels {
    fn empty() -> Self {
        Levels {
            trie: SequenceTrie::new(),
        }
    }

    fn new(config: &LoggingConfig) -> Self {
        let mut trie = SequenceTrie::new();
        trie.insert_owned([], config.level());
        for (logger, level) in config.loggers() {
            trie.insert(logger.split("::"), *level);
        }

        Levels { trie }
    }

    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level()
            <= *self
                .trie
                .get_ancestor(metadata.target().split("::"))
                .unwrap()
    }

    fn max_level(&self) -> LevelFilter {
        self.trie.values().cloned().max().unwrap()
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

#[cfg(test)]
mod test {
    use super::*;
    use conjure_serde::json;
    use witchcraft_log::Level;

    #[test]
    fn loggers() {
        let config = r#"
        {
            "level": "INFO",
            "loggers": {
                "foo": "WARN",
                "foo::bar": "DEBUG"
            }
        }
        "#;
        let config = json::server_from_str(config).unwrap();

        let loggers = Levels::new(&config);

        assert!(loggers.enabled(&Metadata::builder().level(Level::Info).target("bar").build()));
        assert!(!loggers.enabled(
            &Metadata::builder()
                .level(Level::Debug)
                .target("bar")
                .build()
        ));

        assert!(loggers.enabled(&Metadata::builder().level(Level::Warn).target("foo").build()));
        assert!(!loggers.enabled(&Metadata::builder().level(Level::Info).target("foo").build()));

        assert!(loggers.enabled(
            &Metadata::builder()
                .level(Level::Debug)
                .target("foo::bar::baz")
                .build()
        ));
        assert!(!loggers.enabled(
            &Metadata::builder()
                .level(Level::Trace)
                .target("foo::bar::baz")
                .build()
        ));

        assert_eq!(loggers.max_level(), LevelFilter::Debug);
    }
}
