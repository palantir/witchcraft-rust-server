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
use crate::logging::format::LogFormat;
use crate::shutdown::ShutdownHooks;
use futures_sink::Sink;
use futures_util::{ready, SinkExt};
use parking_lot::Mutex;
use pin_project::pin_project;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use tokio::task;
use witchcraft_metrics::{MetricId, MetricRegistry};

pub struct Closed;

const QUEUE_LIMIT: usize = 10_000;

struct State<T> {
    queue: VecDeque<T>,
    write_waker: Option<Waker>,
    read_waker: Option<Waker>,
    flushed: bool,
    closed: bool,
}

impl<T> State<T> {
    fn ready(&self) -> bool {
        self.queue.len() < QUEUE_LIMIT
    }

    fn start_send(&mut self, item: T) {
        debug_assert!(self.queue.len() < QUEUE_LIMIT);

        self.queue.push_back(item);
        self.flushed = false;
        if let Some(waker) = self.read_waker.take() {
            waker.wake();
        }
    }
}

pub struct AsyncAppender<T> {
    state: Arc<Mutex<State<T>>>,
}

impl<T> Drop for AsyncAppender<T> {
    fn drop(&mut self) {
        self.state.lock().closed = true;
    }
}

impl<T> AsyncAppender<T> {
    pub fn new<S>(inner: S, metrics: &MetricRegistry, hooks: &mut ShutdownHooks) -> Self
    where
        S: Sink<T> + 'static + Send,
        T: LogFormat + 'static + Send,
    {
        let state = Arc::new(Mutex::new(State {
            queue: VecDeque::new(),
            write_waker: None,
            read_waker: None,
            flushed: true,
            closed: false,
        }));

        metrics.gauge(MetricId::new("logging.queue").with_tag("type", T::TYPE), {
            let state = state.clone();
            move || state.lock().queue.len()
        });

        task::spawn({
            let state = state.clone();
            WorkerFuture { state, inner }
        });

        hooks.push({
            let mut appender = AsyncAppender {
                state: state.clone(),
            };
            async move {
                let _ = Pin::new(&mut appender).close().await;
            }
        });

        AsyncAppender { state }
    }

    pub fn try_send(&self, item: T) -> Result<(), T> {
        let mut state = self.state.lock();

        if state.closed || !state.ready() {
            return Err(item);
        }
        state.start_send(item);
        Ok(())
    }

    pub fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Closed>> {
        let mut state = self.state.lock();

        if state.closed {
            return Poll::Ready(Err(Closed));
        }

        if state.ready() {
            return Poll::Ready(Ok(()));
        }

        if state
            .write_waker
            .as_ref()
            .map_or(true, |w| !w.will_wake(cx.waker()))
        {
            state.write_waker = Some(cx.waker().clone());
        }

        Poll::Pending
    }

    pub fn start_send(&self, item: T) -> Result<(), Closed> {
        let mut state = self.state.lock();

        if state.closed {
            return Err(Closed);
        }

        state.start_send(item);
        Ok(())
    }

    pub fn poll_flush(&self, cx: &mut Context<'_>) -> Poll<()> {
        let mut state = self.state.lock();

        if state.flushed {
            return Poll::Ready(());
        }

        if state
            .write_waker
            .as_ref()
            .map_or(true, |w| !w.will_wake(cx.waker()))
        {
            state.write_waker = Some(cx.waker().clone());
        }

        Poll::Pending
    }

    pub fn poll_close(&self, cx: &mut Context<'_>) -> Poll<()> {
        let mut state = self.state.lock();

        if !state.closed {
            state.closed = true;
            if let Some(waker) = state.read_waker.take() {
                waker.wake();
            }

            if let Some(waker) = state.write_waker.take() {
                waker.wake();
            }
        }

        drop(state);
        ready!(self.poll_flush(cx));
        Poll::Ready(())
    }
}

impl<T> Sink<T> for AsyncAppender<T> {
    type Error = Closed;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        (*self).poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        (*self).start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!((*self).poll_flush(cx));
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!((*self).poll_close(cx));
        Poll::Ready(Ok(()))
    }
}

#[pin_project]
struct WorkerFuture<T, S> {
    #[pin]
    inner: S,
    state: Arc<Mutex<State<T>>>,
}

impl<T, S> Future for WorkerFuture<T, S>
where
    S: Sink<T>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let mut state = this.state.lock();

        if state
            .read_waker
            .as_ref()
            .map_or(true, |w| !w.will_wake(cx.waker()))
        {
            state.read_waker = Some(cx.waker().clone());
        }

        while !state.queue.is_empty() {
            drop(state);
            let _ = ready!(this.inner.as_mut().poll_ready(cx));
            state = this.state.lock();

            let value = state.queue.pop_front().unwrap();
            if let Some(waker) = state.write_waker.take() {
                waker.wake();
            }

            drop(state);
            let _ = this.inner.as_mut().start_send(value);
            state = this.state.lock();
        }

        if !state.flushed {
            drop(state);
            let _ = ready!(this.inner.as_mut().poll_flush(cx));
            state = this.state.lock();

            if state.queue.is_empty() {
                state.flushed = true;
                if let Some(waker) = state.write_waker.take() {
                    waker.wake();
                }
            }
        }

        if state.closed && state.flushed {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
