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
use pin_project::{pin_project, pinned_drop};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use witchcraft_log::info;

/// A layer which logs a message when the inner service's future is dropped before completion.
pub struct CancellationLayer;

impl<S> Layer<S> for CancellationLayer {
    type Service = CancellationService<S>;

    fn layer(self, inner: S) -> Self::Service {
        CancellationService { inner }
    }
}

pub struct CancellationService<S> {
    inner: S,
}

impl<S, R> Service<R> for CancellationService<S>
where
    S: Service<R>,
{
    type Response = S::Response;

    type Future = CancellationFuture<S::Future>;

    fn call(&self, req: R) -> Self::Future {
        CancellationFuture {
            inner: self.inner.call(req),
            complete: false,
        }
    }
}

#[pin_project(PinnedDrop)]
pub struct CancellationFuture<F> {
    #[pin]
    inner: F,
    complete: bool,
}

#[pinned_drop]
impl<F> PinnedDrop for CancellationFuture<F> {
    fn drop(self: Pin<&mut Self>) {
        let this = self.project();
        if !*this.complete {
            info!("request cancelled during processing");
        }
    }
}

impl<F> Future for CancellationFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let r = this.inner.poll(cx);
        if r.is_ready() {
            *this.complete = true;
        }
        r
    }
}
