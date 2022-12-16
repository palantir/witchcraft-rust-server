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

use crate::logging::logger::Payload;
use bytes::{Bytes, BytesMut};
use futures_channel::oneshot;
use futures_sink::Sink;
use futures_util::ready;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

#[pin_project(project = BufBytesSinkProj)]
pub struct BufBytesSink<S> {
    buf: BytesMut,
    pending_cbs: Vec<oneshot::Sender<bool>>,
    pending_write: Option<Payload<Bytes>>,
    needs_flush: bool,
    limit: usize,
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
            pending_cbs: vec![],
            pending_write: None,
            needs_flush: false,
            limit,
            inner,
        }
    }
}

impl<S> BufBytesSinkProj<'_, S>
where
    S: Sink<Bytes>,
{
    fn inner_poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        let poll = self.inner.as_mut().poll_ready(cx);
        if let Poll::Ready(Err(_)) = poll {
            self.drain_cbs(false);
        }

        poll
    }

    fn inner_poll_flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        let result = ready!(self.inner.as_mut().poll_flush(cx));
        *self.needs_flush = false;
        self.drain_cbs(result.is_ok());

        Poll::Ready(result)
    }

    fn inner_poll_close(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        let result = ready!(self.inner.as_mut().poll_close(cx));
        self.drain_cbs(result.is_ok());

        Poll::Ready(result)
    }

    fn inner_start_send(&mut self, item: Bytes) -> Result<(), S::Error> {
        let result = self.inner.as_mut().start_send(item);
        match result {
            Ok(_) => *self.needs_flush = true,
            Err(_) => self.drain_cbs(false),
        }

        result
    }

    fn drain_cbs(&mut self, ok: bool) {
        for cb in self.pending_cbs.drain(..) {
            let _ = cb.send(ok);
        }
    }

    fn poll_write_pending(&mut self, cx: &mut Context) -> Poll<Result<(), S::Error>> {
        let pending_write = match self.pending_write.take() {
            Some(buf) => buf,
            None => return Poll::Ready(Ok(())),
        };

        if pending_write.value.len() <= *self.limit - self.buf.len() {
            self.buf.extend_from_slice(&pending_write.value);
            if let Some(cb) = pending_write.cb {
                self.pending_cbs.push(cb);
            }
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
        if pending_write.value.len() < *self.limit {
            self.buf.extend_from_slice(&pending_write.value);
            if let Some(cb) = pending_write.cb {
                self.pending_cbs.push(cb);
            }
            return Poll::Ready(Ok(()));
        }

        match self.inner.as_mut().poll_ready(cx) {
            Poll::Ready(Ok(())) => {}
            poll => {
                *self.pending_write = Some(pending_write);
                return poll;
            }
        }

        if let Some(cb) = pending_write.cb {
            self.pending_cbs.push(cb);
        }
        self.inner_start_send(pending_write.value)?;

        Poll::Ready(Ok(()))
    }

    fn poll_write_buf(&mut self, cx: &mut Context) -> Poll<Result<(), S::Error>> {
        if self.buf.is_empty() {
            return Poll::Ready(Ok(()));
        }

        ready!(self.inner_poll_ready(cx))?;
        let buf = self.buf.split().freeze();
        self.inner_start_send(buf)?;

        Poll::Ready(Ok(()))
    }

    fn poll_flush_shallow(&mut self, cx: &mut Context) -> Poll<Result<(), S::Error>> {
        ready!(self.poll_write_pending(cx))?;
        ready!(self.poll_write_buf(cx))?;

        Poll::Ready(Ok(()))
    }
}

impl<S> Sink<Payload<Bytes>> for BufBytesSink<S>
where
    S: Sink<Bytes>,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        ready!(this.poll_write_pending(cx))?;

        if *this.needs_flush {
            ready!(this.inner_poll_flush(cx))?;
        }

        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Payload<Bytes>) -> Result<(), Self::Error> {
        let this = self.project();
        debug_assert!(this.pending_write.is_none());
        debug_assert!(!*this.needs_flush);
        *this.pending_write = Some(item);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        ready!(this.poll_flush_shallow(cx))?;
        this.inner_poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        ready!(this.poll_flush_shallow(cx))?;
        this.inner_poll_close(cx)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures_util::{FutureExt, SinkExt};

    #[tokio::test]
    async fn simple_writes() {
        let mut sink = BufBytesSink::with_capacity(5, Vec::<Bytes>::new());

        let (tx1, mut rx1) = oneshot::channel();
        sink.feed(Payload {
            value: Bytes::from(&b"aaaa"[..]),
            cb: Some(tx1),
        })
        .await
        .unwrap();

        sink.feed(Payload {
            value: Bytes::from(&b"b"[..]),
            cb: None,
        })
        .await
        .unwrap();

        sink.feed(Payload {
            value: Bytes::from(&b"ccc"[..]),
            cb: None,
        })
        .await
        .unwrap();
        assert_eq!((&mut rx1).now_or_never(), None);

        let (tx2, mut rx2) = oneshot::channel();
        sink.feed(Payload {
            value: Bytes::from(&b"d"[..]),
            cb: Some(tx2),
        })
        .await
        .unwrap();

        assert_eq!(sink.inner, vec![Bytes::from(&b"aaaab"[..])]);
        assert_eq!(rx1.now_or_never(), Some(Ok(true)));
        assert_eq!((&mut rx2).now_or_never(), None);

        sink.flush().await.unwrap();

        assert_eq!(
            sink.inner,
            vec![Bytes::from(&b"aaaab"[..]), Bytes::from(&b"cccd"[..])],
        );
        assert_eq!(rx2.now_or_never(), Some(Ok(true)));
    }

    #[tokio::test]
    async fn oversized_writes() {
        let mut sink = BufBytesSink::with_capacity(5, Vec::<Bytes>::new());

        let (tx1, rx1) = oneshot::channel();
        sink.feed(Payload {
            value: Bytes::from(&b"aaaa"[..]),
            cb: Some(tx1),
        })
        .await
        .unwrap();
        let (tx2, rx2) = oneshot::channel();
        sink.feed(Payload {
            value: Bytes::from(&b"bbbbbb"[..]),
            cb: Some(tx2),
        })
        .await
        .unwrap();
        let (tx3, rx3) = oneshot::channel();
        sink.feed(Payload {
            value: Bytes::from(&b"c"[..]),
            cb: Some(tx3),
        })
        .await
        .unwrap();
        sink.flush().await.unwrap();

        assert_eq!(
            sink.inner,
            vec![
                Bytes::from(&b"aaaa"[..]),
                Bytes::from(&b"bbbbbb"[..]),
                Bytes::from(&b"c"[..])
            ],
        );
        assert_eq!(rx1.await, Ok(true));
        assert_eq!(rx2.await, Ok(true));
        assert_eq!(rx3.await, Ok(true));
    }

    struct FailingSink;

    impl Sink<Bytes> for FailingSink {
        type Error = &'static str;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, _: Bytes) -> Result<(), Self::Error> {
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Err("blammo"))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn failed_flushes() {
        let mut sink = BufBytesSink::with_capacity(5, FailingSink);

        let (tx1, rx1) = oneshot::channel();
        sink.feed(Payload {
            value: Bytes::new(),
            cb: Some(tx1),
        })
        .await
        .unwrap();

        let (tx2, rx2) = oneshot::channel();
        sink.feed(Payload {
            value: Bytes::new(),
            cb: Some(tx2),
        })
        .await
        .unwrap();

        sink.flush().await.unwrap_err();

        assert_eq!(rx1.await, Ok(false));
        assert_eq!(rx2.await, Ok(false));
    }
}
