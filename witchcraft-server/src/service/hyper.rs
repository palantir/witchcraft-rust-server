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
use http::{Request, Response};
use http_body::Body;
use hyper::server::conn::{Connection, Http};
use pin_project::pin_project;
use std::error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_openssl::SslStream;

pub struct NewConnection<S, L> {
    pub stream: S,
    pub service_builder: ServiceBuilder<L>,
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

impl<S, R, L, B> Service<NewConnection<SslStream<R>, L>> for HyperService<S>
where
    L: Layer<Arc<S>>,
    L::Service: Service<Request<hyper::Body>, Response = Response<B>>,
    <L::Service as Service<Request<hyper::Body>>>::Future: 'static + Send,
    R: AsyncRead + AsyncWrite + Unpin + 'static,
    B: Body + 'static + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    type Response = Result<(), Error>;

    type Future = HyperFuture<L::Service, SslStream<R>, B>;

    fn call(&self, req: NewConnection<SslStream<R>, L>) -> Self::Future {
        let mut http = Http::new();

        if req.stream.ssl().selected_alpn_protocol() == Some(b"h2") {
            http.http2_only(true);
        } else {
            http.http1_only(true).http1_half_close(false);
        }

        HyperFuture {
            inner: http.serve_connection(
                req.stream,
                AdaptorService {
                    inner: req.service_builder.service(self.request_service.clone()),
                },
            ),
        }
    }
}

#[pin_project]
pub struct HyperFuture<S, R, B>
where
    S: Service<Request<hyper::Body>, Response = Response<B>>,
    B: Body,
{
    #[pin]
    inner: Connection<R, AdaptorService<S>>,
}

impl<S, R, B> Future for HyperFuture<S, R, B>
where
    S: Service<Request<hyper::Body>, Response = Response<B>>,
    S::Future: 'static + Send,
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
    S: Service<Request<hyper::Body>, Response = Response<B>>,
    S::Future: 'static + Send,
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
    inner: S,
}

impl<S, R> hyper::service::Service<R> for AdaptorService<S>
where
    S: Service<R>,
{
    type Response = S::Response;

    type Error = Void;

    type Future = AdaptorFuture<S::Future>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: R) -> Self::Future {
        AdaptorFuture {
            inner: self.inner.call(req),
        }
    }
}

pub enum Void {}

impl From<Void> for Box<dyn error::Error + Sync + Send> {
    fn from(void: Void) -> Self {
        match void {}
    }
}

#[pin_project]
pub struct AdaptorFuture<F> {
    #[pin]
    inner: F,
}

impl<F, T> Future for AdaptorFuture<F>
where
    F: Future<Output = T>,
{
    type Output = Result<T, Void>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().inner.poll(cx).map(Ok)
    }
}
