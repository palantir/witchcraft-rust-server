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
use crate::body::ClientIo;
use crate::server::RawBody;
use bytes::{Buf, Bytes, BytesMut};
use conjure_error::Error;
use futures_channel::mpsc;
use futures_util::{Future, SinkExt};
use http::HeaderMap;
use http_body::Body;
use std::io::{BufRead, Read, Write};
use std::time::Duration;
use std::{error, io, mem};
use tokio::runtime::Handle;
use tokio::time;

const IO_TIMEOUT: Duration = Duration::from_secs(60);

/// A streaming request body for blocking requests.
pub struct RequestBody {
    inner: RawBody,
    handle: Handle,
    cur: Bytes,
}

impl RequestBody {
    pub(crate) fn new(inner: RawBody, handle: Handle) -> Self {
        RequestBody {
            inner,
            handle,
            cur: Bytes::new(),
        }
    }

    /// Returns the request's trailers, if any are present.
    ///
    /// The body must have been completely read before this is called.
    pub fn trailers(&mut self) -> Result<Option<HeaderMap>, Error> {
        self.handle
            .block_on(async { time::timeout(IO_TIMEOUT, self.inner.trailers()).await })
            .map_err(|e| Error::service_safe(e, ClientIo))?
            .map_err(|e| Error::service_safe(e, ClientIo))
    }

    fn next_raw(&mut self) -> Result<Option<Bytes>, Box<dyn error::Error + Sync + Send>> {
        let next = self
            .handle
            .block_on(async { time::timeout(IO_TIMEOUT, self.inner.data()).await })?
            .transpose()?;

        Ok(next)
    }
}

impl Iterator for RequestBody {
    type Item = Result<Bytes, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.has_remaining() {
            return Some(Ok(mem::take(&mut self.cur)));
        }

        self.next_raw()
            .map_err(|e| Error::service_safe(e, ClientIo))
            .transpose()
    }
}

impl Read for RequestBody {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let in_buf = self.fill_buf()?;
        let len = usize::min(in_buf.len(), buf.len());
        buf[..len].copy_from_slice(&in_buf[..len]);
        self.consume(len);
        Ok(len)
    }
}

impl BufRead for RequestBody {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        while self.cur.is_empty() {
            match self
                .next_raw()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            {
                Some(bytes) => self.cur = bytes,
                None => break,
            }
        }

        Ok(&self.cur)
    }

    fn consume(&mut self, amt: usize) {
        self.cur.advance(amt)
    }
}

pub enum BodyPart {
    Data(Bytes),
    Trailers(HeaderMap),
    Done,
}

/// The writer used for streaming response bodies of blocking endpoints.
pub struct ResponseWriter {
    sender: mpsc::Sender<BodyPart>,
    handle: Handle,
    buf: BytesMut,
}

impl ResponseWriter {
    pub(crate) fn new(sender: mpsc::Sender<BodyPart>, handle: Handle) -> Self {
        Self {
            sender,
            handle,
            buf: BytesMut::new(),
        }
    }

    /// Writes a block of [`Bytes`] to the response body.
    ///
    /// Compared to `ResponseWriter`'s [`Write`] implementation, this method can avoid some copies if the data is
    /// already represented as a [`Bytes`] value.
    pub fn send(&mut self, bytes: Bytes) -> Result<(), Error> {
        self.send_inner(BodyPart::Data(bytes))
    }

    /// Writes the response's trailers.
    ///
    /// The body must be fully written before calling this method.
    pub fn send_trailers(&mut self, trailers: HeaderMap) -> Result<(), Error> {
        self.send_inner(BodyPart::Trailers(trailers))
    }

    fn send_inner(&mut self, part: BodyPart) -> Result<(), Error> {
        self.flush_shallow()
            .map_err(|e| Error::service_safe(e, ClientIo))?;

        Self::with_timeout(&self.handle, self.sender.feed(part))
            .map_err(|e| Error::service_safe(e, ClientIo))?;

        Ok(())
    }

    fn with_timeout<F, R, E>(
        handle: &Handle,
        future: F,
    ) -> Result<R, Box<dyn error::Error + Sync + Send>>
    where
        F: Future<Output = Result<R, E>>,
        E: Into<Box<dyn error::Error + Sync + Send>>,
    {
        handle
            .block_on(async { time::timeout(IO_TIMEOUT, future).await })?
            .map_err(Into::into)
    }

    fn flush_shallow(&mut self) -> Result<(), Box<dyn error::Error + Sync + Send>> {
        if self.buf.is_empty() {
            return Ok(());
        }

        Self::with_timeout(
            &self.handle,
            self.sender.feed(BodyPart::Data(self.buf.split().freeze())),
        )
    }

    pub(crate) fn finish(mut self) -> Result<(), Error> {
        self.flush_shallow()
            .map_err(|e| Error::service_safe(e, ClientIo))?;

        Self::with_timeout(&self.handle, self.sender.send(BodyPart::Done))
            .map_err(|e| Error::service_safe(e, ClientIo))?;

        Ok(())
    }
}

impl Write for ResponseWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.buf.len() > 4096 {
            self.flush_shallow()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        }

        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_shallow()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Self::with_timeout(&self.handle, self.sender.flush())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}
