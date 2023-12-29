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
use crate::service::routing::Route;
use crate::service::{Layer, Service};
use futures_util::ready;
use http::header::HeaderName;
use http::{HeaderValue, Request, Response};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

const DEPRECATION: HeaderName = HeaderName::from_static("deprecation");
#[allow(clippy::declare_interior_mutable_const)]
const IS_DEPRECATED: HeaderValue = HeaderValue::from_static("true");

/// A layer which adds a deprecation header to deprecated endpoints.
///
/// It must be installed after routing.
pub struct DeprecationHeaderLayer;

impl<S> Layer<S> for DeprecationHeaderLayer {
    type Service = DeprecationHeaderService<S>;

    fn layer(self, inner: S) -> Self::Service {
        DeprecationHeaderService { inner }
    }
}

pub struct DeprecationHeaderService<S> {
    inner: S,
}

impl<S, B1, B2> Service<Request<B1>> for DeprecationHeaderService<S>
where
    S: Service<Request<B1>, Response = Response<B2>>,
{
    type Response = S::Response;

    async fn call(&self, req: Request<B1>) -> Self::Response {
        let route = req
            .extensions()
            .get::<Route>()
            .expect("Route missing from request extensions");

        let deprecated = match route {
            Route::Resolved(endpoint) => endpoint.deprecated().is_some(),
            _ => false,
        };

        let mut response = self.inner.call(req).await;
        if deprecated {
            response.headers_mut().insert(DEPRECATION, IS_DEPRECATED);
        }

        response
    }
}

#[pin_project]
pub struct DeprecationHeaderFuture<F> {
    #[pin]
    inner: F,
    deprecated: bool,
}

impl<F, B> Future for DeprecationHeaderFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = Response<B>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut response = ready!(this.inner.poll(cx));
        if *this.deprecated {
            response
                .headers_mut()
                .insert(DEPRECATION.clone(), IS_DEPRECATED);
        }
        Poll::Ready(response)
    }
}
