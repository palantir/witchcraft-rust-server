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
use http::header::{Entry, CACHE_CONTROL};
use http::{HeaderValue, Method, Request, Response};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[allow(clippy::declare_interior_mutable_const)]
const DO_NOT_CACHE: HeaderValue = HeaderValue::from_static("no-cache, no-store, must-revalidate");

/// A layer which disables caching of responses to GET requests that do not already contain a `Cache-Control` header.
pub struct NoCachingLayer;

impl<S> Layer<S> for NoCachingLayer {
    type Service = NoCachingService<S>;

    fn layer(self, inner: S) -> Self::Service {
        NoCachingService { inner }
    }
}

pub struct NoCachingService<S> {
    inner: S,
}

impl<S, B1, B2> Service<Request<B1>> for NoCachingService<S>
where
    S: Service<Request<B1>, Response = Response<B2>> + Sync,
    B1: Send,
{
    type Response = S::Response;

    async fn call(&self, req: Request<B1>) -> Self::Response {
        let is_get = req.method() == Method::GET;

        let mut response = self.inner.call(req).await;
        if is_get {
            if let Entry::Vacant(e) = response.headers_mut().entry(CACHE_CONTROL) {
                e.insert(DO_NOT_CACHE);
            }
        }

        response
    }
}

#[pin_project]
pub struct NoCachingFuture<F> {
    #[pin]
    inner: F,
    is_get: bool,
}

impl<F, B> Future for NoCachingFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut response = ready!(this.inner.poll(cx));
        if *this.is_get {
            if let Entry::Vacant(e) = response.headers_mut().entry(CACHE_CONTROL) {
                e.insert(DO_NOT_CACHE);
            }
        }

        Poll::Ready(response)
    }
}
