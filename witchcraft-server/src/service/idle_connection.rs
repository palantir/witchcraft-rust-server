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
use crate::service::hyper::{GracefulShutdown, NewConnection};
use crate::service::{Layer, Service, Stack};
use futures_util::ready;
use http::{HeaderMap, Response};
use http_body::Body;
use parking_lot::Mutex;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::Duration;
use tokio::time::{self, Instant, Sleep};
use witchcraft_server_config::install::InstallConfig;

use super::hyper::ShutdownService;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// A layer which triggers a graceful shutdown of connections that have been idle for a certain period of time.
///
/// The implementation is unfortunately somewhat tightly coupled to the hyper layer with the `NewConnection` type.
pub struct IdleConnectionLayer {
    idle_timeout: Duration,
}

impl IdleConnectionLayer {
    pub fn new(config: &InstallConfig) -> Self {
        IdleConnectionLayer {
            idle_timeout: config
                .server()
                .idle_connection_timeout()
                .unwrap_or(DEFAULT_TIMEOUT),
        }
    }
}

impl<S> Layer<S> for IdleConnectionLayer {
    type Service = IdleConnectionService<S>;

    fn layer(self, inner: S) -> Self::Service {
        IdleConnectionService {
            inner,
            idle_timeout: self.idle_timeout,
        }
    }
}

pub struct IdleConnectionService<S> {
    inner: S,
    idle_timeout: Duration,
}

impl<S, R, L> ShutdownService<NewConnection<R, L>> for IdleConnectionService<S>
where
    S: ShutdownService<NewConnection<R, Stack<L, RequestTrackerLayer>>>,
{
    type Response = S::Response;

    fn call(
        &self,
        req: NewConnection<R, L>,
    ) -> impl Future<Output = Self::Response> + GracefulShutdown + Send {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                mode: Mode::Idle,
                waker: None,
                sleep: Box::pin(time::sleep(self.idle_timeout)),
                idle_timeout: self.idle_timeout,
            }),
        });

        IdleConnectionFuture {
            inner: self.inner.call(NewConnection {
                stream: req.stream,
                service_builder: req.service_builder.layer(RequestTrackerLayer {
                    shared: shared.clone(),
                }),
            }),
            shared,
        }
    }
}

#[pin_project]
pub struct IdleConnectionFuture<F> {
    #[pin]
    inner: F,
    shared: Arc<Shared>,
}

impl<F> Future for IdleConnectionFuture<F>
where
    F: Future + GracefulShutdown,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.shared.poll_timed_out(cx).is_ready() {
            self.as_mut().graceful_shutdown();
        }

        self.project().inner.poll(cx)
    }
}

impl<F> GracefulShutdown for IdleConnectionFuture<F>
where
    F: GracefulShutdown,
{
    fn graceful_shutdown(self: Pin<&mut Self>) {
        let this = self.project();

        this.shared.graceful_shutdown();
        this.inner.graceful_shutdown();
    }
}

pub struct RequestTrackerLayer {
    shared: Arc<Shared>,
}

impl<S> Layer<S> for RequestTrackerLayer {
    type Service = RequestTrackerService<S>;

    fn layer(self, inner: S) -> Self::Service {
        RequestTrackerService {
            inner,
            shared: self.shared,
        }
    }
}

pub struct RequestTrackerService<S> {
    inner: S,
    shared: Arc<Shared>,
}

impl<S, R, B> Service<R> for RequestTrackerService<S>
where
    S: Service<R, Response = Response<B>> + Sync,
    R: Send,
{
    type Response = Response<RequestTrackerBody<B>>;

    async fn call(&self, req: R) -> Self::Response {
        self.shared.inc_active();
        let guard = ActiveGuard {
            shared: self.shared.clone(),
        };

        let response = self.inner.call(req).await;

        response.map(|inner| RequestTrackerBody {
            inner,
            _guard: guard,
        })
    }
}

#[pin_project]
pub struct RequestTrackerFuture<F> {
    #[pin]
    inner: F,
    guard: Option<ActiveGuard>,
}

impl<F, B> Future for RequestTrackerFuture<F>
where
    F: Future<Output = Response<B>>,
{
    type Output = Response<RequestTrackerBody<B>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let response = ready!(this.inner.poll(cx));
        let response = response.map(|inner| RequestTrackerBody {
            inner,
            _guard: this.guard.take().unwrap(),
        });
        Poll::Ready(response)
    }
}

#[pin_project]
pub struct RequestTrackerBody<B> {
    #[pin]
    inner: B,
    _guard: ActiveGuard,
}

impl<B> Body for RequestTrackerBody<B>
where
    B: Body,
{
    type Data = B::Data;

    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.project().inner.poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        self.project().inner.poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

struct ActiveGuard {
    shared: Arc<Shared>,
}

impl Drop for ActiveGuard {
    fn drop(&mut self) {
        self.shared.dec_active();
    }
}

enum Mode {
    Active(usize),
    Idle,
    ShuttingDown,
}

struct State {
    mode: Mode,
    waker: Option<Waker>,
    sleep: Pin<Box<Sleep>>,
    idle_timeout: Duration,
}

struct Shared {
    state: Mutex<State>,
}

impl Shared {
    fn poll_timed_out(&self, cx: &mut Context<'_>) -> Poll<()> {
        let mut state = self.state.lock();

        if state
            .waker
            .as_ref()
            .map_or(true, |waker| !cx.waker().will_wake(waker))
        {
            state.waker = Some(cx.waker().clone());
        }

        match state.mode {
            Mode::Idle => state.sleep.as_mut().poll(cx),
            _ => Poll::Pending,
        }
    }

    fn graceful_shutdown(&self) {
        self.state.lock().mode = Mode::ShuttingDown
    }

    fn inc_active(&self) {
        let mut state = self.state.lock();

        match &mut state.mode {
            Mode::Active(num) => *num += 1,
            Mode::Idle => state.mode = Mode::Active(1),
            Mode::ShuttingDown => {}
        }
    }

    fn dec_active(&self) {
        let mut state = self.state.lock();

        if let Mode::Active(num) = &mut state.mode {
            *num -= 1;
            if *num == 0 {
                state.mode = Mode::Idle;
                let deadline = Instant::now() + state.idle_timeout;
                state.sleep.as_mut().reset(deadline);
                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }
            }
        }
    }
}
