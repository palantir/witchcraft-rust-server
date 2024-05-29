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
use conjure_http::server::EndpointMetadata;
use http::{Request, Response};
use http_body::{Body, Frame};
use pin_project::{pin_project, pinned_drop};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::time::Instant;
use witchcraft_metrics::{Meter, MetricId, MetricRegistry, Timer};

#[derive(Clone)]
pub struct EndpointMetrics {
    response: Arc<Timer>,
    response_error: Arc<Meter>,
}

impl EndpointMetrics {
    pub fn new(metrics: &MetricRegistry, endpoint: &dyn EndpointMetadata) -> Self {
        EndpointMetrics {
            response: metrics.timer(
                MetricId::new("server.response")
                    .with_tag("service-name", endpoint.service_name().to_string())
                    .with_tag("endpoint", endpoint.name().to_string()),
            ),
            response_error: metrics.meter(
                MetricId::new("server.response.error")
                    .with_tag("service-name", endpoint.service_name().to_string())
                    .with_tag("endpoint", endpoint.name().to_string()),
            ),
        }
    }
}

/// A layer which records endpoint-specific metrics.
///
/// It must be installed after routing.
pub struct EndpointMetricsLayer;

impl<S> Layer<S> for EndpointMetricsLayer {
    type Service = EndpointMetricsService<S>;

    fn layer(self, inner: S) -> Self::Service {
        EndpointMetricsService { inner }
    }
}

pub struct EndpointMetricsService<S> {
    inner: S,
}

impl<S, B1, B2> Service<Request<B1>> for EndpointMetricsService<S>
where
    S: Service<Request<B1>, Response = Response<B2>> + Sync,
    B1: Send,
{
    type Response = Response<EndpointMetricsBody<B2>>;

    async fn call(&self, req: Request<B1>) -> Self::Response {
        let endpoint_metrics = match req
            .extensions()
            .get::<Route>()
            .expect("Route missing from request extensions")
        {
            Route::Resolved(endpoint) => endpoint.metrics().cloned(),
            _ => None,
        };

        let start_time = Instant::now();
        let response = self.inner.call(req).await;
        if response.status().is_server_error() {
            if let Some(metrics) = &endpoint_metrics {
                metrics.response_error.mark(1);
            }
        }

        response.map(|inner| EndpointMetricsBody {
            inner,
            start_time,
            response: endpoint_metrics.map(|m| m.response.clone()),
        })
    }
}

#[pin_project(PinnedDrop)]
pub struct EndpointMetricsBody<B> {
    #[pin]
    inner: B,
    start_time: Instant,
    response: Option<Arc<Timer>>,
}

#[pinned_drop]
impl<B> PinnedDrop for EndpointMetricsBody<B> {
    fn drop(self: Pin<&mut Self>) {
        if let Some(response) = &self.response {
            response.update(self.start_time.elapsed());
        }
    }
}

impl<B> Body for EndpointMetricsBody<B>
where
    B: Body,
{
    type Data = B::Data;

    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        self.project().inner.poll_frame(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}
