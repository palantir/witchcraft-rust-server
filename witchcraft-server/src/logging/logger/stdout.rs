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
use crate::logging::logger::byte_buffer::BufBytesSink;
use bytes::Bytes;
use futures_sink::Sink;
use futures_util::ready;
use pin_project::pin_project;
use std::future::Future;
use std::io::{self, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::{self, JoinHandle};

#[pin_project]
pub struct StdoutAppender {
    #[pin]
    inner: BufBytesSink<StdoutSink>,
}

impl StdoutAppender {
    pub fn new() -> Self {
        StdoutAppender {
            inner: BufBytesSink::new(StdoutSink { state: State::Idle }),
        }
    }
}

impl Sink<Bytes> for StdoutAppender {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        self.project().inner.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

enum State {
    Idle,
    Busy(JoinHandle<io::Result<()>>),
}

struct StdoutSink {
    state: State,
}

impl Sink<Bytes> for StdoutSink {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        debug_assert!(matches!(self.state, State::Idle));
        self.state = State::Busy(task::spawn_blocking(move || {
            let mut stdout = io::stdout().lock();
            stdout.write_all(&item)?;
            stdout.flush()
        }));
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut self.state {
            State::Idle => Poll::Ready(Ok(())),
            State::Busy(handle) => {
                let result = ready!(Pin::new(handle).poll(cx))?;
                self.state = State::Idle;
                Poll::Ready(result)
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}
