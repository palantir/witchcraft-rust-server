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
use futures_util::ready;
use http::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue, Request, Response};
use http_body::{Body, SizeHint};
use pin_project::pin_project;
use std::error;
use std::io::Write;
use std::pin::Pin;
use std::task::{Context, Poll};
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
        let encoder = if can_gzip && should_gzip(&response) {
            response.headers_mut().remove(CONTENT_LENGTH);
            response.headers_mut().insert(CONTENT_ENCODING, GZIP);

            Some(GzEncoder::new(
                BytesMut::new().writer(),
                Compression::fast(),
            ))
        } else {
            None
        };

        response.map(|body| GzipBody {
            body,
            encoder,
            done: false,
        })
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

#[pin_project]
pub struct GzipBody<B> {
    #[pin]
    body: B,
    encoder: Option<GzEncoder<Writer<BytesMut>>>,
    done: bool,
}

impl<B> Body for GzipBody<B>
where
    B: Body<Data = Bytes>,
{
    type Data = Bytes;

    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let mut this = self.project();

        if *this.done {
            return Poll::Ready(None);
        }

        let next = ready!(this.body.as_mut().poll_data(cx)).transpose()?;

        let Some(encoder) = this.encoder else {
            return Poll::Ready(next.map(Ok));
        };

        match next {
            Some(next) => {
                encoder.write_all(&next).unwrap();
                if this.body.is_end_stream() {
                    encoder.try_finish().unwrap();
                    *this.done = true;
                } else {
                    encoder.flush().unwrap();
                }
            }
            None => {
                encoder.try_finish().unwrap();
                *this.done = true;
            }
        }

        Poll::Ready(Some(Ok(encoder.get_mut().get_mut().split().freeze())))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        self.project().body.poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.body.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        if self.encoder.is_some() {
            SizeHint::new()
        } else {
            self.body.size_hint()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::test_util::service_fn;
    use flate2::write::GzDecoder;
    use std::io::Write;
    use tokio::task;

    #[tokio::test]
    async fn gzip_large_response() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::new(hyper::Body::from(vec![0; MIN_SIZE as usize + 1]))
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
        while let Some(chunk) = body.data().await {
            decompressor.write_all(&chunk.unwrap()).unwrap();
        }

        assert_eq!(decompressor.finish().unwrap(), [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn respect_missing_accept_encoding() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::new(hyper::Body::from(vec![0; MIN_SIZE as usize + 1]))
        }));

        let response = service.call(Request::builder().body(()).unwrap()).await;

        assert_eq!(response.headers().get(CONTENT_ENCODING), None);

        let mut body = response.into_body();

        let mut buf = vec![];
        while let Some(chunk) = body.data().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        assert_eq!(buf, [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn respect_rejecting_accept_encoding() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::new(hyper::Body::from(vec![0; MIN_SIZE as usize + 1]))
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

        let mut body = response.into_body();

        let mut buf = vec![];
        while let Some(chunk) = body.data().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        assert_eq!(buf, [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn dont_gzip_small_response() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::new(hyper::Body::from(vec![0; 10]))
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

        let mut body = response.into_body();

        let mut buf = vec![];
        while let Some(chunk) = body.data().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        assert_eq!(buf, [0; 10]);
    }

    #[tokio::test]
    async fn preserve_existing_encodings() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::builder()
                .header(CONTENT_ENCODING, "deflate")
                .body(hyper::Body::from(vec![0; MIN_SIZE as usize + 1]))
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

        let mut body = response.into_body();

        let mut buf = vec![];
        while let Some(chunk) = body.data().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        assert_eq!(buf, [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn dont_compress_images() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            Response::builder()
                .header(CONTENT_TYPE, "image/jpeg")
                .body(hyper::Body::from(vec![0; MIN_SIZE as usize + 1]))
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

        let mut body = response.into_body();

        let mut buf = vec![];
        while let Some(chunk) = body.data().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        assert_eq!(buf, [0; MIN_SIZE as usize + 1]);
    }

    #[tokio::test]
    async fn each_chunk_is_decodable() {
        let service = GzipLayer { enabled: true }.layer(service_fn(|_| async {
            let (mut tx, rx) = hyper::Body::channel();
            task::spawn(async move {
                let _ = tx.send_data(Bytes::from("hello")).await;
                let _ = tx.send_data(Bytes::from("world")).await;
            });

            Response::builder().body(rx).unwrap()
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

        let chunk = response.into_body().data().await.unwrap().unwrap();
        let mut decoder = GzDecoder::new(vec![]);
        decoder.write_all(&chunk).unwrap();
        decoder.flush().unwrap();

        assert_eq!(decoder.get_ref(), b"hello");
    }
}
