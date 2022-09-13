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
use crate::blocking::body::BodyPart;
use crate::blocking::cancellation::CancellationGuard;
use crate::blocking::pool::ThreadPool;
use crate::blocking::{Cancellation, RequestBody, ResponseWriter};
use crate::body::ClientIo;
use crate::endpoint::{errors, WitchcraftEndpoint};
use crate::health::endpoint_500s::EndpointHealth;
use crate::server::RawBody;
use crate::service::endpoint_metrics::EndpointMetrics;
use crate::service::handler::{BodyWriteAborted, EmptyBody};
use async_trait::async_trait;
use bytes::Bytes;
use conjure_error::Error;
use conjure_http::server::{self, Endpoint, EndpointMetadata, PathSegment, WriteBody};
use futures_channel::{mpsc, oneshot};
use futures_util::Stream;
use http::{Extensions, HeaderMap, Method, Request, Response, StatusCode};
use http_body::combinators::BoxBody;
use http_body::{Body, SizeHint};
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{mem, panic};
use tokio::runtime::Handle;
use witchcraft_log::{info, mdc};
use witchcraft_metrics::MetricRegistry;
use zipkin::TraceContext;

/// A [`WitchcraftEndpoint`] wrapping a Conjure [`Endpoint`].
pub struct ConjureBlockingEndpoint {
    inner: Arc<dyn Endpoint<RequestBody, ResponseWriter> + Sync + Send>,
    thread_pool: Arc<ThreadPool>,
    metrics: EndpointMetrics,
    health: Arc<EndpointHealth>,
}

impl ConjureBlockingEndpoint {
    pub fn new(
        metrics: &MetricRegistry,
        thread_pool: &Arc<ThreadPool>,
        inner: Box<dyn Endpoint<RequestBody, ResponseWriter> + Sync + Send>,
    ) -> Self {
        ConjureBlockingEndpoint {
            metrics: EndpointMetrics::new(metrics, &inner),
            health: Arc::new(EndpointHealth::new()),
            inner: Arc::from(inner),
            thread_pool: thread_pool.clone(),
        }
    }
}

impl EndpointMetadata for ConjureBlockingEndpoint {
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
impl WitchcraftEndpoint for ConjureBlockingEndpoint {
    fn metrics(&self) -> Option<&EndpointMetrics> {
        Some(&self.metrics)
    }

    fn health(&self) -> Option<&Arc<EndpointHealth>> {
        Some(&self.health)
    }

    async fn handle(
        &self,
        mut req: Request<RawBody>,
    ) -> Response<BoxBody<Bytes, BodyWriteAborted>> {
        let (cancellation, guard) = Cancellation::new();
        req.extensions_mut().insert(cancellation);

        let trace_context = zipkin::current();
        let snapshot = mdc::snapshot();
        let (sender, receiver) = oneshot::channel();
        let endpoint = self.inner.clone();
        let handle = Handle::current();

        let blocking = move || {
            let _guard = trace_context.map(zipkin::set_current);
            mdc::set(snapshot);

            let req = req.map(|inner| RequestBody::new(inner, handle.clone()));
            let mut response_extensions = Extensions::new();

            let mut response = match panic::catch_unwind(AssertUnwindSafe(|| {
                endpoint.handle(req, &mut response_extensions)
            })) {
                Ok(Ok(resp)) => resp,
                Ok(Err(e)) => errors::to_response(e, |o| {
                    o.map_or(server::ResponseBody::Empty, server::ResponseBody::Fixed)
                }),
                Err(_) => {
                    let mut response = Response::new(server::ResponseBody::Empty);
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    response
                }
            };

            response.extensions_mut().extend(response_extensions);

            let (parts, body) = response.into_parts();
            let (body, writer) = ResponseBody::new(body, guard, handle);

            let response = Response::from_parts(parts, body.boxed());
            let _ = sender.send(response);

            if let Some(writer) = writer {
                if let Err(e) = writer.write_body() {
                    info!("error writing streaming response body", error: e);
                }
            }
        };

        if self.thread_pool.try_execute(blocking).is_err() {
            let mut response = Response::new(EmptyBody.boxed());
            *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
            return response;
        }

        match receiver.await {
            Ok(response) => response,
            // If we don't get a response, the handler must have panicked. We don't actually care about the payload at
            // this point (it's already been logged), so we just want to propagate a panic with an arbitrary payload to
            // have the same panicking behavior as the async implementation.
            Err(_canceled) => panic::resume_unwind(Box::new("")),
        }
    }
}

struct ResponseBody {
    state: State,
    trailers: Option<HeaderMap>,
    _guard: CancellationGuard,
}

enum State {
    Empty,
    Fixed(Bytes),
    Streaming {
        context_sender: Option<oneshot::Sender<Option<TraceContext>>>,
        receiver: mpsc::Receiver<BodyPart>,
    },
}

impl ResponseBody {
    fn new(
        body: server::ResponseBody<ResponseWriter>,
        guard: CancellationGuard,
        handle: Handle,
    ) -> (Self, Option<StreamingWriter>) {
        let (state, writer) = match body {
            server::ResponseBody::Empty => (State::Empty, None),
            server::ResponseBody::Fixed(bytes) => (State::Fixed(bytes), None),
            server::ResponseBody::Streaming(writer) => {
                let (context_sender, context_receiver) = oneshot::channel();
                let (sender, receiver) = mpsc::channel(1);
                (
                    State::Streaming {
                        context_sender: Some(context_sender),
                        receiver,
                    },
                    Some(StreamingWriter {
                        context_receiver,
                        sender,
                        writer,
                        handle,
                    }),
                )
            }
        };

        (
            ResponseBody {
                state,
                trailers: None,
                _guard: guard,
            },
            writer,
        )
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
                mut context_sender,
                mut receiver,
            } => {
                if let Some(context_sender) = context_sender.take() {
                    let _ = context_sender.send(zipkin::current());
                }

                let poll = loop {
                    match Pin::new(&mut receiver).poll_next(cx) {
                        Poll::Pending => break Poll::Pending,
                        Poll::Ready(Some(BodyPart::Data(bytes))) => {
                            break Poll::Ready(Some(Ok(bytes)));
                        }
                        Poll::Ready(Some(BodyPart::Trailers(trailers))) => {
                            self.trailers = Some(trailers);
                        }
                        Poll::Ready(Some(BodyPart::Done)) => break Poll::Ready(None),
                        Poll::Ready(None) => break Poll::Ready(Some(Err(BodyWriteAborted))),
                    }
                };

                if !matches!(poll, Poll::Ready(None)) {
                    self.state = State::Streaming {
                        context_sender,
                        receiver,
                    };
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

struct StreamingWriter {
    context_receiver: oneshot::Receiver<Option<TraceContext>>,
    sender: mpsc::Sender<BodyPart>,
    writer: Box<dyn WriteBody<ResponseWriter>>,
    handle: Handle,
}

impl StreamingWriter {
    fn write_body(self) -> Result<(), Error> {
        let context = match self.handle.block_on(self.context_receiver) {
            Ok(context) => context,
            Err(e) => return Err(Error::service_safe(e, ClientIo)),
        };
        let _guard = context.map(zipkin::set_current);

        let mut response_writer = ResponseWriter::new(self.sender, self.handle);
        self.writer.write_body(&mut response_writer)?;
        response_writer.finish()?;

        Ok(())
    }
}
