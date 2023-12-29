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
use crate::service::hyper::NewConnection;
use crate::service::{Layer, Service};
use std::sync::Arc;
use tokio_rustls::server::TlsStream;
use witchcraft_metrics::{MetricId, MetricRegistry};

/// A layer which records metrics about TLS handshakes.
pub struct TlsMetricsLayer {
    metrics: Arc<MetricRegistry>,
}

impl TlsMetricsLayer {
    pub fn new(metrics: &Arc<MetricRegistry>) -> Self {
        TlsMetricsLayer {
            metrics: metrics.clone(),
        }
    }
}

impl<S> Layer<S> for TlsMetricsLayer {
    type Service = TlsMetricsService<S>;

    fn layer(self, inner: S) -> Self::Service {
        TlsMetricsService {
            inner,
            metrics: self.metrics,
        }
    }
}

pub struct TlsMetricsService<S> {
    inner: S,
    metrics: Arc<MetricRegistry>,
}

impl<S, R, L> Service<NewConnection<TlsStream<R>, L>> for TlsMetricsService<S>
where
    S: Service<NewConnection<TlsStream<R>, L>>,
{
    type Response = S::Response;

    async fn call(&self, req: NewConnection<TlsStream<R>, L>) -> Self::Response {
        let protocol = req
            .stream
            .get_ref()
            .1
            .protocol_version()
            .expect("session is active");

        let cipher = req
            .stream
            .get_ref()
            .1
            .negotiated_cipher_suite()
            .expect("session is active");

        self.metrics
            .meter(
                MetricId::new("tls.handshake")
                    .with_tag("context", "server")
                    .with_tag("protocol", protocol.as_str().unwrap_or("unknown"))
                    .with_tag("cipher", cipher.suite().as_str().unwrap_or("unknown")),
            )
            .mark(1);

        self.inner.call(req).await
    }
}
