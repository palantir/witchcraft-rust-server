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
use crate::service::peer_addr::GetPeerAddr;
use crate::service::{Layer, Service};
use conjure_error::Error;
use pin_project::pin_project;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use witchcraft_server_config::install::InstallConfig;

/// A layer which limits the number of active connections by throttling calls to downstream services.
pub struct ConnectionLimitLayer {
    semaphore: Arc<Semaphore>,
}

impl ConnectionLimitLayer {
    pub fn new(config: &InstallConfig) -> Self {
        ConnectionLimitLayer {
            semaphore: Arc::new(Semaphore::new(config.server().max_connections())),
        }
    }
}

impl<S> Layer<S> for ConnectionLimitLayer {
    type Service = ConnectionLimitService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ConnectionLimitService {
            inner: Arc::new(inner),
            semaphore: self.semaphore,
        }
    }
}

pub struct ConnectionLimitService<S> {
    inner: Arc<S>,
    semaphore: Arc<Semaphore>,
}

impl<S, R> Service<R> for ConnectionLimitService<S>
where
    S: Service<R>,
{
    type Response = ConnectionLimitStream<S::Response>;

    async fn call(&self, req: R) -> Self::Response {
        let permit = self.semaphore.acquire().await;
        let inner = self.inner.call(req).await;
        ConnectionLimitStream { inner, permit }
    }
}

#[pin_project]
pub struct ConnectionLimitStream<S> {
    #[pin]
    inner: S,
    permit: OwnedSemaphorePermit,
}

impl<S> AsyncRead for ConnectionLimitStream<S>
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

impl<S> AsyncWrite for ConnectionLimitStream<S>
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

impl<S> GetPeerAddr for ConnectionLimitStream<S>
where
    S: GetPeerAddr,
{
    fn peer_addr(&self) -> Result<SocketAddr, Error> {
        self.inner.peer_addr()
    }
}
