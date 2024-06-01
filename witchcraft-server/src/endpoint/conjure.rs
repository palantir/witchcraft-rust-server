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
use crate::endpoint::{errors, WitchcraftEndpoint};
use crate::health::endpoint_500s::EndpointHealth;
use crate::server::RawBody;
use crate::service::endpoint_metrics::EndpointMetrics;
use crate::service::handler::BodyWriteAborted;
use crate::{RequestBody, ResponseWriter};
use async_trait::async_trait;
use bytes::Bytes;
use conjure_error::Error;
use conjure_http::server::{
    AsyncEndpoint, AsyncResponseBody, AsyncWriteBody, BoxAsyncEndpoint, EndpointMetadata,
    PathSegment,
};
use futures_channel::mpsc;
use futures_util::future::{BoxFuture, Fuse, FusedFuture};
use futures_util::{FutureExt, Stream};
use http::{Extensions, Method, Request, Response, StatusCode};
use http_body::{Body, Frame, SizeHint};
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use std::future::Future;
use std::mem;
use std::panic::AssertUnwindSafe;
use std::pin::{pin, Pin};
use std::sync::Arc;
use std::task::{Context, Poll};
use sync_wrapper::SyncWrapper;
use witchcraft_log::info;
use witchcraft_metrics::MetricRegistry;

/// A [`WitchcraftEndpoint`] wrapping a Conjure [`AsyncEndpoint`].
pub struct ConjureEndpoint {
    inner: BoxAsyncEndpoint<'static, RequestBody, ResponseWriter>,
    metrics: Option<EndpointMetrics>,
    health: Option<Arc<EndpointHealth>>,
}

impl ConjureEndpoint {
    pub fn new(
        metrics: Option<&MetricRegistry>,
        inner: BoxAsyncEndpoint<'static, RequestBody, ResponseWriter>,
    ) -> Self {
        ConjureEndpoint {
            metrics: metrics.map(|metrics| EndpointMetrics::new(metrics, &inner)),
            health: metrics.map(|_| Arc::new(EndpointHealth::new())),
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

    fn health(&self) -> Option<&Arc<EndpointHealth>> {
        self.health.as_ref()
    }

    async fn handle(&self, req: Request<RawBody>) -> Response<BoxBody<Bytes, BodyWriteAborted>> {
        let req = req.map(RequestBody::new);
        let mut response_extensions = Extensions::new();

        let mut response = match AssertUnwindSafe(self.inner.handle(req, &mut response_extensions))
            .catch_unwind()
            .await
        {
            Ok(Ok(response)) => response.map(ResponseBody::new),
            Ok(Err(error)) => errors::to_response(error, |o| {
                o.map_or(
                    ResponseBody {
                        state: State::Empty,
                    },
                    |b| ResponseBody {
                        state: State::Fixed(Frame::data(b)),
                    },
                )
            }),
            Err(_) => {
                let mut response = Response::new(ResponseBody {
                    state: State::Empty,
                });
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                response
            }
        };
        response.extensions_mut().extend(response_extensions);
        response.map(|b| b.boxed())
    }
}

enum State {
    Empty,
    Fixed(Frame<Bytes>),
    Streaming {
        writer: SyncWrapper<Fuse<BoxFuture<'static, Result<(), Error>>>>,
        receiver: mpsc::Receiver<Frame<Bytes>>,
    },
}

struct ResponseBody {
    state: State,
}

impl ResponseBody {
    fn new(body: AsyncResponseBody<ResponseWriter>) -> Self {
        let state = match body {
            AsyncResponseBody::Empty => State::Empty,
            AsyncResponseBody::Fixed(bytes) => State::Fixed(Frame::data(bytes)),
            AsyncResponseBody::Streaming(writer) => {
                let (sender, receiver) = mpsc::channel(1);
                let writer = async move {
                    let mut body_writer = pin!(ResponseWriter::new(sender));
                    writer.write_body(body_writer.as_mut()).await?;
                    body_writer.finish().await?;
                    Ok(())
                };
                State::Streaming {
                    writer: SyncWrapper::new(writer.boxed().fuse()),
                    receiver,
                }
            }
        };

        ResponseBody { state }
    }
}

impl Body for ResponseBody {
    type Data = Bytes;

    type Error = BodyWriteAborted;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match mem::replace(&mut self.state, State::Empty) {
            State::Empty => Poll::Ready(None),
            State::Fixed(bytes) => Poll::Ready(Some(Ok(bytes))),
            State::Streaming {
                mut writer,
                mut receiver,
            } => {
                if !writer.get_mut().is_terminated() {
                    if let Poll::Ready(Err(error)) = Pin::new(writer.get_mut()).poll(cx) {
                        info!("error writing response body", error: error);
                        return Poll::Ready(Some(Err(BodyWriteAborted)));
                    }
                }

                // NB: it's safe to poll an mpsc::Receiver after termination
                let poll = match Pin::new(&mut receiver).poll_next(cx) {
                    Poll::Ready(Some(frame)) => Poll::Ready(Some(Ok(frame))),
                    Poll::Ready(None) => {
                        if writer.get_mut().is_terminated() {
                            Poll::Ready(None)
                        } else {
                            Poll::Pending
                        }
                    }
                    Poll::Pending => Poll::Pending,
                };

                if !matches!(poll, Poll::Ready(None)) {
                    self.state = State::Streaming { writer, receiver };
                }

                poll
            }
        }
    }

    fn is_end_stream(&self) -> bool {
        matches!(self.state, State::Empty)
    }

    fn size_hint(&self) -> SizeHint {
        match &self.state {
            State::Empty => SizeHint::with_exact(0),
            State::Fixed(bytes) => match bytes.data_ref() {
                Some(data) => SizeHint::with_exact(data.len() as u64),
                None => SizeHint::with_exact(0),
            },
            State::Streaming { .. } => SizeHint::new(),
        }
    }
}
