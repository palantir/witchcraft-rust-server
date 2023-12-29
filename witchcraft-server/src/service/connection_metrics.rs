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
use crate::service::peer_addr::GetPeerAddr;
use crate::service::{Layer, Service};
use pin_project::{pin_project, pinned_drop};
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use witchcraft_metrics::{Counter, MetricId, MetricRegistry};
use witchcraft_server_config::install::InstallConfig;

/// A layer which tracks active connection metrics.
pub struct ConnectionMetricsLayer {
    active_connections: Arc<Counter>,
}

impl ConnectionMetricsLayer {
    pub fn new(config: &InstallConfig, metrics: &MetricRegistry, listener: Listener) -> Self {
        let active_connections = metrics.counter(
            MetricId::new("server.connection.active").with_tag("listener", listener.tag()),
        );

        metrics.gauge(
            MetricId::new("server.connection.utilization").with_tag("listener", listener.tag()),
            {
                let active_connections = active_connections.clone();
                let max_connections = config.server().max_connections();
                move || active_connections.count() as f64 / max_connections as f64
            },
        );

        ConnectionMetricsLayer { active_connections }
    }
}

impl<S> Layer<S> for ConnectionMetricsLayer {
    type Service = ConnectionMetricsService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ConnectionMetricsService {
            inner,
            active_connections: self.active_connections,
        }
    }
}

pub struct ConnectionMetricsService<S> {
    inner: S,
    active_connections: Arc<Counter>,
}

impl<S, R> Service<R> for ConnectionMetricsService<S>
where
    S: Service<R>,
{
    type Response = ConnectionMetricsStream<S::Response>;

    async fn call(&self, req: R) -> Self::Response {
        let inner = self.inner.call(req).await;
        self.active_connections.inc();

        ConnectionMetricsStream {
            inner,
            active_connections: self.active_connections.clone(),
        }
    }
}

#[pin_project(PinnedDrop)]
pub struct ConnectionMetricsStream<S> {
    #[pin]
    inner: S,
    active_connections: Arc<Counter>,
}

#[pinned_drop]
impl<S> PinnedDrop for ConnectionMetricsStream<S> {
    fn drop(self: Pin<&mut Self>) {
        self.active_connections.dec();
    }
}

impl<S> AsyncRead for ConnectionMetricsStream<S>
where
    S: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl<S> AsyncWrite for ConnectionMetricsStream<S>
where
    S: AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_shutdown(cx)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }
}

impl<S> GetPeerAddr for ConnectionMetricsStream<S>
where
    S: GetPeerAddr,
{
    fn peer_addr(&self) -> Result<std::net::SocketAddr, conjure_error::Error> {
        self.inner.peer_addr()
    }
}
