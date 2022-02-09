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
use crate::service::{Layer, Service};
use futures_util::ready;
use pin_project::pin_project;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::PollSemaphore;
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

    type Future = ConnectionLimitFuture<S, R>;

    fn call(&self, req: R) -> Self::Future {
        ConnectionLimitFuture::Acquiring {
            inner: self.inner.clone(),
            req: Some(req),
            semaphore: PollSemaphore::new(self.semaphore.clone()),
        }
    }
}

#[pin_project(project = ConnectionLimitFutureProj)]
pub enum ConnectionLimitFuture<S, R>
where
    S: Service<R>,
{
    Acquiring {
        inner: Arc<S>,
        req: Option<R>,
        semaphore: PollSemaphore,
    },
    Inner {
        #[pin]
        inner: S::Future,
        permit: Option<OwnedSemaphorePermit>,
    },
}

impl<S, R> Future for ConnectionLimitFuture<S, R>
where
    S: Service<R>,
{
    type Output = ConnectionLimitStream<S::Response>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project() {
                ConnectionLimitFutureProj::Acquiring {
                    inner,
                    req,
                    semaphore,
                } => {
                    let permit = ready!(semaphore.poll_acquire(cx));
                    let new_self = ConnectionLimitFuture::Inner {
                        inner: inner.call(req.take().unwrap()),
                        permit,
                    };
                    self.set(new_self);
                }
                ConnectionLimitFutureProj::Inner { inner, permit } => {
                    return inner.poll(cx).map(|inner| ConnectionLimitStream {
                        inner,
                        permit: permit.take().unwrap(),
                    });
                }
            }
        }
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
