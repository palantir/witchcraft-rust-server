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
//! A highly opinionated embedded application server for RESTy APIs.
//!
//! # Initialization
//!
//! The entrypoint of a Witchcraft server is an initialization function annotated with `#[witchcraft_server::main]`:
//!
//! ```ignore
//! use conjure_error::Error;
//! use refreshable::Refreshable;
//! use witchcraft_server::config::install::InstallConfig;
//! use witchcraft_server::config::runtime::RuntimeConfig;
//!
//! #[witchcraft_server::main]
//! fn main(
//!     install: InstallConfig,
//!     runtime: Refreshable<RuntimeConfig>,
//!     wc: &mut Witchcraft,
//! ) -> Result<(), Error> {
//!     wc.api(CustomApiEndpoints::new(CustomApiResource));
//!
//!     Ok(())
//! }
//! ```
//!
//! The function is provided with the server's install and runtime configuration, as well as the [`Witchcraft`] object
//! which can be used to configure the server. Once the initialization function returns, the server will start.
//!
//! ## Note
//!
//! The initialization function is expected to return quickly - any long-running work required should happen in the
//! background.
//!
//! # Configuration
//!
//! Witchcraft divides configuration into two categories:
//!
//! * *Install* - Configuration that is fixed for the lifetime of a service. For example, the port that the server
//!     listens on is part of the server's install configuration.
//! * *Runtime* - Configuration that can dynamically update while the service is running. For example, the logging
//!     verbosity level is part of the server's runtime configuration.
//!
//! Configuration is loaded from the `var/conf/install.yml` and `var/conf/runtime.yml` files respectively. The
//! `runtime.yml` file is automatically checked for updates every few seconds.
//!
//! ## Extension
//!
//! The configuration files are deserialized into Rust types via the [`serde::Deserialize`] trait. `witchcraft-server`'s
//! own internal configuration is represented by the [`InstallConfig`] and [`RuntimeConfig`] types. Services that need
//! their own configuration should embed the Witchcraft configuration within their own using the `#[serde(flatten)]`
//! annotation and implement the [`AsRef`] trait:
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
//! The service's custom configuration will then sit next to the standard Witchcraft configuration:
//!
//! ```yml
//! product-name: my-service
//! product-version: 1.0.0
//! port: 12345
//! shave-yaks: true
//! ```
//!
//! ## Sensitive values
//!
//! The server's configuration deserializer uses [`serde_encrypted_value`] to transparently decrypt values in
//! in the configuration files using the key stored in `var/conf/encrypted-config-value.key`.
//!
//! ## Refreshable runtime configuration
//!
//! The server's runtime configuration is wrapped in the [`Refreshable`] type to allow code to properly handle updates
//! to the configuration. Depending on the use case, implementations can use the [`Refreshable::get`] to retrieve the
//! current state of the configuration when needed or the [`Refreshable::subscribe`] method to be notified of changes
//! to the configuration as they happen. See the documentation of the [`refreshable`] crate for more details.
//!
//! # HTTP APIs
//!
//! The server supports HTTP endpoints implementing the [`Service`] and [`AsyncService`]
//! traits. These implementations can be generated from a [Conjure] YML [definition] with the [`conjure-codegen`] crate.
//!
//! While we strongly encourage the use of Conjure-generated APIs, some services may need to expose endpoints that can't
//! be defined in Conjure. The [`conjure_http::conjure_endpoints`] macro can be used to define arbitrary HTTP endpoints.
//!
//! API endpoints should normally be registered with the [`Witchcraft::api`] and [`Witchcraft::blocking_api`] methods,
//! which will place the endpoints under the `/api` route. If necessary, the [`Witchcraft::app`] and
//! [`Witchcraft::blocking_app`] methods can be used to place the endpoints directly at the root route instead.
//!
//! [`Service`]: conjure_http::server::Service
//! [Conjure]: https://github.com/palantir/conjure
//! [definition]: https://palantir.github.io/conjure/#/docs/spec/conjure_definitions
//! [`conjure-codegen`]: https://docs.rs/conjure-codegen
//!
//! # HTTP clients
//!
//! Remote services are configured in the `service-discovery` section of the runtime configuration, and clients can be
//! created from the [`ClientFactory`] returned by the [`Witchcraft::client_factory`] method. The clients will
//! automatically update based on changes to the runtime configuration. See the documentation of the [`conjure_runtime`]
//! crate for more details.
//!
//! # Status endpoints
//!
//! The server exposes several "status" endpoints to report various aspects of the server.
//!
//! ## Liveness
//!
//! The `/status/liveness` endpoint returns a successful response to all requests, indicating that the server is alive.
//!
//! ## Readiness
//!
//! The `/status/readiness` endpoint returns a response indicating the server's readiness to handle requests to its
//! endpoints. Deployment infrastructure uses the result of this endpoint to decide if requests should be routed to a
//! given instance of the service. Custom readiness checks can be added to the server via the [`ReadinessCheckRegistry`]
//! returned by the [`Witchcraft::readiness_checks`] method. Any long-running initialization logic should happen
//! asynchronously and use a readiness check to indicate completion.
//!
//! ## Health
//!
//! The `/status/health` endpoint returns a response indicating the server's overall health. Deployment infrastructure
//! uses the result of this endpoint to trigger alerts. Custom health checks can be added to the server via the
//! [`HealthCheckRegistry`] returned by the [`Witchcraft::health_checks`] method. Requests to this endpoint must be
//! authenticated with the `health-checks.shared-secret` bearer token in runtime configuration.
//!
//! The server registers several built-in health checks:
//!
//! * `CONFIG_RELOAD` - Reports an error state if the runtime configuration failed to reload properly.
//! * `ENDPOINT_FIVE_HUNDREDS` - Reports a warning if an endpoint has a high rate of `500 Internal Server Error`
//!     responses.
//! * `SERVICE_DEPENDENCY` - Tracks the status of requests made with HTTP clients created via the server's client
//!     factory, and reports a warning state of requests to a remote service have a high failure rate.
//! * `PANICS` - Reports a warning if the server has panicked at any point.
//!
//! # Diagnostics
//!
//! The `/debug/diagnostic/{diagnosticType}` endpoint returns diagnostic information. Requests to this endpoint must be
//! authenticated with the `diagnostics.debug-shared-secret` bearer token in the runtime configuration.
//!
//! Several diagnostic types are defined:
//!
//! * `diagnostic.types.v1` - Returns a JSON-encoded list of all valid diagnostic types.
//! * `rust.heap.status.v1` - Returns detailed statistics about the state of the heap. Requires the `jemalloc` feature
//!     (enabled by default).
//! * `metric.names.v1` - Returns a JSON-encoded list of the names of all metrics registered with the server.
//! * `rust.thread.dump.v1` - Returns a stack trace of every thread in the process. Only supported when running on
//!     Linux.
//!
//! # Logging
//!
//! `witchcraft-server` emits JSON-encoded logs following the [witchcraft-api spec]. By default, logs will be written to
//! a file in `var/log` corresponding to the type of log message (`service.log`, `request.log`, etc). These files are
//! automatically rotated and compressed based on a non-configurable policy. If running in a Docker container or if the
//! `use-console-log` setting is enabled in the install configuration, logs will instead be written to standard out.
//!
//! [witchcraft-api spec]: https://github.com/palantir/witchcraft-api
//!
//! ## Service
//!
//! The service log contains the messages emitted by invocations of the macros in the [`witchcraft_log`] crate. Messages
//! emitted by the standard Rust [`log`] crate are additionally captured, but code that is written as part of a
//! Witchcraft service should use [`witchcraft_log`] instead for better integration. See the documentation of that crate
//! for more details.
//!
//! ## Request
//!
//! The request log records an entry for each HTTP request processed by the server. Parameters marked marked as safe by
//! an endpoint's Conjure definition will be included as parameters in the log record.
//!
//! ## Trace
//!
//! The trace log records [Zipkin]-style trace spans. The server automatically creates spans for each incoming HTTP
//! request based off of request's propagation metadata. Traces that have not alread had a sampling decision made will
//! be sampled at the rate specified by the `logging.trace-rate` field in the server's runtime configuration, which
//! defaults to 0.005%. Server logic can create additional spans with the [`zipkin`] crate. See the documentation of
//! that crate for more details.
//!
//! [Zipkin]: https://zipkin.io/
//!
//! ## Metric
//!
//! The metric log contains the values of metrics reporting the state of various components of the server. Metrics are
//! recorded every 30 seconds. Server logic can create additional metrics with the [`MetricRegistry`] returned by the
//! [`Witchcraft::metrics`] method. See the documentation of the [`witchcraft_metrics`] crate for more details.
//!
//! # Metrics
//!
//! The server reports a variety of metrics by default:
//!
//! ## Thread Pool
//!
//! * `server.worker.max` (gauge) - The configured maximum size of the server's thread pool used for requests to
//!     blocking endpoints.
//! * `server.worker.active` (gauge) - The number of threads actively processing requests to blocking endpoints.
//! * `server.worker.utilization-max` (gauge) - `server.worker.active` divided by `server.worker.max`. If this is 1, the
//!     server will immediately reject calls to blocking endpoints with a `503 Service Unavailable` status code.
//!
//! ## Logging
//!
//! * `logging.queue (type: <log_type>)` (gauge) - The number of log messages queued for output.
//!
//! ## Process
//!
//! * `process.heap` (gauge) - The total number of bytes allocated from the heap. Requires the `jemalloc` feature
//!     (enabled by default).
//! * `process.heap.active` (gauge) - The total number of bytes in active pages. Requires the `jemalloc` feature
//!     (enabled by default).
//! * `process.heap.resident` (gauge) - The total number of bytes in physically resident pages. Requires the `jemalloc` feature
//!     (enabled by default).
//! * `process.uptime` (gauge) - The number of microseconds that have elapsed since the server started.
//! * `process.panics` (counter) - The number of times the server has panicked.
//! * `process.user-time` (gauge) - The number of microseconds the process has spent running in user-space.
//! * `process.user-time.norm` (gauge) - `process.user-time` divided by the number of CPU cores.
//! * `process.system-time` (gauge) - The number of microseconds the process has spent either running in kernel-space
//!     or in uninterruptable IO wait.
//! * `process.system-time.norm` (gauge) - `process.system-time` divided by the number of CPU cores.
//! * `process.blocks-read` (gauge) - The number of filesystem blocks the server has read.
//! * `process.blocks-written` (gauge) - The number of filesystem blocks the server has written.
//! * `process.threads` (gauge) - The number of threads in the process.
//! * `process.filedescriptor` (gauge) - The number of file descriptors held open by the process divided by the maximum
//!     number of files the server may hold open.
//!
//! ## Connection
//!
//! * `server.connection.active` (counter) - The number of TCP sockets currently connected to the HTTP server.
//! * `server.connection.utilization` (gauge) - `server.connection.active` divided by the maximum number of connections
//!     the server will accept.
//!
//! ## TLS
//!
//! * `tls.handshake (context: server, protocol: <protocol>, cipher: <cipher>)` (meter) - The rate of TLS handshakes
//!     completed by the HTTP server.
//!
//! ## Server
//!
//! * `server.request.active` (counter) - The number of requests being actively processed.
//! * `server.request.unmatched` (meter) - The rate of `404 Not Found` responses returned by the server.
//! * `server.response.all` (meter) - The rate of responses returned by the server.
//! * `server.response.1xx` (meter) - The rate of `1xx` responses returned by the server.
//! * `server.response.2xx` (meter) - The rate of `2xx` responses returned by the server.
//! * `server.response.3xx` (meter) - The rate of `3xx` responses returned by the server.
//! * `server.response.4xx` (meter) - The rate of `4xx` responses returned by the server.
//! * `server.response.5xx` (meter) - The rate of `5xx` responses returned by the server.
//! * `server.response.500` (meter) - The rate of `500 Internal Server Error` responses returned by the server.
//!
//! ## Endpoints
//!
//! * `server.response (service-name: <service_name>, endpoint: <endpoint>)` (timer) - The amount of time required to
//!     process each request to the endpoint, including sending the entire response body.
//! * `server.response.error (service-name: <service_name>, endpoint: <endpoint>)` (meter) - The rate of `5xx` errors
//!     returned for requests to the endpoint.
//!
//! ## HTTP clients
//!
//! See the documentation of the [`conjure_runtime`] crate for the metrics reported by HTTP clients.
#![warn(missing_docs)]

