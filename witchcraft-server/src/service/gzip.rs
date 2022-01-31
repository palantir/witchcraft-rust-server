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
use async_compression::tokio::bufread::GzipEncoder;
use bytes::{Buf, Bytes, BytesMut};
use futures_util::{ready, Stream};
use http::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue, Request, Response};
use http_body::{Body, SizeHint};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{error, io};
use tokio::io::{AsyncBufRead, AsyncRead, ReadBuf};
use tokio_util::codec::{BytesCodec, FramedRead};
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
    S: Service<Request<B1>, Response = Response<B2>>,
    B2: Body<Data = Bytes>,
    B2::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    type Response = Response<GzipBody<B2>>;

    type Future = GzipFuture<S::Future>;

    fn call(&self, req: Request<B1>) -> Self::Future {
        GzipFuture {
            can_gzip: self.enabled && can_gzip(&req),
            inner: self.inner.call(req),
        }
    }
}

#[pin_project]
pub struct GzipFuture<F> {
    #[pin]
    inner: F,
    can_gzip: bool,
}

impl<F, B> Future for GzipFuture<F>
where
    F: Future<Output = Response<B>>,
    B: Body<Data = Bytes>,
    B::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    type Output = Response<GzipBody<B>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let mut response = ready!(this.inner.poll(cx));
        let response = if *this.can_gzip && should_gzip(&response) {
            response.headers_mut().remove(CONTENT_LENGTH);
            response.headers_mut().insert(CONTENT_ENCODING, GZIP);

            response.map(|body| GzipBody::Gzip {
                body: FramedRead::new(
                    GzipEncoder::new(ShimReader {
                        body,
                        buf: Bytes::new(),
                    }),
                    BytesCodec::new(),
                ),
            })
        } else {
            response.map(|body| GzipBody::Identity { body })
        };

        Poll::Ready(response)
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

#[pin_project(project = GzipBodyProj)]
pub enum GzipBody<B> {
    Identity {
        #[pin]
        body: B,
    },
    Gzip {
        #[pin]
        body: FramedRead<GzipEncoder<ShimReader<B>>, BytesCodec>,
    },
}

impl<B> Body for GzipBody<B>
where
    B: Body<Data = Bytes>,
    B::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    type Data = Bytes;

    type Error = io::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        match self.project() {
            GzipBodyProj::Identity { body } => body
                .poll_data(cx)
                .map(|o| o.map(|r| r.map_err(|e| io::Error::new(io::ErrorKind::Other, e)))),
            GzipBodyProj::Gzip { body } => body
                .poll_next(cx)
                .map(|o| o.map(|r| r.map(BytesMut::freeze))),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        match self.project() {
            GzipBodyProj::Identity { body } => body
                .poll_trailers(cx)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
            GzipBodyProj::Gzip { body } => body
                .get_pin_mut()
                .get_pin_mut()
                .project()
                .body
                .poll_trailers(cx)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            GzipBody::Identity { body } => body.is_end_stream(),
            // we can't check the inner body since we may get an error out of the encoder at eof
            GzipBody::Gzip { .. } => false,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            GzipBody::Identity { body } => body.size_hint(),
            GzipBody::Gzip { .. } => SizeHint::new(),
        }
    }
}

#[pin_project]
pub struct ShimReader<B> {
    #[pin]
    body: B,
    buf: Bytes,
}

impl<B> AsyncRead for ShimReader<B>
where
    B: Body<Data = Bytes>,
    B::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let in_buf = ready!(self.as_mut().poll_fill_buf(cx))?;
        let len = usize::min(in_buf.len(), buf.remaining());
        buf.put_slice(&in_buf[..len]);
        self.consume(len);

        Poll::Ready(Ok(()))
    }
}

impl<B> AsyncBufRead for ShimReader<B>
where
    B: Body<Data = Bytes>,
    B::Error: Into<Box<dyn error::Error + Sync + Send>>,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        let mut this = self.project();

        while !this.buf.has_remaining() {
            *this.buf = match ready!(this.body.as_mut().poll_data(cx)) {
                Some(Ok(buf)) => buf,
                Some(Err(e)) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
                None => break,
            }
        }

        Poll::Ready(Ok(&**this.buf))
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().buf.advance(amt)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::test_util::service_fn;
    use flate2::write::GzDecoder;
    use std::io::Write;

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
}
