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
use crate::logging::logger::Payload;
use crate::shutdown_hooks::ShutdownHooks;
use core::fmt;
use futures_sink::Sink;
use futures_util::ready;
use parking_lot::Mutex;
use pin_project::pin_project;
use std::collections::VecDeque;
use std::error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use tokio::task::{self, JoinHandle};
use witchcraft_metrics::{MetricId, MetricRegistry};

#[derive(Debug)]
pub struct Closed;

impl fmt::Display for Closed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Sink has already closed")
    }
}

impl error::Error for Closed {}

const QUEUE_LIMIT: usize = 10_000;

struct State<T> {
    queue: VecDeque<Payload<T>>,
    write_waker: Option<Waker>,
    read_waker: Option<Waker>,
    flushed: bool,
    closed: bool,
}

impl<T> State<T> {
    fn ready(&self) -> bool {
        self.queue.len() < QUEUE_LIMIT
    }

    fn start_send(&mut self, item: Payload<T>) {
        debug_assert!(self.queue.len() < QUEUE_LIMIT);

        self.queue.push_back(item);
        self.flushed = false;
        if let Some(waker) = self.read_waker.take() {
            waker.wake();
        }
    }

    fn start_close(&mut self) {
        if self.closed {
            return;
        }

        self.closed = true;
        if let Some(waker) = self.read_waker.take() {
            waker.wake();
        }

        if let Some(waker) = self.write_waker.take() {
            waker.wake();
        }
    }
}

pub struct AsyncAppender<T> {
    state: Arc<Mutex<State<T>>>,
}

impl<T> Clone for AsyncAppender<T> {
    fn clone(&self) -> Self {
        AsyncAppender {
            state: self.state.clone(),
        }
    }
}

impl<T> Drop for AsyncAppender<T> {
    fn drop(&mut self) {
        self.state.lock().start_close();
    }
}

impl<T> AsyncAppender<T> {
    pub fn new<S>(inner: S, metrics: &MetricRegistry, hooks: &mut ShutdownHooks) -> Self
    where
        S: Sink<Payload<T>> + 'static + Send,
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

        let handle = task::spawn({
            let state = state.clone();
            WorkerFuture { state, inner }
        });

        hooks.push(ShutdownFuture {
            state: state.clone(),
            handle,
        });

        AsyncAppender { state }
    }

    pub fn try_send(&self, item: Payload<T>) -> Result<(), Payload<T>> {
        let mut state = self.state.lock();

        if state.closed || !state.ready() {
            return Err(item);
        }
        state.start_send(item);
        Ok(())
    }
}

impl<T> Sink<Payload<T>> for AsyncAppender<T> {
    type Error = Closed;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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

    fn start_send(self: Pin<&mut Self>, item: Payload<T>) -> Result<(), Self::Error> {
        let mut state = self.state.lock();

        if state.closed {
            return Err(Closed);
        }

        state.start_send(item);
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut state = self.state.lock();

        if state.flushed {
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

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.state.lock().start_close();
        ready!(self.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }
}

#[pin_project]
struct ShutdownFuture<T> {
    state: Arc<Mutex<State<T>>>,
    #[pin]
    handle: JoinHandle<()>,
}

impl<T> Future for ShutdownFuture<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        this.state.lock().start_close();
        let _ = ready!(this.handle.poll(cx));

        Poll::Ready(())
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
    S: Sink<Payload<T>>,
{
    type Output = ();

    // There is some subtlety here to avoid holding the lock across calls to the inner sink.
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        let mut state = this.state.lock();

        while !state.queue.is_empty() {
            drop(state);
            let _ = ready!(this.inner.as_mut().poll_ready(cx));
            state = this.state.lock();

            // Even though we've released the lock since seeing that the queue was not empty, we know that fact hasn't
            // changed since this task is the only thing that removes items from the queue.
            let value = state.queue.pop_front().unwrap();
            if let Some(waker) = state.write_waker.take() {
                waker.wake();
            }

            drop(state);
            let _ = this.inner.as_mut().start_send(value);
            state = this.state.lock();
        }

        // Doing this after processing all pending requests avoids some extra wakeups if we're waiting on the inner
        // sink while the queue is being written to.
        if state
            .read_waker
            .as_ref()
            .map_or(true, |w| !w.will_wake(cx.waker()))
        {
            state.read_waker = Some(cx.waker().clone());
        }

        if !state.flushed {
            drop(state);
            let _ = ready!(this.inner.as_mut().poll_flush(cx));
            state = this.state.lock();

            // Since we've released the lock after seeing that the queue was empty above, we don't want to claim to a
            // writer task that we've flushed the queue unless it is still empty. If new items have been added to it,
            // this task will be immediately re-woken to handle that new data and then attempt to flush again.
            if state.queue.is_empty() {
                state.flushed = true;
                if let Some(waker) = state.write_waker.take() {
                    waker.wake();
                }
            }
        }

        if !state.flushed || !state.closed {
            return Poll::Pending;
        }

        drop(state);
        let _ = ready!(this.inner.as_mut().poll_close(cx));

        Poll::Ready(())
    }
}
