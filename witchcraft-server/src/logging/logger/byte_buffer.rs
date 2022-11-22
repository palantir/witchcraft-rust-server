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

use bytes::{Bytes, BytesMut};
use futures_sink::Sink;
use futures_util::ready;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

#[pin_project(project = BufBytesSinkProj)]
pub struct BufBytesSink<S> {
    buf: BytesMut,
    limit: usize,
    pending_write: Option<Bytes>,
    #[pin]
    inner: S,
}

impl<S> BufBytesSink<S> {
    pub fn new(inner: S) -> Self {
        Self::with_capacity(8 * 1024, inner)
    }

    fn with_capacity(limit: usize, inner: S) -> Self {
        BufBytesSink {
            buf: BytesMut::new(),
            limit,
            pending_write: None,
            inner,
        }
    }
}

impl<S> BufBytesSinkProj<'_, S>
where
    S: Sink<Bytes>,
{
    fn poll_write_pending(&mut self, cx: &mut Context) -> Poll<Result<(), S::Error>> {
        let pending_write = match self.pending_write.take() {
            Some(buf) => buf,
            None => return Poll::Ready(Ok(())),
        };

        if pending_write.len() <= *self.limit - self.buf.len() {
            self.buf.extend_from_slice(&pending_write);
            return Poll::Ready(Ok(()));
        }

        let poll = self.poll_write_buf(cx);

        match poll {
            Poll::Ready(Ok(())) => {}
            poll => {
                *self.pending_write = Some(pending_write);
                return poll;
            }
        }

        debug_assert!(self.buf.is_empty());
        if pending_write.len() < *self.limit {
            self.buf.extend_from_slice(&pending_write);
            return Poll::Ready(Ok(()));
        }

        match self.inner.as_mut().poll_ready(cx) {
            Poll::Ready(Ok(())) => {}
            poll => {
                *self.pending_write = Some(pending_write);
                return poll;
            }
        }

        self.inner.as_mut().start_send(pending_write)?;
        Poll::Ready(Ok(()))
    }

    fn poll_write_buf(&mut self, cx: &mut Context) -> Poll<Result<(), S::Error>> {
        if self.buf.is_empty() {
            return Poll::Ready(Ok(()));
        }

        ready!(self.inner.as_mut().poll_ready(cx))?;
        self.inner.as_mut().start_send(self.buf.split().freeze())?;

        Poll::Ready(Ok(()))
    }

    fn poll_flush_shallow(&mut self, cx: &mut Context) -> Poll<Result<(), S::Error>> {
        ready!(self.poll_write_pending(cx))?;
        ready!(self.poll_write_buf(cx))?;

        Poll::Ready(Ok(()))
    }
}

impl<S> Sink<Bytes> for BufBytesSink<S>
where
    S: Sink<Bytes>,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().poll_write_pending(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        let this = self.project();
        debug_assert!(this.pending_write.is_none());
        *this.pending_write = Some(item);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        ready!(this.poll_flush_shallow(cx))?;
        this.inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        ready!(this.poll_flush_shallow(cx))?;
        this.inner.poll_close(cx)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures_util::SinkExt;

    #[tokio::test]
    async fn simple_writes() {
        let mut sink = BufBytesSink::with_capacity(5, Vec::<Bytes>::new());

        sink.feed(Bytes::from(&b"aaaa"[..])).await.unwrap();
        sink.feed(Bytes::from(&b"b"[..])).await.unwrap();
        sink.feed(Bytes::from(&b"ccc"[..])).await.unwrap();
        sink.feed(Bytes::from(&b"d"[..])).await.unwrap();

        assert_eq!(sink.inner, vec![Bytes::from(&b"aaaab"[..])]);

        sink.flush().await.unwrap();

        assert_eq!(
            sink.inner,
            vec![Bytes::from(&b"aaaab"[..]), Bytes::from(&b"cccd"[..])],
        );
    }

    #[tokio::test]
    async fn oversized_writes() {
        let mut sink = BufBytesSink::with_capacity(5, Vec::<Bytes>::new());

        sink.feed(Bytes::from(&b"aaaa"[..])).await.unwrap();
        sink.feed(Bytes::from(&b"bbbbbb"[..])).await.unwrap();
        sink.feed(Bytes::from(&b"c"[..])).await.unwrap();
        sink.flush().await.unwrap();

        assert_eq!(
            sink.inner,
            vec![
                Bytes::from(&b"aaaa"[..]),
                Bytes::from(&b"bbbbbb"[..]),
                Bytes::from(&b"c"[..])
            ],
        );
    }
}
