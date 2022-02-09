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
use crate::service::Service;
use bytes::Bytes;
use futures_util::future::BoxFuture;
use http::{HeaderMap, Request, Response, StatusCode};
use http_body::combinators::BoxBody;
use http_body::{Body, SizeHint};
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

    fn call(&self, _req: Request<hyper::Body>) -> Self::Future {
        let mut response = Response::new(EmptyBody.boxed());
        *response.status_mut() = StatusCode::NOT_FOUND;
        Box::pin(async move { response })
    }
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
