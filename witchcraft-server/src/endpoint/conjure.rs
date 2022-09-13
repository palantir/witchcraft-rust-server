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
use crate::body::BodyPart;
use crate::endpoint::{errors, WitchcraftEndpoint};
use crate::health::endpoint_500s::EndpointHealth;
use crate::server::RawBody;
use crate::service::endpoint_metrics::EndpointMetrics;
use crate::service::handler::BodyWriteAborted;
use crate::{RequestBody, ResponseWriter};
use async_trait::async_trait;
use bytes::Bytes;
use conjure_error::Error;
use conjure_http::server::{AsyncEndpoint, AsyncResponseBody, EndpointMetadata, PathSegment};
use futures_channel::mpsc;
use futures_util::future::{BoxFuture, Fuse, FusedFuture};
use futures_util::{FutureExt, Stream};
use http::{Extensions, HeaderMap, Method, Request, Response, StatusCode};
use http_body::combinators::BoxBody;
use http_body::{Body, SizeHint};
use std::future::Future;
use std::mem;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use sync_wrapper::SyncWrapper;
use tokio::pin;
use witchcraft_log::info;
use witchcraft_metrics::MetricRegistry;

/// A [`WitchcraftEndpoint`] wrapping a Conjure [`AsyncEndpoint`].
pub struct ConjureEndpoint {
    inner: Box<dyn AsyncEndpoint<RequestBody, ResponseWriter> + Sync + Send>,
    metrics: Option<EndpointMetrics>,
    health: Option<Arc<EndpointHealth>>,
}

impl ConjureEndpoint {
    pub fn new(
        metrics: Option<&MetricRegistry>,
        inner: Box<dyn AsyncEndpoint<RequestBody, ResponseWriter> + Sync + Send>,
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
                        trailers: None,
                    },
                    |b| ResponseBody {
                        state: State::Fixed(b),
                        trailers: None,
                    },
                )
            }),
            Err(_) => {
                let mut response = Response::new(ResponseBody {
                    state: State::Empty,
                    trailers: None,
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
    Fixed(Bytes),
    Streaming {
        writer: SyncWrapper<Fuse<BoxFuture<'static, Result<(), Error>>>>,
        receiver: mpsc::Receiver<BodyPart>,
    },
}

struct ResponseBody {
    state: State,
    trailers: Option<HeaderMap>,
}

impl ResponseBody {
    fn new(body: AsyncResponseBody<ResponseWriter>) -> Self {
        let state = match body {
            AsyncResponseBody::Empty => State::Empty,
            AsyncResponseBody::Fixed(bytes) => State::Fixed(bytes),
            AsyncResponseBody::Streaming(writer) => {
                let (sender, receiver) = mpsc::channel(1);
                let writer = async move {
                    pin! {
                        let body_writer = ResponseWriter::new(sender);
                    }
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

        ResponseBody {
            state,
            trailers: None,
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
                let poll = loop {
                    match Pin::new(&mut receiver).poll_next(cx) {
                        Poll::Ready(Some(BodyPart::Data(data))) => {
                            break Poll::Ready(Some(Ok(data)));
                        }
                        Poll::Ready(Some(BodyPart::Trailers(trailers))) => {
                            self.trailers = Some(trailers);
                        }
                        Poll::Ready(None) => {
                            if writer.get_mut().is_terminated() {
                                break Poll::Ready(None);
                            } else {
                                break Poll::Pending;
                            }
                        }
                        Poll::Pending => break Poll::Pending,
                    }
                };

                if !matches!(poll, Poll::Ready(None)) {
                    self.state = State::Streaming { writer, receiver };
                }

                poll
            }
        }
    }

    fn poll_trailers(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(self.trailers.take()))
    }

    fn is_end_stream(&self) -> bool {
        matches!(self.state, State::Empty) && self.trailers.is_none()
    }

    fn size_hint(&self) -> SizeHint {
        match &self.state {
            State::Empty => SizeHint::with_exact(0),
            State::Fixed(bytes) => SizeHint::with_exact(bytes.len() as u64),
            State::Streaming { .. } => SizeHint::new(),
        }
    }
}
