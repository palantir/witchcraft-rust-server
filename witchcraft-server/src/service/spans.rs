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
use http::{HeaderMap, Request, Response};
use http_body::Body;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use zipkin::{Detached, OpenSpan, TraceContext};

/// A layer which creates spans covering the request body read, request handling, and response body write.
///
/// It must be installed after trace propagation.
pub struct SpansLayer;

impl<S> Layer<S> for SpansLayer {
    type Service = SpansService<S>;

    fn layer(self, inner: S) -> Self::Service {
        SpansService { inner }
    }
}

pub struct SpansService<S> {
    inner: S,
}

impl<S, B1, B2> Service<Request<B1>> for SpansService<S>
where
    S: Service<Request<SpannedBody<B1>>, Response = Response<B2>>,
{
    type Response = Response<SpannedBody<B2>>;

    type Future = SpansFuture<S::Future>;

    fn call(&self, req: Request<B1>) -> Self::Future {
        let body_context = zipkin::current();
        let req =
            req.map(|inner| SpannedBody::new(inner, "witchcraft: read-request-body", body_context));

        let span = zipkin::next_span().with_name("witchcraft: handle");

        SpansFuture {
            inner: self.inner.call(req),
            body_context,
            span: Some(span.detach()),
        }
    }
}

#[pin_project]
pub struct SpansFuture<F> {
    #[pin]
    inner: F,
    body_context: Option<TraceContext>,
    span: Option<OpenSpan<Detached>>,
}

impl<F, B> Future for SpansFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = Response<SpannedBody<B>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let _guard = zipkin::set_current(this.span.as_ref().unwrap().context());
        let response = ready!(this.inner.poll(cx));
        *this.span = None;

        Poll::Ready(response.map(|inner| {
            SpannedBody::new(inner, "witchcraft: write-response-body", *this.body_context)
        }))
    }
}

#[allow(clippy::large_enum_variant)]
enum LazySpan {
    Pending {
        name: &'static str,
        context: Option<TraceContext>,
    },
    Live(OpenSpan<Detached>),
}

impl LazySpan {
    fn get(&mut self) -> &OpenSpan<Detached> {
        loop {
            match self {
                LazySpan::Pending { name, context } => {
                    let span = match context {
                        Some(context) => zipkin::new_child(*context),
                        None => zipkin::new_trace(),
                    };
                    let span = span.with_name(name).detach();
                    *self = LazySpan::Live(span);
                }
                LazySpan::Live(span) => return span,
            }
        }
    }
}

#[pin_project]
pub struct SpannedBody<B> {
    #[pin]
    inner: B,
    span: Option<LazySpan>,
}

impl<B> SpannedBody<B> {
    fn new(inner: B, name: &'static str, context: Option<TraceContext>) -> Self {
        SpannedBody {
            inner,
            span: Some(LazySpan::Pending { name, context }),
        }
    }
}

impl<B> Body for SpannedBody<B>
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

        let span = match this.span.as_mut() {
            Some(span) => span.get(),
            // Early-exit on polls after error or EOF
            None => return Poll::Ready(None),
        };

        let _guard = zipkin::set_current(span.context());

        let poll = this.inner.poll_data(cx);
        if matches!(poll, Poll::Ready(None | Some(Err(_)))) {
            *this.span = None;
        }

        poll
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        self.project().inner.poll_trailers(cx)
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
    async fn body_spans_are_lazy() {
        test_util::setup_tracer();

        let service = SpansLayer.layer(service_fn(|_| async {
            Response::new(hyper::Body::from("foo"))
        }));

        zipkin::new_trace()
            .detach()
            .bind(async {
                service.call(Request::new(hyper::Body::from("bar"))).await;
            })
            .await;

        let spans = test_util::spans();
        assert_eq!(spans.len(), 2);

        assert_eq!(spans[0].name(), Some("witchcraft: handle"));
        assert_eq!(spans[0].parent_id(), Some(spans[1].id()));
    }

    #[tokio::test]
    async fn all_spans() {
        test_util::setup_tracer();

        let service = SpansLayer.layer(service_fn(
            |req: Request<SpannedBody<hyper::Body>>| async move {
                let mut body = req.into_body();
                while body.data().await.is_some() {}
                Response::new(hyper::Body::from("response"))
            },
        ));

        zipkin::new_trace()
            .detach()
            .bind(async {
                let response = service
                    .call(Request::new(hyper::Body::from("request")))
                    .await;

                let mut body = response.into_body();
                while body.data().await.is_some() {}
            })
            .await;

        let spans = test_util::spans();
        assert_eq!(spans.len(), 4);

        assert_eq!(spans[0].name(), Some("witchcraft: read-request-body"));
        assert_eq!(spans[0].parent_id(), Some(spans[3].id()));

        assert_eq!(spans[1].name(), Some("witchcraft: handle"));
        assert_eq!(spans[1].parent_id(), Some(spans[3].id()));

        assert_eq!(spans[2].name(), Some("witchcraft: write-response-body"));
        assert_eq!(spans[2].parent_id(), Some(spans[3].id()));
    }
}
