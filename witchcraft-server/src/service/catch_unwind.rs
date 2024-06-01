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
#![allow(clippy::type_complexity)]
use crate::service::handler::BodyWriteAborted;
use crate::service::{Layer, Service};
use futures_util::FutureExt;
use http::{Response, StatusCode};
use http_body::{Body, Frame, SizeHint};
use pin_project::pin_project;
use std::panic::{self, AssertUnwindSafe};
use std::pin::Pin;
use std::task::{Context, Poll};

/// A layer which catches panics in the inner service.
pub struct CatchUnwindLayer;

impl<S> Layer<S> for CatchUnwindLayer {
    type Service = CatchUnwindService<S>;

    fn layer(self, inner: S) -> Self::Service {
        CatchUnwindService { inner }
    }
}

pub struct CatchUnwindService<S> {
    inner: S,
}

impl<S, R, B> Service<R> for CatchUnwindService<S>
where
    S: Service<R, Response = Response<B>> + Sync,
    R: Send,
{
    type Response = Response<CatchUnwindBody<B>>;

    async fn call(&self, req: R) -> Self::Response {
        let r = match panic::catch_unwind(AssertUnwindSafe(|| self.inner.call(req))) {
            Ok(future) => AssertUnwindSafe(future).catch_unwind().await,
            Err(e) => Err(e),
        };

        match r {
            Ok(response) => response.map(|inner| CatchUnwindBody { inner: Some(inner) }),
            Err(_) => panic_response(),
        }
    }
}

fn panic_response<B>() -> Response<CatchUnwindBody<B>> {
    let mut response = Response::new(CatchUnwindBody { inner: None });
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    response
}

#[pin_project]
pub struct CatchUnwindBody<B> {
    #[pin]
    inner: Option<B>,
}

impl<B> Body for CatchUnwindBody<B>
where
    B: Body<Error = BodyWriteAborted>,
{
    type Data = B::Data;

    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let mut this = self.project();

        match this.inner.as_mut().as_pin_mut() {
            Some(inner) => match panic::catch_unwind(AssertUnwindSafe(|| inner.poll_frame(cx))) {
                Ok(poll) => poll,
                Err(_) => {
                    this.inner.set(None);
                    Poll::Ready(Some(Err(BodyWriteAborted)))
                }
            },
            None => Poll::Ready(None),
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.as_ref().map_or(true, Body::is_end_stream)
    }

    fn size_hint(&self) -> SizeHint {
        self.inner
            .as_ref()
            .map_or_else(|| SizeHint::with_exact(0), Body::size_hint)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::test_util::service_fn;
    use bytes::Bytes;
    use futures::future;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn service_panic() {
        fn handle() -> future::Ready<Response<()>> {
            panic!()
        }
        let service = CatchUnwindLayer.layer(service_fn(|_| handle()));

        let response = service.call(()).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn service_async_panic() {
        fn handle() -> Response<()> {
            panic!()
        }
        let service = CatchUnwindLayer.layer(service_fn(|_| async { handle() }));

        let response = service.call(()).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn body_panic() {
        struct TestBody;

        impl Body for TestBody {
            type Data = Bytes;

            type Error = BodyWriteAborted;

            fn poll_frame(
                self: Pin<&mut Self>,
                _: &mut Context<'_>,
            ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
                panic!()
            }
        }

        let service = CatchUnwindLayer.layer(service_fn(|_| async { Response::new(TestBody) }));

        let response = service.call(()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(matches!(
            response.into_body().frame().await,
            Some(Err(BodyWriteAborted))
        ));
    }
}
