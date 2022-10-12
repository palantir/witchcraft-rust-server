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

use crate::extensions::AuditLogEntry;
use crate::logging::api::AuditLogV3;
use crate::service::{Layer, Service};
use conjure_error::Error;
use futures_sink::Sink;
use futures_util::future::BoxFuture;
use futures_util::SinkExt;
use http::{HeaderMap, Response, StatusCode};
use http_body::{Body, SizeHint};
use pin_project::pin_project;
use std::error;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use witchcraft_log::error;

/// A layer which records audit logs associated with requests.
///
/// If a response contains a [`AuditLogEntry`] extension, it will be sent to the provided logger sink and flushed
/// before the response is returned. If the entry cannot be logged, the response will be converted into a 500. This
/// guarantees that a user will only see the results of a request after its associated audit log has been persisted.
///
/// Since it can change the response it must installed after request logging.
pub struct AuditLogLayer<T> {
    logger: T,
}

impl<T> AuditLogLayer<T> {
    pub fn new(logger: T) -> Self {
        AuditLogLayer { logger }
    }
}

impl<S, T> Layer<S> for AuditLogLayer<T> {
    type Service = AuditLogService<S, T>;

    fn layer(self, inner: S) -> Self::Service {
        AuditLogService {
            logger: Arc::new(Mutex::new(self.logger)),
            inner,
        }
    }
}

pub struct AuditLogService<S, T> {
    logger: Arc<Mutex<T>>,
    inner: S,
}

impl<S, T, R, B> Service<R> for AuditLogService<S, T>
where
    S: Service<R, Response = Response<B>>,
    S::Future: 'static + Send,
    T: Sink<AuditLogV3> + Unpin + 'static + Send,
    T::Error: Into<Box<dyn error::Error + Sync + Send>>,
    B: Send,
{
    type Response = Response<AuditLogResponseBody<B>>;

    type Future = BoxFuture<'static, Self::Response>;

    fn call(&self, req: R) -> Self::Future {
        let inner = self.inner.call(req);

        // Mutex::lock is an async method so we have to box regardless
        Box::pin({
            let logger = self.logger.clone();
            async move {
                let mut response = inner.await;

                if let Some(audit_log_entry) = response.extensions_mut().remove::<AuditLogEntry>() {
                    // NB: SinkExt::send includes a flush call
                    if let Err(e) = logger.lock().await.send(audit_log_entry.0).await {
                        error!(
                            "error persisting audit log entry",
                            error: Error::internal_safe(e)
                        );

                        let mut response = Response::new(AuditLogResponseBody { inner: None });
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

                        return response;
                    }
                }

                response.map(|inner| AuditLogResponseBody { inner: Some(inner) })
            }
        })
    }
}

#[pin_project]
pub struct AuditLogResponseBody<B> {
    #[pin]
    inner: Option<B>,
}

impl<B> Body for AuditLogResponseBody<B>
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

        match this.inner.as_pin_mut() {
            Some(inner) => inner.poll_data(cx),
            None => Poll::Ready(None),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        let this = self.project();

        match this.inner.as_pin_mut() {
            Some(inner) => inner.poll_trailers(cx),
            None => Poll::Ready(Ok(None)),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.inner {
            Some(inner) => inner.size_hint(),
            None => SizeHint::with_exact(0),
        }
    }

    fn is_end_stream(&self) -> bool {
        match &self.inner {
            Some(inner) => inner.is_end_stream(),
            None => true,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::logging::api::{AuditProducer, AuditResult};
    use crate::service::test_util::service_fn;
    use conjure_object::{Utc, Uuid};

    #[allow(clippy::large_enum_variant)]
    #[derive(PartialEq, Debug)]
    enum TestSinkEvent {
        Item(AuditLogV3),
        Flush,
    }

    struct TestSink {
        events: Vec<TestSinkEvent>,
    }

    impl Sink<AuditLogV3> for TestSink {
        type Error = &'static str;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(mut self: Pin<&mut Self>, item: AuditLogV3) -> Result<(), Self::Error> {
            self.events.push(TestSinkEvent::Item(item));
            Ok(())
        }

        fn poll_flush(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            self.events.push(TestSinkEvent::Flush);
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn no_op_with_no_audit_event() {
        let service = AuditLogLayer::new(TestSink { events: vec![] })
            .layer(service_fn(|_| async { Response::new(()) }));

        let response = service.call(()).await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.body().inner.is_some());

        assert_eq!(service.logger.lock().await.events, vec![]);
    }

    #[tokio::test]
    async fn log_audit_event() {
        let log = AuditLogV3::builder()
            .type_("audit.3")
            .deployment("foo")
            .host("bar")
            .product("baz")
            .product_version("1")
            .producer_type(AuditProducer::Server)
            .event_id(Uuid::new_v4())
            .time(Utc::now())
            .name("PUT_FILE")
            .result(AuditResult::Success)
            .build();

        let service = AuditLogLayer::new(TestSink { events: vec![] }).layer(service_fn(|_| {
            let log = log.clone();
            async {
                let mut response = Response::new(());
                response.extensions_mut().insert(AuditLogEntry::v3(log));
                response
            }
        }));

        let response = service.call(()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.body().inner.is_some());

        assert_eq!(
            service.logger.lock().await.events,
            vec![TestSinkEvent::Item(log.clone()), TestSinkEvent::Flush]
        );
    }

    struct FailingSink;

    impl Sink<AuditLogV3> for FailingSink {
        type Error = &'static str;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Err("blammo"))
        }

        fn start_send(self: Pin<&mut Self>, _: AuditLogV3) -> Result<(), Self::Error> {
            Err("blammo")
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Err("blammo"))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn failed_log_returns_500() {
        let service = AuditLogLayer::new(FailingSink).layer(service_fn(|_| async {
            let log = AuditLogV3::builder()
                .type_("audit.3")
                .deployment("foo")
                .host("bar")
                .product("baz")
                .product_version("1")
                .producer_type(AuditProducer::Server)
                .event_id(Uuid::new_v4())
                .time(Utc::now())
                .name("PUT_FILE")
                .result(AuditResult::Success)
                .build();

            let mut response = Response::new(());
            response.extensions_mut().insert(AuditLogEntry::v3(log));
            response
        }));

        let response = service.call(()).await;

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(response.body().inner.is_none());
    }
}