use std::env;
use std::process;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use conjure_error::Error;
use conjure_http::server::{AsyncService, ConjureRuntime};
use conjure_runtime::{Agent, ClientFactory, HostMetricsRegistry, UserAgent};
use futures_util::{stream, Stream, StreamExt};
use refreshable::Refreshable;
use serde::de::DeserializeOwned;
use tokio::runtime::{Handle, Runtime};
use tokio::signal::unix::{self, SignalKind};
use tokio::{pin, runtime, select, time};
use witchcraft_log::{fatal, info};
use witchcraft_metrics::MetricRegistry;

pub use body::{RequestBody, ResponseWriter};
use config::install::InstallConfig;
use config::runtime::RuntimeConfig;
pub use witchcraft::Witchcraft;
#[doc(inline)]
pub use witchcraft_server_config as config;
#[doc(inline)]
pub use witchcraft_server_macros::main;

use crate::debug::diagnostic_types::DiagnosticTypesDiagnostic;
use crate::debug::endpoint::DebugEndpoints;
#[cfg(feature = "jemalloc")]
use crate::debug::heap_stats::HeapStatsDiagnostic;
use crate::debug::metric_names::MetricNamesDiagnostic;
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
use crate::debug::thread_dump::ThreadDumpDiagnostic;
use crate::debug::DiagnosticRegistry;
use crate::health::config_reload::ConfigReloadHealthCheck;
use crate::health::endpoint_500s::Endpoint500sHealthCheck;
use crate::health::panics::PanicsHealthCheck;
use crate::health::service_dependency::ServiceDependencyHealthCheck;
use crate::health::HealthCheckRegistry;
use crate::readiness::ReadinessCheckRegistry;
use crate::server::Listener;
use crate::shutdown_hooks::ShutdownHooks;
use crate::status::StatusEndpoints;

