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
use crate::service::Service;
use bytes::Bytes;
use futures_util::future::BoxFuture;
use http::header::ALLOW;
use http::{HeaderMap, HeaderValue, Method, Request, Response, StatusCode};
use http_body::combinators::BoxBody;
use http_body::{Body, SizeHint};
use itertools::Itertools;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{error, fmt};

/// The terminal service in the handler stack which turns [`Request`]s into [`Response`]s.
///
/// It must be installed after routing.
pub struct HandlerService;

impl Service<Request<hyper::Body>> for HandlerService {
    type Response = Response<BoxBody<Bytes, BodyWriteAborted>>;

    type Future = BoxFuture<'static, Self::Response>;

    fn call(&self, mut req: Request<hyper::Body>) -> Self::Future {
        let route = req
            .extensions_mut()
            .remove::<Route>()
            .expect("Route missing from request extensions");

        match route {
            Route::Resolved(endpoint) => Box::pin(async move { endpoint.handle(req).await }),
            Route::MethodNotAllowed(methods) => {
                let mut response = Response::new(EmptyBody.boxed());
                *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                response.headers_mut().insert(ALLOW, allow_header(&methods));
                Box::pin(async { response })
            }
            Route::StarOptions => {
                let mut response = Response::new(EmptyBody.boxed());
                *response.status_mut() = StatusCode::NO_CONTENT;
                Box::pin(async { response })
            }
            Route::Options(methods) => {
                let mut response = Response::new(EmptyBody.boxed());
                *response.status_mut() = StatusCode::NO_CONTENT;
                response.headers_mut().insert(ALLOW, allow_header(&methods));
                Box::pin(async { response })
            }
            Route::Unresolved => {
                let mut response = Response::new(EmptyBody.boxed());
                *response.status_mut() = StatusCode::NOT_FOUND;
                Box::pin(async { response })
            }
        }
    }
}

fn allow_header(methods: &[Method]) -> HeaderValue {
    let header = methods.iter().map(|m| m.to_string()).join(", ");
    HeaderValue::try_from(header).unwrap()
}

#[derive(Debug)]
pub struct BodyWriteAborted;

impl fmt::Display for BodyWriteAborted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("body write aborted")
    }
}

impl error::Error for BodyWriteAborted {}

pub struct EmptyBody;

impl Body for EmptyBody {
    type Data = Bytes;

    type Error = BodyWriteAborted;

    fn poll_data(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Poll::Ready(None)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        true
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(0)
    }
}
