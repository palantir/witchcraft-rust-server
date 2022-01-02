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
use crate::health::HealthCheckRegistry;
use conjure_error::Error;
use conjure_runtime::ClientFactory;
use refreshable::Refreshable;
use std::sync::Arc;
use tokio::runtime::{Handle, Runtime};
use witchcraft_metrics::MetricRegistry;

/// The Witchcraft initialization context.
pub struct Witchcraft<I, R> {
    pub(crate) install_config: Arc<I>,
    pub(crate) runtime_config: Arc<Refreshable<R, Error>>,
    pub(crate) runtime: Runtime,
    pub(crate) metrics: Arc<MetricRegistry>,
    pub(crate) health_checks: Arc<HealthCheckRegistry>,
    pub(crate) client_factory: ClientFactory,
}

impl<I, R> Witchcraft<I, R> {
    /// Returns the server's install configuration.
    #[inline]
    pub fn install_config(&self) -> &Arc<I> {
        &self.install_config
    }

    /// Returns the server's refreshable runtime configuration.
    #[inline]
    pub fn runtime_config(&self) -> &Arc<Refreshable<R, Error>> {
        &self.runtime_config
    }

    /// Returns a handle to the server's Tokio [`Runtime`].
    #[inline]
    pub fn handle(&self) -> &Handle {
        self.runtime.handle()
    }

    /// Returns the server's metric registry.
    #[inline]
    pub fn metrics(&self) -> &Arc<MetricRegistry> {
        &self.metrics
    }

    /// Returns the server's health check registry.
    #[inline]
    pub fn health_checks(&self) -> &Arc<HealthCheckRegistry> {
        &self.health_checks
    }

    /// Returns the server's HTTP client factory.
    #[inline]
    pub fn client_factory(&self) -> &ClientFactory {
        &self.client_factory
    }
}
