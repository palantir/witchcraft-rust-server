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
use crate::service::routing::Route;
use crate::service::{Layer, Service};
use http::{Request, Response};

/// A layer which updates the [`EndpointHealth`] state based on request outcomes.
///
/// It must be installed after routing.
pub struct EndpointHealthLayer;

impl<S> Layer<S> for EndpointHealthLayer {
    type Service = EndpointHealthService<S>;

    fn layer(self, inner: S) -> Self::Service {
        EndpointHealthService { inner }
    }
}

pub struct EndpointHealthService<S> {
    inner: S,
}

impl<S, B1, B2> Service<Request<B1>> for EndpointHealthService<S>
where
    S: Service<Request<B1>, Response = Response<B2>> + Sync,
    B1: Send,
{
    type Response = S::Response;

    async fn call(&self, req: Request<B1>) -> Self::Response {
        let metrics = match req
            .extensions()
            .get::<Route>()
            .expect("Route missing from request extensions")
        {
            Route::Resolved(endpoint) => endpoint.health().cloned(),
            _ => None,
        };

        let response = self.inner.call(req).await;
        if let Some(metrics) = metrics {
            metrics.mark(response.status());
        }
        response
    }
}
