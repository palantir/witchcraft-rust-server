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
use crate::service::{Layer, Service, ServiceBuilder};
use conjure_error::Error;
use futures_util::future::BoxFuture;
use http::{Request, Response};
use http_body::Body;
use hyper::server::conn::{Connection, Http};
use pin_project::pin_project;
use std::convert::Infallible;
use std::error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::server::TlsStream;

pub struct NewConnection<S, L> {
    pub stream: S,
    pub service_builder: ServiceBuilder<L>,
}

pub trait ShutdownService<R> {
    type Response;

    fn call(&self, req: R) -> impl Future<Output = Self::Response> + GracefulShutdown + Send;
}

pub trait GracefulShutdown {
    fn graceful_shutdown(self: Pin<&mut Self>);
}

/// The bridge between the Witchcraft `Service` and Hyper's `Service`.
pub struct HyperService<S> {
    request_service: Arc<S>,
}

impl<S> HyperService<S> {
    pub fn new(request_service: S) -> Self {
        HyperService {
            request_service: Arc::new(request_service),
        }
    }
}

impl<S, R, L, B> ShutdownService<NewConnection<TlsStream<R>, L>> for HyperService<S>
where
    L: Layer<Arc<S>>,
    L::Service: Service<Request<hyper::Body>, Response = Response<B>> + 'static + Sync + Send,
    R: AsyncRead + AsyncWrite + Unpin + 'static + Send,
    B: Body + 'static + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    type Response = Result<(), Error>;

    fn call(
        &self,
        req: NewConnection<TlsStream<R>, L>,
    ) -> impl Future<Output = Self::Response> + GracefulShutdown + Send {
        let mut http = Http::new();

        if req.stream.get_ref().1.alpn_protocol() == Some(b"h2") {
            http.http2_only(true);
        } else {
            http.http1_only(true).http1_half_close(false);
        }

        HyperFuture {
            inner: http.serve_connection(
                req.stream,
                AdaptorService {
                    inner: Arc::new(req.service_builder.service(self.request_service.clone())),
                },
            ),
        }
    }
}

#[pin_project]
pub struct HyperFuture<S, R, B>
where
    S: Service<Request<hyper::Body>, Response = Response<B>> + 'static + Sync + Send,
    B: Body,
{
    #[pin]
    inner: Connection<R, AdaptorService<S>>,
}

impl<S, R, B> Future for HyperFuture<S, R, B>
where
    S: Service<Request<hyper::Body>, Response = Response<B>> + Sync + Send,
    R: AsyncRead + AsyncWrite + Unpin + 'static,
    B: Body + 'static + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    type Output = Result<(), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx).map_err(Error::internal_safe)
    }
}

impl<S, R, B> GracefulShutdown for HyperFuture<S, R, B>
where
    S: Service<Request<hyper::Body>, Response = Response<B>> + Sync + Send,
    R: AsyncRead + AsyncWrite + Unpin + 'static,
    B: Body + 'static + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    fn graceful_shutdown(self: Pin<&mut Self>) {
        self.project().inner.graceful_shutdown()
    }
}

struct AdaptorService<S> {
    inner: Arc<S>,
}

impl<S, R> hyper::service::Service<R> for AdaptorService<S>
where
    S: Service<R> + 'static + Sync + Send,
    R: 'static + Send,
{
    type Response = S::Response;

    type Error = Infallible;

    type Future = BoxFuture<'static, Result<S::Response, Infallible>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: R) -> Self::Future {
        Box::pin({
            let inner = self.inner.clone();
            async move { Ok(inner.call(req).await) }
        })
    }
}
