// Copyright 2021 Palantir Technologies, Inc.
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
use bytes::{BufMut, Bytes, BytesMut};
use conjure_serde::json;
use futures_sink::Sink;
use pin_project::pin_project;
use serde::Serialize;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

#[pin_project]
pub struct JsonAppender<S> {
    #[pin]
    inner: S,
    buf: BytesMut,
}

impl<S> JsonAppender<S> {
    pub fn new(inner: S) -> Self {
        JsonAppender {
            inner,
            buf: BytesMut::new(),
        }
    }
}

impl<T, S> Sink<T> for JsonAppender<S>
where
    T: Serialize,
    S: Sink<Bytes, Error = io::Error>,
{
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, value: T) -> io::Result<()> {
        let this = self.project();
        json::to_writer(this.buf.writer(), &value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        this.buf.put_u8(b'\n');

        this.inner.start_send(this.buf.split().freeze())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().inner.poll_close(cx)
    }
}
