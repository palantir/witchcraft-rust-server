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
use crate::service::request_id::RequestId;
use crate::service::routing::Route;
use crate::service::{Layer, Service};
use futures_util::ready;
use http::header::USER_AGENT;
use http::{HeaderMap, Request, Response};
use http_body::Body;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use zipkin::{Detached, Kind, OpenSpan};

/// A layer which extracts Zipkin tracing information from a request and creates a top-level span which wraps the inner
/// service.
///
/// It must be installed after routing and request ID generation.
pub struct TracePropagationLayer;

impl<S> Layer<S> for TracePropagationLayer {
    type Service = TracePropagationService<S>;

    fn layer(self, inner: S) -> Self::Service {
        TracePropagationService { inner }
    }
}

pub struct TracePropagationService<S> {
    inner: S,
}

impl<S, B1, B2> Service<Request<B1>> for TracePropagationService<S>
where
    S: Service<Request<B1>, Response = Response<B2>>,
{
    type Response = Response<TracePropagationBody<B2>>;

    async fn call(&self, req: Request<B1>) -> Self::Response {
        let route = req
            .extensions()
            .get::<Route>()
            .expect("Route missing from request extensions");

        let mut span = match http_zipkin::get_trace_context(req.headers()) {
            Some(context) => zipkin::new_child(context),
            None => {
                let flags = http_zipkin::get_sampling_flags(req.headers());
                zipkin::new_trace_from(flags)
            }
        };

        let template = match route {
            Route::Resolved(endpoint) => Some(endpoint.template()),
            _ => None,
        };
        span.name(&format!(
            "witchcraft: {} {}",
            req.method(),
            template.unwrap_or("not_found")
        ));
        span.kind(Kind::Server);
        span.tag("http.method", req.method().as_str());
        span.tag(
            "http.request_id",
            &req.extensions()
                .get::<RequestId>()
                .expect("RequestId missing from request extensions")
                .to_string(),
        );
        if let Some(template) = template {
            span.tag("http.url_details.path", template);
        }
        span.tag("http.url_details.scheme", "https");
        if let Some(user_agent) = req.headers().get(USER_AGENT).and_then(|h| h.to_str().ok()) {
            span.tag("http.useragent", user_agent);
        }
        span.tag("http.version", &format!("{:?}", req.version()));

        TracePropagationFuture {
            inner: self.inner.call(req),
            span: Some(span.detach()),
        }
        .await
    }
}

#[pin_project]
pub struct TracePropagationFuture<F> {
    #[pin]
    inner: F,
    span: Option<OpenSpan<Detached>>,
}

impl<F, B> Future for TracePropagationFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = Response<TracePropagationBody<B>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = zipkin::set_current(this.span.as_ref().unwrap().context());

        let response = ready!(this.inner.poll(cx));

        let mut span = this.span.take().unwrap();
        span.tag("http.status_code", response.status().as_str());

        Poll::Ready(response.map(|inner| TracePropagationBody { inner, span }))
    }
}

#[pin_project]
pub struct TracePropagationBody<B> {
    #[pin]
    inner: B,
    span: OpenSpan<Detached>,
}

impl<B> Body for TracePropagationBody<B>
where
    B: Body,
{
    type Data = B::Data;

    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();
        let _guard = zipkin::set_current(this.span.context());
        this.inner.poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        let this = self.project();
        let _guard = zipkin::set_current(this.span.context());
        this.inner.poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::test_util::{self, service_fn};

    #[tokio::test]
    async fn propagated() {
        test_util::setup_tracer();

        let service = TracePropagationLayer.layer(service_fn(|_| async {
            Response::builder().status(204).body(()).unwrap()
        }));

        service
            .call(
                Request::builder()
                    .method("POST")
                    .header("x-b3-traceid", "0011223344556677")
                    .header("x-b3-spanid", "7766554433221100")
                    .header("x-b3-sampled", "1")
                    .extension(Route::Unresolved)
                    .extension(RequestId::random())
                    .body(())
                    .unwrap(),
            )
            .await;

        let spans = test_util::spans();
        assert_eq!(spans.len(), 1);
        let span = &spans[0];

        assert_eq!(span.trace_id(), "0011223344556677".parse().unwrap());
        assert_eq!(span.parent_id(), Some("7766554433221100".parse().unwrap()));
        assert_eq!(span.name(), Some("witchcraft: post not_found"));
    }
}
