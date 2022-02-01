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
use conjure_runtime::ClientFactory;
use std::sync::Arc;
use tokio::runtime::Handle;
use witchcraft_metrics::MetricRegistry;

/// The Witchcraft server context.
pub struct Witchcraft {
    pub(crate) metrics: Arc<MetricRegistry>,
    pub(crate) client_factory: ClientFactory,
    pub(crate) handle: Handle,
}

impl Witchcraft {
    /// Returns a reference to the server's metric registry.
    #[inline]
    pub fn metrics(&self) -> &Arc<MetricRegistry> {
        &self.metrics
    }

    /// Returns a reference to the server's HTTP client factory.
    #[inline]
    pub fn client_factory(&self) -> &ClientFactory {
        &self.client_factory
    }

    /// Returns a reference to a handle to the server's Tokio runtime.
    #[inline]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}
