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
use crate::logging::api::{OrganizationId, RequestLogV2, SessionId, TokenId, TraceId, UserId};
use crate::logging::{self, Appender, Payload};
use crate::service::routing::Route;
use crate::service::{Layer, Service};
use bytes::Buf;
use conjure_http::SafeParams;
use conjure_object::{Any, SafeLong, Utc};
use futures_util::ready;
use http::{HeaderMap, Request, Response};
use http_body::Body;
use pin_project::pin_project;
use serde::Deserialize;
use std::mem;
use std::pin::Pin;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::time::Instant;
use witchcraft_log::mdc;

const USER_AGENT: &str = "User-Agent";
const BROWSER_USER_AGENT: &str = "Browser-User-Agent";
const FETCH_USER_AGENT: &str = "Fetch-User-Agent";

const SAFE_HEADERS: &[&str] = &[
    "Accept",
    "Accept-Encoding",
    "Accept-Language",
    "Accept-Ranges",
    "Cache-Control",
    "Connection",
    "Content-Length",
    "Content-Type",
    "Date",
    "Etag",
    "Expires",
    "If-Modified-Since",
    "If-None-Match",
    "Last-Modified",
    "Pragma",
    "Server",
    "Transfer-Encoding",
    "Vary",
    "X-B3-ParentSpanId",
    "X-B3-Sampled",
    "X-B3-SpanId",
    "X-B3-TraceId",
    "X-Content-Type-Options",
    "X-XSS-Protection",
];

// We only want a few witchcraft-specific MDC entries in request log params
const MDC_KEYS: &[&str] = &[logging::REQUEST_ID_KEY, logging::SAMPLED_KEY];

/// A layer which records request logs.
///
/// It must be installed after routing and logger MDC initialization. It will add the contents of the response's
/// [`SafeParams`] extension as safe parameters.
pub struct RequestLogLayer {
    appender: Arc<Appender<RequestLogV2>>,
}

impl RequestLogLayer {
    pub fn new(appender: Arc<Appender<RequestLogV2>>) -> Self {
        RequestLogLayer { appender }
    }
}

impl<S> Layer<S> for RequestLogLayer {
    type Service = RequestLogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        RequestLogService {
            inner,
            appender: self.appender,
        }
    }
}

pub struct RequestLogService<S> {
    inner: S,
    appender: Arc<Appender<RequestLogV2>>,
}

impl<S, B1, B2> Service<Request<B1>> for RequestLogService<S>
where
    S: Service<Request<RequestLogRequestBody<B1>>, Response = Response<B2>>,
{
    type Response = Response<RequestLogResponseBody<B2>>;

    async fn call(&self, req: Request<B1>) -> Self::Response {
        let protocol = format!("{:?}", req.version());
        let method = req.method().as_str().to_string();
        let path = match req
            .extensions()
            .get::<Route>()
            .expect("Route missing from request extensions")
        {
            Route::Resolved(endpoint) => endpoint.template().to_string(),
            _ => "Unmatched Path".to_string(),
        };

        let mdc = mdc::snapshot();
        let uid = mdc
            .safe()
            .get(logging::mdc::UID_KEY)
            .and_then(|v| UserId::deserialize(v.clone()).ok());
        let sid = mdc
            .safe()
            .get(logging::mdc::SID_KEY)
            .and_then(|v| SessionId::deserialize(v.clone()).ok());
        let token_id = mdc
            .safe()
            .get(logging::mdc::TOKEN_ID_KEY)
            .and_then(|v| TokenId::deserialize(v.clone()).ok());
        let org_id = mdc
            .safe()
            .get(logging::mdc::ORG_ID_KEY)
            .and_then(|v| OrganizationId::deserialize(v.clone()).ok());
        let trace_id = mdc
            .safe()
            .get(logging::mdc::TRACE_ID_KEY)
            .and_then(|v| TraceId::deserialize(v.clone()).ok());

        let mut params = vec![];

        for key in MDC_KEYS {
            if let Some(value) = mdc.safe().get(key) {
                params.push((key.to_string(), value.clone()));
            }
        }

        for header in SAFE_HEADERS {
            if let Some(value) = req.headers().get(*header) {
                params.push((
                    header.to_string(),
                    Any::new(String::from_utf8_lossy(value.as_bytes())).unwrap(),
                ));
            }
        }

        match req.headers().get(FETCH_USER_AGENT) {
            Some(fetch_user_agent) => {
                params.push((
                    USER_AGENT.to_string(),
                    Any::new(String::from_utf8_lossy(fetch_user_agent.as_bytes())).unwrap(),
                ));
                if let Some(user_agent) = req.headers().get(USER_AGENT) {
                    params.push((
                        BROWSER_USER_AGENT.to_string(),
                        Any::new(String::from_utf8_lossy(user_agent.as_bytes())).unwrap(),
                    ));
                }
            }
            None => {
                if let Some(user_agent) = req.headers().get(USER_AGENT) {
                    params.push((
                        USER_AGENT.to_string(),
                        Any::new(String::from_utf8_lossy(user_agent.as_bytes())).unwrap(),
                    ));
                }
            }
        }

        let mut unsafe_params = vec![];
        if let Some(path_and_query) = req.uri().path_and_query() {
            unsafe_params.push((
                "path".to_string(),
                Any::new(path_and_query.as_str()).unwrap(),
            ));
        }

        let mut state = State {
            protocol,
            method,
            path,
            status: 0,
            uid,
            sid,
            token_id,
            org_id,
            trace_id,
            params,
            unsafe_params,
            start_time: Instant::now(),
            request_size: Arc::new(AtomicI64::new(0)),
            response_size: 0,
            appender: self.appender.clone(),
        };

        let response = self
            .inner
            .call(req.map(|inner| RequestLogRequestBody {
                inner,
                request_size: state.request_size.clone(),
            }))
            .await;

        state.status = i32::from(response.status().as_u16());
        if let Some(safe_params) = response.extensions().get::<SafeParams>() {
            state
                .params
                .extend(safe_params.iter().map(|(k, v)| (k.to_string(), v.clone())));
        }

        response.map(|inner| RequestLogResponseBody { inner, state })
    }
}

