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
use conjure_error::Error;
use futures_util::ready;
use http::{Response, StatusCode};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use witchcraft_log::{log, Level};

/// A layer which logs errors attached to responses.
pub struct ErrorLogLayer;

impl<S> Layer<S> for ErrorLogLayer {
    type Service = ErrorLogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ErrorLogService { inner }
    }
}

pub struct ErrorLogService<S> {
    inner: S,
}

impl<S, R, B> Service<R> for ErrorLogService<S>
where
    S: Service<R, Response = Response<B>>,
{
    type Response = S::Response;

    type Future = ErrorLogFuture<S::Future>;

    fn call(&self, req: R) -> Self::Future {
        ErrorLogFuture {
            inner: self.inner.call(req),
        }
    }
}

#[pin_project]
pub struct ErrorLogFuture<F> {
    #[pin]
    inner: F,
}

impl<F, B> Future for ErrorLogFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let response = ready!(self.project().inner.poll(cx));

        if let Some(error) = response.extensions().get::<Error>() {
            let level = match response.status() {
                StatusCode::INTERNAL_SERVER_ERROR => Level::Error,
                _ => Level::Info,
            };

            log!(level, "handler returned non-success", error: error);
        }

        Poll::Ready(response)
    }
}