pub mod blocking;
mod body;
mod configs;
pub mod debug;
mod endpoint;
pub mod extensions;
pub mod health;
pub mod logging;
mod metrics;
mod minidump;
pub mod readiness;
mod server;
mod service;
mod shutdown_hooks;
mod status;
pub mod tls;
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
    init_with_configs(init, configs::load_install::<I>, configs::load_runtime::<R>)
}

/// Initializes a Witchcraft server with custom config loaders.
///
/// `init` is invoked with the install and runtime configs from the provided loaders as well as the [`Witchcraft`]
/// context object. It is expected to return quickly; any long running initialization should be spawned off into
/// the background to run asynchronously.
pub fn init_with_configs<I, R, F, LI, LR>(init: F, load_install: LI, load_runtime: LR)
where
    I: AsRef<InstallConfig> + DeserializeOwned,
    R: AsRef<RuntimeConfig> + DeserializeOwned + PartialEq + 'static + Sync + Send,
    F: FnOnce(I, Refreshable<R, Error>, &mut Witchcraft) -> Result<(), Error>,
    LI: FnOnce() -> Result<I, Error>,
    LR: FnOnce(&Handle, &Arc<AtomicBool>) -> Result<Refreshable<R, Error>, Error>,
{
    let mut runtime_guard = None;

    let ret = match init_inner(init, load_install, load_runtime, &mut runtime_guard) {
        Ok(()) => 0,
        Err(e) => {
            fatal!("error starting server", error: e);
            1
        }
    };
    drop(runtime_guard);

    process::exit(ret);
}

