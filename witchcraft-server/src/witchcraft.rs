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
use crate::blocking::conjure::ConjureBlockingEndpoint;
use crate::blocking::pool::ThreadPool;
use crate::debug::DiagnosticRegistry;
use crate::endpoint::conjure::ConjureEndpoint;
use crate::endpoint::extended_path::ExtendedPathEndpoint;
use crate::endpoint::WitchcraftEndpoint;
use crate::health::HealthCheckRegistry;
use crate::readiness::ReadinessCheckRegistry;
use crate::shutdown_hooks::ShutdownHooks;
use crate::{blocking, RequestBody, ResponseWriter};
use conjure_error::Error;
use conjure_http::server::{AsyncService, BoxAsyncEndpoint, ConjureRuntime, Endpoint, Service};
use conjure_runtime::ClientFactory;
use futures_util::Future;
use http::HeaderValue;
use itertools::Itertools;
use std::sync::Arc;
use tokio::runtime::Handle;
use witchcraft_metrics::MetricRegistry;
use witchcraft_server_config::install::InstallConfig;

/// A client-side node selection strategy that a server can recommend for incoming requests.
///
/// See [`Witchcraft::set_recommended_node_selection_strategies`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NodeSelectionStrategy {
    /// Distributes requests across nodes based on their load.
    Balanced,
    /// Pins requests to a node until an error, periodically reshuffling node order.
    PinUntilError,
    /// Pins requests to a node until an error, without periodically reshuffling node order.
    PinUntilErrorWithoutReshuffle,
}

impl NodeSelectionStrategy {
    fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "BALANCED",
            Self::PinUntilError => "PIN_UNTIL_ERROR",
            Self::PinUntilErrorWithoutReshuffle => "PIN_UNTIL_ERROR_WITHOUT_RESHUFFLE",
        }
    }
}

/// The Witchcraft server context.
pub struct Witchcraft {
    pub(crate) metrics: Arc<MetricRegistry>,
    pub(crate) health_checks: Arc<HealthCheckRegistry>,
    pub(crate) readiness_checks: Arc<ReadinessCheckRegistry>,
    pub(crate) diagnostics: Arc<DiagnosticRegistry>,
    pub(crate) client_factory: ClientFactory,
    pub(crate) handle: Handle,
    pub(crate) install_config: InstallConfig,
    pub(crate) thread_pool: Option<Arc<ThreadPool>>,
    pub(crate) thread_prefix: Option<String>,
    pub(crate) endpoints: Vec<Box<dyn WitchcraftEndpoint + Sync + Send>>,
    pub(crate) shutdown_hooks: ShutdownHooks,
    pub(crate) conjure_runtime: Arc<ConjureRuntime>,
    pub(crate) recommended_node_selection_strategies: Option<HeaderValue>,
}

impl Witchcraft {
    /// Returns a reference to the server's metric registry.
    #[inline]
    pub fn metrics(&self) -> &Arc<MetricRegistry> {
        &self.metrics
    }

    /// Returns a reference to the server's health check registry.
    #[inline]
    pub fn health_checks(&self) -> &Arc<HealthCheckRegistry> {
        &self.health_checks
    }

    /// Returns a reference to the server's readiness check registry.
    #[inline]
    pub fn readiness_checks(&self) -> &Arc<ReadinessCheckRegistry> {
        &self.readiness_checks
    }

    /// Returns a reference to the server's HTTP client factory.
    #[inline]
    pub fn client_factory(&self) -> &ClientFactory {
        &self.client_factory
    }

    /// Returns a reference to the server's diagnostics registry.
    #[inline]
    pub fn diagnostics(&self) -> &Arc<DiagnosticRegistry> {
        &self.diagnostics
    }

    /// Returns a reference to a handle to the server's Tokio runtime.
    #[inline]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Sets the node selection strategies recommended to clients calling this server, in preference order.
    ///
    /// Call this during initialization to add a `Node-Selection-Strategy` header to responses from both the service
    /// and management listeners, including error responses. By default, no recommendation header is added. Calling
    /// this method again replaces the previous recommendation.
    ///
    /// This recommends routing for incoming requests; it does not configure the server's outbound Conjure clients.
    /// Clients may ignore the recommendation. Compatible clients select the first strategy they support according to
    /// the [server-recommended node selection protocol].
    ///
    /// Returns an error if `strategies` is empty, leaving any previous recommendation unchanged.
    ///
    /// ```
    /// use witchcraft_server::{NodeSelectionStrategy, Witchcraft};
    /// # fn init(wc: &mut Witchcraft) -> Result<(), conjure_error::Error> {
    /// wc.set_recommended_node_selection_strategies([NodeSelectionStrategy::Balanced])?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [server-recommended node selection protocol]: https://github.com/palantir/dialogue#server-recommended-node-selection-strategies
    pub fn set_recommended_node_selection_strategies(
        &mut self,
        strategies: impl IntoIterator<Item = NodeSelectionStrategy>,
    ) -> Result<(), Error> {
        self.recommended_node_selection_strategies =
            Some(node_selection_strategy_header(strategies)?);
        Ok(())
    }

