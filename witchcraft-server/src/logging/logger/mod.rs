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
use crate::logging::format::LogFormat;
use crate::logging::logger::json::JsonAppender;
use crate::logging::logger::metrics::MetricsAppender;
use crate::logging::logger::r#async::AsyncAppender;
use crate::logging::logger::rolling_file::RollingFileAppender;
use crate::logging::logger::stdout::StdoutAppender;
use crate::shutdown_hooks::ShutdownHooks;
use bytes::Bytes;
use conjure_error::Error;
use futures_sink::Sink;
use serde::Serialize;
use std::io;
use std::pin::Pin;
use witchcraft_metrics::MetricRegistry;
use witchcraft_server_config::install::InstallConfig;

pub mod r#async;
mod byte_buffer;
pub mod json;
pub mod metrics;
pub mod rolling_file;
pub mod stdout;

pub type Appender<T> = AsyncAppender<T>;
pub type SyncAppender<T> =
    MetricsAppender<JsonAppender<Pin<Box<dyn Sink<Bytes, Error = io::Error> + Sync + Send>>>, T>;

pub async fn appender<T>(
    config: &InstallConfig,
    metrics: &MetricRegistry,
    hooks: &mut ShutdownHooks,
) -> Result<Appender<T>, Error>
where
    T: Serialize + LogFormat + 'static + Send,
    T::Reporter: 'static + Send,
{
    let appender = sync_appender(config, metrics).await?;
    let appender = AsyncAppender::new(appender, metrics, hooks);

    Ok(appender)
}

pub async fn sync_appender<T>(
    config: &InstallConfig,
    metrics: &MetricRegistry,
) -> Result<SyncAppender<T>, Error>
where
    T: Serialize + LogFormat + 'static + Send,
{
    let appender: Pin<Box<dyn Sink<Bytes, Error = io::Error> + Sync + Send>> = if config
        .use_console_log()
    {
        Box::pin(StdoutAppender::new())
    } else {
        let appender =
            RollingFileAppender::new(T::FILE_STEM, T::SIZE_LIMIT_GB, T::TIME_LIMIT_DAYS).await?;
        Box::pin(appender)
    };

    let appender = JsonAppender::new(appender);
    let appender = MetricsAppender::new(appender, metrics);

    Ok(appender)
}
