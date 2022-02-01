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
use crate::body::ClientIo;
use crate::endpoint::{errors, WitchcraftEndpoint};
use crate::server::RawBody;
use crate::service::endpoint_metrics::EndpointMetrics;
use crate::service::handler::BodyWriteAborted;
use crate::{RequestBody, ResponseWriter};
use async_trait::async_trait;
use bytes::Bytes;
use conjure_error::Error;
use conjure_http::server::{AsyncEndpoint, AsyncResponseBody, EndpointMetadata, PathSegment};
use conjure_http::SafeParams;
use futures_channel::mpsc;
use futures_util::future::{BoxFuture, Fuse, FusedFuture};
use futures_util::{FutureExt, Stream};
use http::{HeaderMap, Method, Request, Response};
use http_body::combinators::BoxBody;
use http_body::{Body, SizeHint};
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};
use sync_wrapper::SyncWrapper;
use tokio::io::AsyncWriteExt;
use tokio::pin;
use witchcraft_log::info;
use witchcraft_metrics::MetricRegistry;

/// A [`WitchcraftEndpoint`] wrapping a Conjure [`AsyncEndpoint`].
pub struct ConjureEndpoint {
    inner: Box<dyn AsyncEndpoint<RequestBody, ResponseWriter> + Sync + Send>,
    metrics: Option<EndpointMetrics>,
}

impl ConjureEndpoint {
    pub fn new(
        metrics: Option<&MetricRegistry>,
        inner: Box<dyn AsyncEndpoint<RequestBody, ResponseWriter> + Sync + Send>,
    ) -> Self {
        ConjureEndpoint {
            metrics: metrics.map(|metrics| EndpointMetrics::new(metrics, &inner)),
            inner,
        }
    }
}

impl EndpointMetadata for ConjureEndpoint {
    fn method(&self) -> Method {
        self.inner.method()
    }

    fn path(&self) -> &[PathSegment] {
        self.inner.path()
    }

    fn template(&self) -> &str {
        self.inner.template()
    }

    fn service_name(&self) -> &str {
        self.inner.service_name()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn deprecated(&self) -> Option<&str> {
        self.inner.deprecated()
    }
}

#[async_trait]
impl WitchcraftEndpoint for ConjureEndpoint {
    fn metrics(&self) -> Option<&EndpointMetrics> {
        self.metrics.as_ref()
    }

    async fn handle(&self, req: Request<RawBody>) -> Response<BoxBody<Bytes, BodyWriteAborted>> {
        let mut safe_params = SafeParams::new();
        let req = req.map(RequestBody::new);
        let mut response = match self.inner.handle(&mut safe_params, req).await {
            Ok(response) => response.map(ResponseBody::new),
            Err(error) => errors::to_response(error, |o| {
                o.map_or(ResponseBody::Empty, ResponseBody::Fixed)
            }),
        };
        response.extensions_mut().insert(safe_params);
        response.map(|b| b.boxed())
    }
}

enum ResponseBody {
    Empty,
    Fixed(Bytes),
    Streaming {
        writer: SyncWrapper<Fuse<BoxFuture<'static, Result<(), Error>>>>,
        receiver: mpsc::Receiver<Bytes>,
    },
}

impl ResponseBody {
    fn new(body: AsyncResponseBody<ResponseWriter>) -> Self {
        match body {
            AsyncResponseBody::Empty => ResponseBody::Empty,
            AsyncResponseBody::Fixed(bytes) => ResponseBody::Fixed(bytes),
            AsyncResponseBody::Streaming(writer) => {
                let (sender, receiver) = mpsc::channel(1);
                let writer = async move {
                    pin! {
                        let body_writer = ResponseWriter::new(sender);
                    }
                    writer.write_body(body_writer.as_mut()).await?;
                    body_writer
                        .flush()
                        .await
                        .map_err(|e| Error::service_safe(e, ClientIo))
                };
                ResponseBody::Streaming {
                    writer: SyncWrapper::new(writer.boxed().fuse()),
                    receiver,
                }
            }
        }
    }
}

impl Body for ResponseBody {
    type Data = Bytes;

    type Error = BodyWriteAborted;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match mem::replace(&mut *self, ResponseBody::Empty) {
            ResponseBody::Empty => Poll::Ready(None),
            ResponseBody::Fixed(bytes) => Poll::Ready(Some(Ok(bytes))),
            ResponseBody::Streaming {
                mut writer,
                mut receiver,
            } => {
                if !writer.get_mut().is_terminated() {
                    if let Poll::Ready(Err(error)) = Pin::new(writer.get_mut()).poll(cx) {
                        info!("error writing response body", error: error);
                        return Poll::Ready(Some(Err(BodyWriteAborted)));
                    }
                }

                let poll = Pin::new(&mut receiver).poll_next(cx).map(|o| o.map(Ok));
                if !matches!(poll, Poll::Ready(None)) {
                    *self = ResponseBody::Streaming { writer, receiver };
                }

                poll
            }
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        matches!(self, &ResponseBody::Empty)
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            ResponseBody::Empty => SizeHint::with_exact(0),
            ResponseBody::Fixed(bytes) => SizeHint::with_exact(bytes.len() as u64),
            ResponseBody::Streaming { .. } => SizeHint::new(),
        }
    }
}
