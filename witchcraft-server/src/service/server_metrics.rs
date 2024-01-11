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
use crate::server::Listener;
use crate::service::{Layer, Service};
use http::{HeaderMap, Response, StatusCode};
use http_body::Body;
use pin_project::pin_project;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use witchcraft_metrics::{Counter, Meter, MetricId, MetricRegistry};

struct Metrics {
    request_active: Arc<Counter>,
    request_unmatched: Arc<Meter>,
    response_all: Arc<Meter>,
    response_xxx: [Arc<Meter>; 5],
    response_500: Arc<Meter>,
}

/// A layer that records global metrics about requests.
pub struct ServerMetricsLayer {
    metrics: Metrics,
}

impl ServerMetricsLayer {
    pub fn new(metrics: &MetricRegistry, listener: Listener) -> Self {
        ServerMetricsLayer {
            metrics: Metrics {
                request_active: metrics.counter(
                    MetricId::new("server.request.active").with_tag("listener", listener.tag()),
                ),
                request_unmatched: metrics.meter("server.request.unmatched"),
                response_all: metrics.meter("server.response.all"),
                response_xxx: [
                    metrics.meter("server.response.1xx"),
                    metrics.meter("server.response.2xx"),
                    metrics.meter("server.response.3xx"),
                    metrics.meter("server.response.4xx"),
                    metrics.meter("server.response.5xx"),
                ],
                response_500: metrics.meter("server.response.500"),
            },
        }
    }
}

impl<S> Layer<S> for ServerMetricsLayer {
    type Service = ServerMetricsService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ServerMetricsService {
            inner,
            metrics: self.metrics,
        }
    }
}

pub struct ServerMetricsService<S> {
    inner: S,
    metrics: Metrics,
}

impl<S, R, B> Service<R> for ServerMetricsService<S>
where
    S: Service<R, Response = Response<B>> + Sync,
    R: Send,
{
    type Response = Response<ServerMetricsBody<B>>;

    async fn call(&self, req: R) -> Self::Response {
        self.metrics.request_active.inc();
        let guard = ActiveGuard {
            request_active: self.metrics.request_active.clone(),
        };

        let response = self.inner.call(req).await;
        if response.status() == StatusCode::NOT_FOUND {
            self.metrics.request_unmatched.mark(1);
        }
        self.metrics.response_all.mark(1);
        if let Some(gauge) = self
            .metrics
            .response_xxx
            .get(response.status().as_u16() as usize / 100 - 1)
        {
            gauge.mark(1);
        }
        if response.status() == StatusCode::INTERNAL_SERVER_ERROR {
            self.metrics.response_500.mark(1);
        }

        response.map(|inner| ServerMetricsBody {
            inner,
            _guard: guard,
        })
    }
}

#[pin_project]
pub struct ServerMetricsBody<B> {
    #[pin]
    inner: B,
    _guard: ActiveGuard,
}

impl<B> Body for ServerMetricsBody<B>
where
    B: Body,
{
    type Data = B::Data;

    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.project().inner.poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        self.project().inner.poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

struct ActiveGuard {
    request_active: Arc<Counter>,
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.request_active.dec();
    }
}
