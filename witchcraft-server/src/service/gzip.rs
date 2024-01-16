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
use bytes::buf::Writer;
use bytes::{BufMut, Bytes, BytesMut};
use flate2::write::GzEncoder;
use flate2::Compression;
use http::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use http::{HeaderValue, Request, Response};
use http_body::{Body, Frame, SizeHint};
use pin_project::pin_project;
use std::io::Write;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{error, mem};
use witchcraft_server_config::install::InstallConfig;

const MIN_SIZE: u64 = 1024 * 1024;

const EXCLUDED_CONTENT_TYPE_PREFIXES: &[&str] = &[
    "video/",
    "audio/",
    "image/",
    "application/bzip2",
    "application/brotli",
    "application/x-rar-compressed",
    "application/gzip",
    "application/compress",
    "application/zip",
    "application/x-xz",
];

#[allow(clippy::declare_interior_mutable_const)]
const GZIP: HeaderValue = HeaderValue::from_static("gzip");

/// A layer which compresses large response bodies.
pub struct GzipLayer {
    enabled: bool,
}

impl GzipLayer {
    pub fn new(config: &InstallConfig) -> Self {
        GzipLayer {
            enabled: config.server().gzip(),
        }
    }
}

impl<S> Layer<S> for GzipLayer {
    type Service = GzipService<S>;

    fn layer(self, inner: S) -> Self::Service {
        GzipService {
            inner,
            enabled: self.enabled,
        }
    }
}

pub struct GzipService<S> {
    inner: S,
    enabled: bool,
}

impl<S, B1, B2> Service<Request<B1>> for GzipService<S>
where
    S: Service<Request<B1>, Response = Response<B2>> + Sync,
    B1: Send,
    B2: Body<Data = Bytes>,
    B2::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    type Response = Response<GzipBody<B2>>;

    async fn call(&self, req: Request<B1>) -> Self::Response {
        let can_gzip = self.enabled && can_gzip(&req);

        let mut response = self.inner.call(req).await;
        let state = if can_gzip && should_gzip(&response) {
            response.headers_mut().remove(CONTENT_LENGTH);
            response.headers_mut().insert(CONTENT_ENCODING, GZIP);

            State::Compressing(GzEncoder::new(
                BytesMut::new().writer(),
                Compression::fast(),
            ))
        } else {
            State::Done
        };

        response.map(|body| GzipBody { body, state })
    }
}

fn can_gzip<B>(request: &Request<B>) -> bool {
    let accept_encoding = match request
        .headers()
        .get(ACCEPT_ENCODING)
        .and_then(|h| h.to_str().ok())
    {
        Some(value) => value,
        None => return false,
    };

    for quality_item in accept_encoding.split(',') {
        let mut it = quality_item.splitn(2, ';');
        let coding = it.next().unwrap().trim();
        if !coding.eq_ignore_ascii_case("gzip") && coding != "*" {
            continue;
        }

        if let Some(weight) = it.next() {
            if matches!(weight.trim(), "q=0" | "q=0." | "q=0.0" | "q=0.00" | "q=000") {
                return false;
            }
        }

        return true;
    }

    false
}

fn should_gzip<B>(response: &Response<B>) -> bool
where
    B: Body,
{
    // We don't compress bodies known to be less than 1MB
    if response
        .body()
        .size_hint()
        .upper()
        .map_or(false, |s| s < MIN_SIZE)
    {
        return false;
    }

    // We don't want to modify already encoded bodies
    if response.headers().contains_key(CONTENT_ENCODING) {
        return false;
    }

    // We don't compress bodies with content types that indicate they're already compressed
    if let Some(content_type) = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
    {
        if EXCLUDED_CONTENT_TYPE_PREFIXES
            .iter()
            .any(|prefix| content_type.starts_with(prefix))
        {
            return false;
        }
    }

    true
}

enum State {
    Compressing(GzEncoder<Writer<BytesMut>>),
    Last(Frame<Bytes>),
    Done,
}

#[pin_project]
pub struct GzipBody<B> {
    #[pin]
    body: B,
    state: State,
}

