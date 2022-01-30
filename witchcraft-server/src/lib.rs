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

use crate::shutdown_hooks::ShutdownHooks;
use config::install::InstallConfig;
use config::runtime::RuntimeConfig;
use conjure_error::Error;
use conjure_object::Utc;
use conjure_runtime::{Agent, ClientFactory, HostMetricsRegistry, UserAgent};
use futures_util::{stream, Stream, StreamExt};
use refreshable::Refreshable;
use serde::de::DeserializeOwned;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::signal::unix::{self, SignalKind};
use tokio::{pin, runtime, select, time};
pub use witchcraft::Witchcraft;
use witchcraft_log::{fatal, info};
use witchcraft_metrics::MetricRegistry;
#[doc(inline)]
pub use witchcraft_server_config as config;

mod configs;
mod endpoint;
mod logging;
mod metrics;
mod server;
mod service;
mod shutdown_hooks;
mod witchcraft;

/// Initializes a Witchcraft server.
///
/// `init` is invoked with the parsed install and runtime configs as well as the [`Witchcraft`] context object. It
/// is expected to return quickly; any long running initialization should be spawned off into the background to run
/// asynchronously.
pub fn init<I, R, F>(init: F)
where
    I: AsRef<InstallConfig> + DeserializeOwned,
    R: AsRef<RuntimeConfig> + DeserializeOwned + PartialEq + 'static + Sync + Send,
    F: FnOnce(I, Refreshable<R, Error>, &mut Witchcraft) -> Result<(), Error>,
{
    match init_inner(init) {
        Ok(()) => (),
        Err(e) => {
            // we don't know if logging has been initialized, so both log and print the error.
            fatal!("error starting server", error: e);
            eprintln!(
                "[{}] - {} safe: {:?} unsafe: {:?}",
                Utc::now(),
                e.cause(),
                e.safe_params(),
                e.unsafe_params(),
            );
            for backtrace in e.backtraces() {
                eprintln!("{:?}", backtrace);
            }
            process::exit(1)
        }
    }
}

fn init_inner<I, R, F>(init: F) -> Result<(), Error>
where
    I: AsRef<InstallConfig> + DeserializeOwned,
    R: AsRef<RuntimeConfig> + DeserializeOwned + PartialEq + 'static + Sync + Send,
    F: FnOnce(I, Refreshable<R, Error>, &mut Witchcraft) -> Result<(), Error>,
{
    let install_config = configs::load_install::<I>()?;

    let thread_id = AtomicUsize::new(0);
    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(move || format!("runtime-{}", thread_id.fetch_add(1, Ordering::Relaxed)))
        .worker_threads(install_config.as_ref().server().io_threads())
        .thread_keep_alive(install_config.as_ref().server().idle_thread_timeout())
        .build()
        .map_err(Error::internal_safe)?;

    let handle = runtime.handle().clone();
    // ensure that we exit quickly even if there are stuck blocking tasks
    let _runtime = QuickDropRuntime {
        runtime: Some(runtime),
    };

    let runtime_config = configs::load_runtime::<R>(&handle)?;

    let metrics = Arc::new(MetricRegistry::new());

    let mut shutdown_hooks = ShutdownHooks::new();

    let loggers = handle.block_on(logging::init(
        &metrics,
        install_config.as_ref(),
        &runtime_config.map(|c| c.as_ref().logging().clone()),
        &mut shutdown_hooks,
    ))?;

    info!("server starting");

    metrics::init(&metrics);

    let host_metrics = Arc::new(HostMetricsRegistry::new());

    let mut client_factory =
        ClientFactory::new(runtime_config.map(|c| c.as_ref().service_discovery().clone()));
    client_factory
        .user_agent(UserAgent::new(Agent::new(
            install_config.as_ref().product_name(),
            install_config.as_ref().product_version(),
        )))
        .metrics(metrics.clone())
        .host_metrics(host_metrics)
        .blocking_handle(handle.clone());

    let mut witchcraft = Witchcraft {
        metrics,
        client_factory,
        handle: handle.clone(),
    };

    let base_install_config = install_config.as_ref().clone();

    init(install_config, runtime_config, &mut witchcraft)?;

    handle.block_on(server::start(
        &base_install_config,
        &mut witchcraft,
        loggers.request_logger,
    ))?;

    handle.block_on(shutdown(
        shutdown_hooks,
        base_install_config.server().shutdown_timeout(),
    ))
}

async fn shutdown(shutdown_hooks: ShutdownHooks, timeout: Duration) -> Result<(), Error> {
    pin! {
        let signals = signals()?;
    }

    signals.next().await;
    info!("server shutting down");

    select! {
        _ = shutdown_hooks => {}
        _ = signals.next() => {}
        _ = time::sleep(timeout) => {}
    }

    Ok(())
}

fn signals() -> Result<impl Stream<Item = ()>, Error> {
    let sigint = signal(SignalKind::interrupt())?;
    let sigterm = signal(SignalKind::terminate())?;
    Ok(stream::select(sigint, sigterm))
}

fn signal(kind: SignalKind) -> Result<impl Stream<Item = ()>, Error> {
    let mut signal = unix::signal(kind).map_err(Error::internal_safe)?;
    Ok(stream::poll_fn(move |cx| signal.poll_recv(cx)))
}

struct QuickDropRuntime {
    runtime: Option<Runtime>,
}

impl Drop for QuickDropRuntime {
    fn drop(&mut self) {
        self.runtime.take().unwrap().shutdown_background()
    }
}
