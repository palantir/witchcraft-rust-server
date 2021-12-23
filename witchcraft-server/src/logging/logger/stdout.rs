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
use bytes::Bytes;
use futures_sink::Sink;
use futures_util::ready;
use std::future::Future;
use std::io::{self, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::{self, JoinHandle};

enum State {
    Idle,
    Busy {
        handle: JoinHandle<io::Result<()>>,
        flushing: bool,
    },
}

pub struct StdoutAppender {
    state: State,
}

impl StdoutAppender {
    pub fn new() -> Self {
        StdoutAppender { state: State::Idle }
    }
}

impl Sink<Bytes> for StdoutAppender {
    type Error = io::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut self.state {
            State::Idle => Poll::Ready(Ok(())),
            State::Busy { handle, .. } => {
                let result = ready!(Pin::new(handle).poll(cx))?;
                self.state = State::Idle;

                Poll::Ready(result)
            }
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        debug_assert!(matches!(self.state, State::Idle));
        self.state = State::Busy {
            handle: task::spawn_blocking(move || io::stdout().write_all(&item)),
            flushing: false,
        };
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            match &mut self.state {
                State::Idle => {
                    self.state = State::Busy {
                        handle: task::spawn_blocking(|| io::stdout().flush()),
                        flushing: true,
                    };
                }
                State::Busy {
                    handle,
                    flushing: false,
                } => {
                    let result = ready!(Pin::new(handle).poll(cx))?;
                    self.state = State::Idle;
                    result?;
                }
                State::Busy {
                    handle,
                    flushing: true,
                } => {
                    let result = ready!(Pin::new(handle).poll(cx))?;
                    self.state = State::Idle;
                    return Poll::Ready(result);
                }
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}