impl<B> Body for GzipBody<B>
where
    B: Body<Data = Bytes>,
{
    type Data = Bytes;

    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let mut this = self.project();

        match mem::replace(this.state, State::Done) {
            State::Compressing(mut encoder) => match this.body.as_mut().poll_frame(cx) {
                Poll::Ready(Some(Ok(frame))) => match frame.data_ref() {
                    Some(data) => {
                        encoder.write_all(data).unwrap();
                        if this.body.is_end_stream() {
                            encoder.try_finish().unwrap();
                            let buf = encoder.get_mut().get_mut().split().freeze();
                            Poll::Ready(Some(Ok(Frame::data(buf))))
                        } else {
                            encoder.flush().unwrap();
                            let buf = encoder.get_mut().get_mut().split().freeze();
                            *this.state = State::Compressing(encoder);
                            Poll::Ready(Some(Ok(Frame::data(buf))))
                        }
                    }
                    None => {
                        encoder.try_finish().unwrap();
                        if encoder.get_ref().get_ref().is_empty() {
                            Poll::Ready(Some(Ok(frame)))
                        } else {
                            *this.state = State::Last(frame);
                            let buf = encoder.get_mut().get_mut().split().freeze();
                            Poll::Ready(Some(Ok(Frame::data(buf))))
                        }
                    }
                },
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    encoder.try_finish().unwrap();
                    if encoder.get_ref().get_ref().is_empty() {
                        Poll::Ready(None)
                    } else {
                        let buf = encoder.get_mut().get_mut().split().freeze();
                        Poll::Ready(Some(Ok(Frame::data(buf))))
                    }
                }
                Poll::Pending => {
                    *this.state = State::Compressing(encoder);
                    Poll::Pending
                }
            },
            State::Last(frame) => Poll::Ready(Some(Ok(frame))),
            State::Done => this.body.poll_frame(cx),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self.state {
            State::Compressing(_) | State::Done => self.body.is_end_stream(),
            State::Last(_) => false,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.state {
            State::Compressing(_) => SizeHint::new(),
            State::Last(frame) => match frame.data_ref() {
                Some(data) => SizeHint::with_exact(data.len() as u64),
                None => SizeHint::with_exact(0),
            },
            State::Done => self.body.size_hint(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::test_util::service_fn;
    use flate2::write::GzDecoder;
    use futures_channel::mpsc;
    use futures_util::SinkExt;
    use http_body_util::{BodyExt, Full, StreamBody};
    use std::{convert::Infallible, io::Write};
    use tokio::task;

    #[tokio::test]
    async fn gzip_large_response() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::new(Full::new(Bytes::from(vec![0; MIN_SIZE as usize + 1])))
        }));

        let response = service
            .call(
                Request::builder()
                    .header(ACCEPT_ENCODING, "gzip")
                    .body(())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.headers().get(CONTENT_ENCODING).unwrap(), "gzip");

        let mut body = response.into_body();

        let mut decompressor = GzDecoder::new(vec![]);
        while let Some(chunk) = body.frame().await {
            decompressor
                .write_all(chunk.unwrap().data_ref().unwrap())
                .unwrap();
        }

        assert_eq!(decompressor.finish().unwrap(), [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn respect_missing_accept_encoding() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::new(Full::new(Bytes::from(vec![0; MIN_SIZE as usize + 1])))
        }));

        let response = service.call(Request::builder().body(()).unwrap()).await;

        assert_eq!(response.headers().get(CONTENT_ENCODING), None);

        let body = response.into_body();
        let buf = body.collect().await.unwrap().to_bytes();
        assert_eq!(&*buf, [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn respect_rejecting_accept_encoding() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::new(Full::new(Bytes::from(vec![0; MIN_SIZE as usize + 1])))
        }));

        let response = service
            .call(
                Request::builder()
                    .header(ACCEPT_ENCODING, "gzip;q=0")
                    .body(())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.headers().get(CONTENT_ENCODING), None);

        let body = response.into_body();
        let buf = body.collect().await.unwrap().to_bytes();
        assert_eq!(&*buf, [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn dont_gzip_small_response() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::new(Full::new(Bytes::from(vec![0; 10])))
        }));

        let response = service
            .call(
                Request::builder()
                    .header(ACCEPT_ENCODING, "gzip")
                    .body(())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.headers().get(CONTENT_ENCODING), None);

        let body = response.into_body();
        let buf = body.collect().await.unwrap().to_bytes();
        assert_eq!(&*buf, [0; 10]);
    }

    #[tokio::test]
    async fn preserve_existing_encodings() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::builder()
                .header(CONTENT_ENCODING, "deflate")
                .body(Full::new(Bytes::from(vec![0; MIN_SIZE as usize + 1])))
                .unwrap()
        }));

        let response = service
            .call(
                Request::builder()
                    .header(ACCEPT_ENCODING, "gzip")
                    .body(())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.headers().get(CONTENT_ENCODING).unwrap(), "deflate");

        let body = response.into_body();
        let buf = body.collect().await.unwrap().to_bytes();
        assert_eq!(&*buf, [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn dont_compress_images() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::builder()
                .header(CONTENT_TYPE, "image/jpeg")
                .body(Full::new(Bytes::from(vec![0; MIN_SIZE as usize + 1])))
                .unwrap()
        }));

        let response = service
            .call(
                Request::builder()
                    .header(ACCEPT_ENCODING, "gzip")
                    .body(())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.headers().get(CONTENT_ENCODING), None);

        let body = response.into_body();
        let buf = body.collect().await.unwrap().to_bytes();
        assert_eq!(&*buf, [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn each_chunk_is_decodable() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            let (mut tx, rx) = mpsc::channel::<Result<_, Infallible>>(1);
            task::spawn(async move {
                let _ = tx.send(Ok(Frame::data(Bytes::from("hello")))).await;
                let _ = tx.send(Ok(Frame::data(Bytes::from("world")))).await;
            });

            Response::builder().body(StreamBody::new(rx)).unwrap()
        }));

        let response = service
            .call(
                Request::builder()
                    .header(ACCEPT_ENCODING, "gzip")
                    .body(())
                    .unwrap(),
            )
            .await;

        assert_eq!(response.headers().get(CONTENT_ENCODING).unwrap(), "gzip");

        let chunk = response
            .into_body()
            .frame()
            .await
            .unwrap()
            .unwrap()
            .into_data()
            .unwrap();
        let mut decoder = GzDecoder::new(vec![]);
        decoder.write_all(&chunk).unwrap();
        decoder.flush().unwrap();

        assert_eq!(decoder.get_ref(), b"hello");
    }
}
