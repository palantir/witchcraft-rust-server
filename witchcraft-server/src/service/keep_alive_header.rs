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
use http::{HeaderValue, Request, Response, Version};
use once_cell::sync::Lazy;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use witchcraft_server_config::install::InstallConfig;

static KEEP_ALIVE: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("keep-alive"));

/// A layer which adds a `Keep-Alive` header to responses.
pub struct KeepAliveHeaderLayer {
    value: Option<HeaderValue>,
}

impl KeepAliveHeaderLayer {
    pub fn new(config: &InstallConfig) -> Self {
        KeepAliveHeaderLayer {
            value: config
                .server()
                .idle_connection_timeout()
                .map(|d| HeaderValue::try_from(format!("timeout={}", d.as_secs())).unwrap()),
        }
    }
}

impl<S> Layer<S> for KeepAliveHeaderLayer {
    type Service = KeepAliveHeaderService<S>;

    fn layer(self, inner: S) -> Self::Service {
        KeepAliveHeaderService {
            inner,
            value: self.value,
        }
    }
}

pub struct KeepAliveHeaderService<S> {
    inner: S,
    value: Option<HeaderValue>,
}

impl<S, B1, B2> Service<Request<B1>> for KeepAliveHeaderService<S>
where
    S: Service<Request<B1>, Response = Response<B2>>,
{
    type Response = S::Response;

    type Future = KeepAliveHeaderFuture<S::Future>;

    fn call(&self, req: Request<B1>) -> Self::Future {
        // https://datatracker.ietf.org/doc/html/rfc7540#section-8.1.2.2
        let value = match req.version() {
            Version::HTTP_09 | Version::HTTP_10 | Version::HTTP_11 => self.value.clone(),
            _ => None,
        };

        KeepAliveHeaderFuture {
            inner: self.inner.call(req),
            value,
        }
    }
}

#[pin_project]
pub struct KeepAliveHeaderFuture<F> {
    #[pin]
    inner: F,
    value: Option<HeaderValue>,
}

impl<F, B> Future for KeepAliveHeaderFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = Response<B>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut response = ready!(this.inner.poll(cx));
        if let Some(value) = this.value.take() {
            response.headers_mut().insert(KEEP_ALIVE.clone(), value);
        }

        Poll::Ready(response)
    }
}
