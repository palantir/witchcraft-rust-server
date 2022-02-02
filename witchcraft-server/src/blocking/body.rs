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
use futures_util::SinkExt;
use http_body::Body;
use std::io::{BufRead, Read, Write};
use std::{io, mem};
use tokio::runtime::Handle;

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
}

impl Iterator for RequestBody {
    type Item = Result<Bytes, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.has_remaining() {
            return Some(Ok(mem::take(&mut self.cur)));
        }

        self.handle
            .block_on(self.inner.data())
            .map(|r| r.map_err(|e| Error::service_safe(e, ClientIo)))
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
                .handle
                .block_on(self.inner.data())
                .transpose()
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

    fn flush_shallow(&mut self) -> io::Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }

        self.handle
            .block_on(self.sender.feed(BodyPart::Data(self.buf.split().freeze())))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }

    pub(crate) fn finish(mut self) -> io::Result<()> {
        self.flush()?;

        self.handle
            .block_on(self.sender.send(BodyPart::Done))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}

impl Write for ResponseWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.buf.len() > 4096 {
            self.flush_shallow()?;
        }

        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_shallow()?;

        self.handle
            .block_on(self.sender.flush())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}
