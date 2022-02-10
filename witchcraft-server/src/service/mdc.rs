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
use http::{HeaderMap, Response};
use http_body::Body;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use witchcraft_log::mdc::{self, Snapshot};

/// A layer which manages the witchcraft-log MDC around the inner service.
pub struct MdcLayer;

impl<S> Layer<S> for MdcLayer {
    type Service = MdcService<S>;

    fn layer(self, inner: S) -> Self::Service {
        MdcService { inner }
    }
}

pub struct MdcService<S> {
    inner: S,
}

impl<S, R, B> Service<R> for MdcService<S>
where
    S: Service<R, Response = Response<B>>,
{
    type Response = Response<MdcBody<B>>;

    type Future = MdcFuture<S::Future>;

    fn call(&self, req: R) -> Self::Future {
        let mut snapshot = Snapshot::new();
        let guard = with(&mut snapshot);
        let inner = self.inner.call(req);
        drop(guard);

        MdcFuture { inner, snapshot }
    }
}

#[pin_project]
pub struct MdcFuture<F> {
    #[pin]
    inner: F,
    snapshot: Snapshot,
}

impl<F, B> Future for MdcFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = Response<MdcBody<B>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = with(this.snapshot);

        this.inner.poll(cx).map(|r| {
            r.map(|inner| MdcBody {
                inner,
                snapshot: mdc::snapshot(),
            })
        })
    }
}

#[pin_project]
pub struct MdcBody<B> {
    #[pin]
    inner: B,
    snapshot: Snapshot,
}

impl<B> Body for MdcBody<B>
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
        let _guard = with(this.snapshot);
        this.inner.poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        let this = self.project();
        let _guard = with(this.snapshot);
        this.inner.poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

fn with(snapshot: &mut Snapshot) -> MdcGuard<'_> {
    mdc::swap(snapshot);
    MdcGuard { snapshot }
}

struct MdcGuard<'a> {
    snapshot: &'a mut Snapshot,
}

impl Drop for MdcGuard<'_> {
    fn drop(&mut self) {
        mdc::swap(self.snapshot);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::test_util::service_fn;
    use bytes::Bytes;

    #[tokio::test]
    async fn basic() {
        struct TestBody(bool);

        impl Body for TestBody {
            type Data = Bytes;

            type Error = ();

            fn poll_data(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
                if !self.0 {
                    self.0 = true;
                    mdc::insert_safe("c", "c");
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    let mut expected = mdc::Map::new();
                    expected.insert("a", "a");
                    expected.insert("b", "b");
                    expected.insert("c", "c");
                    assert_eq!(mdc::snapshot().safe(), &expected);
                    Poll::Ready(Some(Ok(Bytes::from("hi"))))
                }
            }

            fn poll_trailers(
                self: Pin<&mut Self>,
                _: &mut Context<'_>,
            ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
                unimplemented!()
            }
        }
        let service = MdcLayer.layer(service_fn(|()| {
            mdc::insert_safe("a", "a");
            async {
                mdc::insert_safe("b", "b");
                Response::new(TestBody(false))
            }
        }));

        mdc::insert_safe("external", "external");
        let msg = service.call(()).await.data().await.unwrap().unwrap();
        assert_eq!(msg, "hi");

        let mut expected = mdc::Map::new();
        expected.insert("external", "external");
        assert_eq!(mdc::snapshot().safe(), &expected);
    }
}