    /// Installs an async service at the server's root.
    pub fn app<T>(&mut self, service: T)
    where
        T: AsyncService<RequestBody, ResponseWriter>,
    {
        self.endpoints(None, service.endpoints(&self.conjure_runtime), true)
    }

    /// Installs an async service under the server's `/api` prefix.
    pub fn api<T>(&mut self, service: T)
    where
        T: AsyncService<RequestBody, ResponseWriter>,
    {
        self.endpoints(Some("/api"), service.endpoints(&self.conjure_runtime), true)
    }

    pub(crate) fn endpoints(
        &mut self,
        prefix: Option<&str>,
        endpoints: Vec<BoxAsyncEndpoint<'static, RequestBody, ResponseWriter>>,
        track_metrics: bool,
    ) {
        let metrics = if track_metrics {
            Some(&*self.metrics)
        } else {
            None
        };

        self.endpoints.extend(
            endpoints
                .into_iter()
                .map(|e| Box::new(ConjureEndpoint::new(metrics, e)))
                .map(|e| extend_path(e, self.install_config.context_path(), prefix)),
        )
    }

    /// Installs a blocking service at the server's root.
    pub fn blocking_app<T>(&mut self, service: T)
    where
        T: Service<blocking::RequestBody, blocking::ResponseWriter>,
    {
        self.blocking_endpoints(None, service.endpoints(&self.conjure_runtime))
    }

    /// Installs a blocking service under the server's `/api` prefix.
    pub fn blocking_api<T>(&mut self, service: T)
    where
        T: Service<blocking::RequestBody, blocking::ResponseWriter>,
    {
        self.blocking_endpoints(Some("/api"), service.endpoints(&self.conjure_runtime))
    }

    fn blocking_endpoints(
        &mut self,
        prefix: Option<&str>,
        endpoints: Vec<
            Box<dyn Endpoint<blocking::RequestBody, blocking::ResponseWriter> + Sync + Send>,
        >,
    ) {
        let thread_pool = self.thread_pool.get_or_insert_with(|| {
            Arc::new(ThreadPool::new(
                &self.install_config,
                &self.metrics,
                self.thread_prefix.clone(),
            ))
        });

        self.endpoints.extend(
            endpoints
                .into_iter()
                .map(|e| Box::new(ConjureBlockingEndpoint::new(&self.metrics, thread_pool, e)))
                .map(|e| extend_path(e, self.install_config.context_path(), prefix)),
        )
    }

    /// Adds a future that will be run when the server begins its shutdown process.
    ///
    /// The server will not shut down until the future completes or the configured shutdown timeout elapses.
    pub fn on_shutdown<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static + Send,
    {
        self.shutdown_hooks.push(future)
    }
}

fn node_selection_strategy_header(
    strategies: impl IntoIterator<Item = NodeSelectionStrategy>,
) -> Result<HeaderValue, Error> {
    let value = strategies
        .into_iter()
        .map(NodeSelectionStrategy::as_str)
        .join(",");
    if value.is_empty() {
        return Err(Error::internal_safe(
            "node selection strategies must not be empty",
        ));
    }
    HeaderValue::try_from(value).map_err(Error::internal_safe)
}

fn extend_path(
    endpoint: Box<dyn WitchcraftEndpoint + Sync + Send>,
    context_path: &str,
    prefix: Option<&str>,
) -> Box<dyn WitchcraftEndpoint + Sync + Send> {
    let context_path = if context_path == "/" {
        ""
    } else {
        context_path
    };
    let prefix = format!("{context_path}{}", prefix.unwrap_or(""));

    if prefix.is_empty() {
        endpoint
    } else {
        Box::new(ExtendedPathEndpoint::new(endpoint, &prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_node_selection_strategy() {
        assert_eq!(
            node_selection_strategy_header([NodeSelectionStrategy::Balanced]).unwrap(),
            "BALANCED",
        );
    }

    #[test]
    fn node_selection_strategies_preserve_preference_order() {
        assert_eq!(
            node_selection_strategy_header([
                NodeSelectionStrategy::PinUntilErrorWithoutReshuffle,
                NodeSelectionStrategy::Balanced,
                NodeSelectionStrategy::PinUntilError,
            ])
            .unwrap(),
            "PIN_UNTIL_ERROR_WITHOUT_RESHUFFLE,BALANCED,PIN_UNTIL_ERROR",
        );
    }

    #[test]
    fn empty_node_selection_strategies_are_rejected() {
        assert!(node_selection_strategy_header([]).is_err());
    }
}
