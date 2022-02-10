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
use http::header::HeaderName;
use http::{HeaderValue, Response};
use once_cell::sync::Lazy;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

static TRACE_ID: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("x-b3-traceid"));

/// A layer which adds an `X-B3-TraceId` header to responses.
///
/// It must be installed after trace propagation.
pub struct TraceIdHeaderLayer;

impl<S> Layer<S> for TraceIdHeaderLayer {
    type Service = TraceIdHeaderService<S>;

    fn layer(self, inner: S) -> Self::Service {
        TraceIdHeaderService { inner }
    }
}

pub struct TraceIdHeaderService<S> {
    inner: S,
}

impl<S, R, B> Service<R> for TraceIdHeaderService<S>
where
    S: Service<R, Response = Response<B>>,
{
    type Response = S::Response;

    type Future = TraceIdHeaderFuture<S::Future>;

    fn call(&self, req: R) -> Self::Future {
        TraceIdHeaderFuture {
            inner: self.inner.call(req),
        }
    }
}

#[pin_project]
pub struct TraceIdHeaderFuture<F> {
    #[pin]
    inner: F,
}

impl<F, B> Future for TraceIdHeaderFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut response = ready!(this.inner.poll(cx));
        let context = zipkin::current().expect("zipkin trace not initialized");
        response.headers_mut().insert(
            TRACE_ID.clone(),
            HeaderValue::from_str(&context.trace_id().to_string()).unwrap(),
        );

        Poll::Ready(response)
    }
}