fn init_inner<I, R, F, LI, LR>(
    init: F,
    load_install: LI,
    load_runtime: LR,
    runtime_guard: &mut Option<RuntimeGuard>,
) -> Result<(), Error>
where
    I: AsRef<InstallConfig> + DeserializeOwned,
    R: AsRef<RuntimeConfig> + DeserializeOwned + PartialEq + 'static + Sync + Send,
    F: FnOnce(I, Refreshable<R, Error>, &mut Witchcraft) -> Result<(), Error>,
    LI: FnOnce() -> Result<I, Error>,
    LR: FnOnce(&Handle, &Arc<AtomicBool>) -> Result<Refreshable<R, Error>, Error>,
{
    if env::args_os().nth(1).map_or(false, |a| a == "minidump") {
        return minidump::server();
    }

    logging::early_init();

    let install_config = load_install()?;

    let thread_id = AtomicUsize::new(0);
    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(move || format!("runtime-{}", thread_id.fetch_add(1, Ordering::Relaxed)))
        .worker_threads(install_config.as_ref().server().io_threads())
        .thread_keep_alive(install_config.as_ref().server().idle_thread_timeout())
        .build()
        .map_err(Error::internal_safe)?;

    let handle = runtime.handle().clone();
    let runtime = runtime_guard.insert(RuntimeGuard {
        runtime: Some(runtime),
        logger_shutdown: Some(ShutdownHooks::new()),
    });

    let runtime_config_ok = Arc::new(AtomicBool::new(true));
    let runtime_config = load_runtime(&handle, &runtime_config_ok)?;

    let metrics = Arc::new(MetricRegistry::new());

    let loggers = handle.block_on(logging::init(
        &metrics,
        install_config.as_ref(),
        &runtime_config.map(|c| c.as_ref().logging().clone()),
        runtime.logger_shutdown.as_mut().unwrap(),
    ))?;

    info!("server starting");

    handle.block_on(minidump::init())?;

    metrics::init(&metrics);

    let host_metrics = Arc::new(HostMetricsRegistry::new());

    let health_checks = Arc::new(HealthCheckRegistry::new(&handle));
    health_checks.register(ServiceDependencyHealthCheck::new(&host_metrics));
    health_checks.register(PanicsHealthCheck::new());
    health_checks.register(ConfigReloadHealthCheck::new(runtime_config_ok));

    let readiness_checks = Arc::new(ReadinessCheckRegistry::new());

    let diagnostics = Arc::new(DiagnosticRegistry::new());
    diagnostics.register(MetricNamesDiagnostic::new(&metrics));
    #[cfg(feature = "jemalloc")]
    diagnostics.register(HeapStatsDiagnostic);
    #[cfg(target_os = "linux")]
    diagnostics.register(ThreadDumpDiagnostic);
    diagnostics.register(DiagnosticTypesDiagnostic::new(Arc::downgrade(&diagnostics)));
    let client_factory = ClientFactory::builder()
        .config(runtime_config.map(|c| c.as_ref().service_discovery().clone()))
        .user_agent(UserAgent::new(Agent::new(
            install_config.as_ref().product_name(),
            install_config.as_ref().product_version(),
        )))
        .metrics(metrics.clone())
        .host_metrics(host_metrics)
        .blocking_handle(handle.clone());

    let mut witchcraft = Witchcraft {
        metrics,
        health_checks,
        readiness_checks,
        client_factory,
        diagnostics: diagnostics.clone(),
        handle: handle.clone(),
        install_config: install_config.as_ref().clone(),
        thread_pool: None,
        endpoints: vec![],
        shutdown_hooks: ShutdownHooks::new(),
        conjure_runtime: Arc::new(ConjureRuntime::new()),
    };

    let status_endpoints = StatusEndpoints::new(
        &runtime_config,
        &witchcraft.health_checks,
        &witchcraft.readiness_checks,
    );
    witchcraft.endpoints(
        None,
        status_endpoints.endpoints(&witchcraft.conjure_runtime),
        false,
    );

    let debug_endpoints = DebugEndpoints::new(&runtime_config, diagnostics);
    witchcraft.app(debug_endpoints);

    // server::start clears out the previously-registered endpoints so the existing Witchcraft
    // is ready to reuse for the main port afterwards.
    if let Some(management_port) = install_config.as_ref().management_port() {
        if management_port != install_config.as_ref().port() {
            handle.block_on(server::start(
                &mut witchcraft,
                &loggers,
                Listener::Management,
                management_port,
            ))?;
        }
    }

    init(install_config, runtime_config, &mut witchcraft)?;

    witchcraft
        .health_checks
        .register(Endpoint500sHealthCheck::new(&witchcraft.endpoints));

    let port = witchcraft.install_config.port();
    handle.block_on(server::start(
        &mut witchcraft,
        &loggers,
        Listener::Service,
        port,
    ))?;

    handle.block_on(shutdown(
        witchcraft.shutdown_hooks,
        witchcraft.install_config.server().shutdown_timeout(),
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
        _ = signals.next() => info!("graceful shutdown interrupted by signal"),
        _ = time::sleep(timeout) => {
            info!(
                "graceful shutdown timed out",
                safe: {
                    timeout: format_args!("{timeout:?}"),
                },
            );
        }
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

struct RuntimeGuard {
    runtime: Option<Runtime>,
    logger_shutdown: Option<ShutdownHooks>,
}

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        let runtime = self.runtime.take().unwrap();
        runtime.block_on(self.logger_shutdown.take().unwrap());
        runtime.shutdown_background()
    }
}