#[pin_project]
pub struct RequestLogRequestBody<B> {
    #[pin]
    inner: B,
    request_size: Arc<AtomicI64>,
}

impl<B> Body for RequestLogRequestBody<B>
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

        let value = ready!(this.inner.poll_data(cx));
        if let Some(Ok(chunk)) = &value {
            this.request_size
                .fetch_add(chunk.remaining() as i64, Ordering::Relaxed);
        }
        Poll::Ready(value)
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

#[pin_project]
pub struct RequestLogResponseBody<B> {
    #[pin]
    inner: B,
    state: State,
}

impl<B> Body for RequestLogResponseBody<B>
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
        let value = ready!(this.inner.poll_data(cx));
        if let Some(Ok(chunk)) = &value {
            this.state.response_size += chunk.remaining() as i64;
        }
        Poll::Ready(value)
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

struct State {
    protocol: String,
    method: String,
    path: String,
    status: i32,
    uid: Option<UserId>,
    sid: Option<SessionId>,
    token_id: Option<TokenId>,
    org_id: Option<OrganizationId>,
    trace_id: Option<TraceId>,
    params: Vec<(String, Any)>,
    unsafe_params: Vec<(String, Any)>,
    start_time: Instant,
    request_size: Arc<AtomicI64>,
    response_size: i64,
    appender: Arc<Appender<RequestLogV2>>,
}

impl Drop for State {
    fn drop(&mut self) {
        let duration = SafeLong::try_from(self.start_time.elapsed().as_micros())
            .ok()
            .unwrap_or_else(SafeLong::max_value);
        let request_size = SafeLong::try_from(self.request_size.load(Ordering::Relaxed))
            .ok()
            .unwrap_or_else(SafeLong::max_value);
        let response_size = SafeLong::try_from(self.response_size)
            .ok()
            .unwrap_or_else(SafeLong::max_value);

        let request_log = RequestLogV2::builder()
            .type_("request.2")
            .time(Utc::now())
            .protocol(mem::take(&mut self.protocol))
            .path(mem::take(&mut self.path))
            .status(self.status)
            .request_size(request_size)
            .response_size(response_size)
            .duration(duration)
            .method(mem::take(&mut self.method))
            .uid(self.uid.take())
            .sid(self.sid.take())
            .token_id(self.token_id.take())
            .org_id(self.org_id.take())
            .trace_id(self.trace_id.take())
            .extend_params(self.params.drain(..))
            .extend_unsafe_params(self.unsafe_params.drain(..))
            .build();

        let _ = self.appender.try_send(Payload {
            value: request_log,
            cb: None,
        });
    }
}
