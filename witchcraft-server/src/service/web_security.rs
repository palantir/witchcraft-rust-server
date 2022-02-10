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
use http::header::{
    HeaderName, CONTENT_SECURITY_POLICY, REFERRER_POLICY, USER_AGENT, X_CONTENT_TYPE_OPTIONS,
    X_FRAME_OPTIONS, X_XSS_PROTECTION,
};
use http::{HeaderValue, Request, Response};
use once_cell::sync::Lazy;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[allow(clippy::declare_interior_mutable_const)]
const CONTENT_SECURITY_POLICY_VALUE: HeaderValue = HeaderValue::from_static(
    "default-src 'self'; img-src 'self'; style-src 'self' 'unsafe-inline'; frame-ancestors 'self';",
);
#[allow(clippy::declare_interior_mutable_const)]
const REFERRER_POLICY_VALUE: HeaderValue =
    HeaderValue::from_static("strict-origin-when-cross-origin");
#[allow(clippy::declare_interior_mutable_const)]
const X_CONTENT_TYPE_OPTIONS_VALUE: HeaderValue = HeaderValue::from_static("nosniff");
#[allow(clippy::declare_interior_mutable_const)]
const X_FRAME_OPTIONS_VALUE: HeaderValue = HeaderValue::from_static("sameorigin");
#[allow(clippy::declare_interior_mutable_const)]
const X_XSS_PROTECTION_VALUE: HeaderValue = HeaderValue::from_static("1; mode=block");

static X_CONTENT_SECURITY_POLICY: Lazy<HeaderName> =
    Lazy::new(|| HeaderName::from_static("x-content-security-policy"));

const USER_AGENT_IE_10: &str = "MSIE 10";
const USER_AGENT_IE_11: &str = "rv:11.0";

/// A layer which adds security headers to responses.
pub struct WebSecurityLayer;

impl<S> Layer<S> for WebSecurityLayer {
    type Service = WebSecurityService<S>;

    fn layer(self, inner: S) -> Self::Service {
        WebSecurityService { inner }
    }
}

pub struct WebSecurityService<S> {
    inner: S,
}

impl<S, B1, B2> Service<Request<B1>> for WebSecurityService<S>
where
    S: Service<Request<B1>, Response = Response<B2>>,
{
    type Response = S::Response;

    type Future = WebSecurityFuture<S::Future>;

    fn call(&self, req: Request<B1>) -> Self::Future {
        let is_ie = req
            .headers()
            .get(USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .map_or(false, |s| {
                s.contains(USER_AGENT_IE_10) || s.contains(USER_AGENT_IE_11)
            });

        WebSecurityFuture {
            inner: self.inner.call(req),
            is_ie,
        }
    }
}

#[pin_project]
pub struct WebSecurityFuture<F> {
    #[pin]
    inner: F,
    is_ie: bool,
}

impl<F, B> Future for WebSecurityFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut response = ready!(this.inner.poll(cx));
        response
            .headers_mut()
            .insert(CONTENT_SECURITY_POLICY, CONTENT_SECURITY_POLICY_VALUE);
        response
            .headers_mut()
            .insert(REFERRER_POLICY, REFERRER_POLICY_VALUE);
        response
            .headers_mut()
            .insert(X_CONTENT_TYPE_OPTIONS, X_CONTENT_TYPE_OPTIONS_VALUE);
        response
            .headers_mut()
            .insert(X_FRAME_OPTIONS, X_FRAME_OPTIONS_VALUE);
        response
            .headers_mut()
            .insert(X_XSS_PROTECTION, X_XSS_PROTECTION_VALUE);
        if *this.is_ie {
            response.headers_mut().insert(
                X_CONTENT_SECURITY_POLICY.clone(),
                CONTENT_SECURITY_POLICY_VALUE,
            );
        }

        Poll::Ready(response)
    }
}
