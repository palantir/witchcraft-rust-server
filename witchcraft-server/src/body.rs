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
use crate::server::RawBody;
use bytes::{Buf, Bytes, BytesMut};
use conjure_error::{Error, ErrorCode, ErrorType};
use conjure_object::Uuid;
use futures_channel::mpsc;
use futures_sink::Sink;
use futures_util::{future, ready, SinkExt, Stream};
use http::HeaderMap;
use http_body::{Body, Frame};
use pin_project::pin_project;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{io, mem};
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite, ReadBuf};

/// A streaming request body.
#[pin_project]
pub struct RequestBody {
    #[pin]
    inner: RawBody,
    cur: Bytes,
    trailers: Option<HeaderMap>,
    #[pin]
    _p: PhantomPinned,
}

impl RequestBody {
    pub(crate) fn new(inner: RawBody) -> Self {
        RequestBody {
            inner,
            cur: Bytes::new(),
            trailers: None,
            _p: PhantomPinned,
        }
    }
    /// Returns the request's trailers, if any are present.
    ///
    /// The body must have been completely read before this is called.
    pub fn trailers(self: Pin<&mut Self>) -> Option<HeaderMap> {
        self.project().trailers.take()
    }

    fn poll_next_raw(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, hyper::Error>>> {
        let mut this = self.project();

        loop {
            let next = ready!(this.inner.as_mut().poll_frame(cx)).transpose()?;

            let Some(next) = next else {
                return Poll::Ready(None);
            };

            let next = match next.into_data() {
                Ok(data) => return Poll::Ready(Some(Ok(data))),
                Err(next) => next,
            };

            if let Ok(trailers) = next.into_trailers() {
                match this.trailers {
                    Some(base) => base.extend(trailers),
                    None => *this.trailers = Some(trailers),
                }
            }
        }
    }
}

impl Stream for RequestBody {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        if this.cur.has_remaining() {
            return Poll::Ready(Some(Ok(mem::take(this.cur))));
        }

        self.poll_next_raw(cx)
            .map_err(|e| Error::service_safe(e, ClientIo))
    }
}

impl AsyncRead for RequestBody {
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

impl AsyncBufRead for RequestBody {
    fn poll_fill_buf(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        while self.cur.is_empty() {
            match ready!(self.as_mut().poll_next_raw(cx))
                .transpose()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            {
                Some(bytes) => *self.as_mut().project().cur = bytes,
                None => break,
            }
        }

        Poll::Ready(Ok(self.project().cur))
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().cur.advance(amt)
    }
}

/// The writer used for streaming response bodies.
#[pin_project]
pub struct ResponseWriter {
    #[pin]
    sender: mpsc::Sender<Frame<Bytes>>,
    buf: BytesMut,
    #[pin]
    _p: PhantomPinned,
}

impl ResponseWriter {
    pub(crate) fn new(sender: mpsc::Sender<Frame<Bytes>>) -> Self {
        ResponseWriter {
            sender,
            buf: BytesMut::new(),
            _p: PhantomPinned,
        }
    }

    /// Like [`Sink::start_send`] except that it sends the response's trailers.
    ///
    /// The body must be fully written before calling this method.
    pub fn start_send_trailers(self: Pin<&mut Self>, trailers: HeaderMap) -> Result<(), Error> {
        self.start_send_inner(Frame::trailers(trailers))
    }

    /// Like [`SinkExt::send`] except that it sends the response's trailers.
    ///
    /// The body must be fully written before calling this method.
    pub async fn send_trailers(mut self: Pin<&mut Self>, trailers: HeaderMap) -> Result<(), Error> {
        future::poll_fn(|cx| self.as_mut().poll_flush_shallow(cx))
            .await
            .map_err(|e| Error::service_safe(e, ClientIo))?;

        self.project()
            .sender
            .send(Frame::trailers(trailers))
            .await
            .map_err(|e| Error::service_safe(e, ClientIo))
    }

    pub(crate) async fn finish(mut self: Pin<&mut Self>) -> Result<(), Error> {
        self.flush().await
    }

    fn start_send_inner(self: Pin<&mut Self>, item: Frame<Bytes>) -> Result<(), Error> {
        let this = self.project();

        assert!(this.buf.is_empty());
        this.sender
            .start_send(item)
            .map_err(|e| Error::service_safe(e, ClientIo))
    }

    fn poll_flush_shallow(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), mpsc::SendError>> {
        let mut this = self.project();

        if this.buf.is_empty() {
            return Poll::Ready(Ok(()));
        }

        ready!(this.sender.as_mut().poll_ready(cx))?;
        this.sender
            .start_send(Frame::data(this.buf.split().freeze()))?;

        Poll::Ready(Ok(()))
    }
}

impl Sink<Bytes> for ResponseWriter {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_flush_shallow(cx))
            .map_err(|e| Error::service_safe(e, ClientIo))?;

        self.project()
            .sender
            .poll_ready(cx)
            .map_err(|e| Error::service_safe(e, ClientIo))
    }

    fn start_send(self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        self.start_send_inner(Frame::data(item))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_flush_shallow(cx))
            .map_err(|e| Error::service_safe(e, ClientIo))?;

        self.project()
            .sender
            .poll_flush(cx)
            .map_err(|e| Error::service_safe(e, ClientIo))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.as_mut().poll_flush_shallow(cx))
            .map_err(|e| Error::service_safe(e, ClientIo))?;

        self.project()
            .sender
            .poll_close(cx)
            .map_err(|e| Error::service_safe(e, ClientIo))
    }
}

impl AsyncWrite for ResponseWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.buf.len() > 4096 {
            ready!(self.as_mut().poll_flush_shallow(cx))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        self.project().buf.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        ready!(self.as_mut().poll_flush_shallow(cx))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.project()
            .sender
            .poll_flush(cx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        ready!(self.as_mut().poll_flush_shallow(cx))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        self.project()
            .sender
            .poll_close(cx)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

pub(crate) struct ClientIo;

impl Serialize for ClientIo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_struct("ClientIo", 0)?.end()
    }
}

impl ErrorType for ClientIo {
    fn code(&self) -> ErrorCode {
        ErrorCode::CustomClient
    }

    fn name(&self) -> &str {
        "Witchcraft:ClientIo"
    }

    fn instance_id(&self) -> Option<Uuid> {
        None
    }

    fn safe_args(&self) -> &'static [&'static str] {
        &[]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conjure_error_from_client_io() {
        Error::service_safe("", ClientIo);
    }
}
