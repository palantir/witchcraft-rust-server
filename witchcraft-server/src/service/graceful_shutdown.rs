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
use crate::service::hyper::GracefulShutdown;
use crate::service::{Layer, Service};
use crate::shutdown_hooks::ShutdownHooks;
use futures_util::future::{self, BoxFuture, Fuse, FusedFuture};
use futures_util::FutureExt;
use parking_lot::Mutex;
use pin_project::{pin_project, pinned_drop};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use tokio_util::sync::CancellationToken;

/// A layer which registers a shutdown hook to initiate a graceful shutdown of all futures returned by the delegate
/// service, and waits for them to complete.
pub struct GracefulShutdownLayer {
    shared: Arc<Shared>,
}

impl GracefulShutdownLayer {
    pub fn new(hooks: &mut ShutdownHooks) -> Self {
        let shared = Arc::new(Shared {
            cancellation_token: CancellationToken::new(),
            state: Mutex::new(State {
                connections: 0,
                waker: None,
            }),
        });

        hooks.push({
            let shared = shared.clone();
            async move {
                shared.cancellation_token.cancel();

                future::poll_fn(|cx| {
                    let mut state = shared.state.lock();
                    if state
                        .waker
                        .as_ref()
                        .map_or(true, |w| !cx.waker().will_wake(w))
                    {
                        state.waker = Some(cx.waker().clone());
                    }

                    if state.connections == 0 {
                        Poll::Ready(())
                    } else {
                        Poll::Pending
                    }
                })
                .await
            }
        });

        GracefulShutdownLayer { shared }
    }
}

impl<S> Layer<S> for GracefulShutdownLayer {
    type Service = GracefulShutdownService<S>;

    fn layer(self, inner: S) -> Self::Service {
        GracefulShutdownService {
            inner,
            shared: self.shared,
        }
    }
}

pub struct GracefulShutdownService<S> {
    inner: S,
    shared: Arc<Shared>,
}

impl<S, R> Service<R> for GracefulShutdownService<S>
where
    S: Service<R>,
    S::Future: GracefulShutdown,
{
    type Response = S::Response;

    type Future = GracefulShutdownFuture<S::Future>;

    fn call(&self, req: R) -> Self::Future {
        // Do this first so we don't leak a connection if it panics
        let inner = self.inner.call(req);

        let shutdown: BoxFuture<()> = Box::pin({
            let shared = self.shared.clone();
            async move { shared.cancellation_token.cancelled().await }
        }) as _;

        self.shared.state.lock().connections += 1;
        GracefulShutdownFuture {
            inner,
            shutdown: shutdown.fuse(),
            shared: self.shared.clone(),
        }
    }
}

#[pin_project(PinnedDrop)]
pub struct GracefulShutdownFuture<F> {
    #[pin]
    inner: F,
    #[pin]
    shutdown: Fuse<BoxFuture<'static, ()>>,
    shared: Arc<Shared>,
}

#[pinned_drop]
impl<F> PinnedDrop for GracefulShutdownFuture<F> {
    fn drop(self: Pin<&mut Self>) {
        let mut state = self.shared.state.lock();
        state.connections -= 1;
        if state.connections == 0 {
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        }
    }
}

impl<F> Future for GracefulShutdownFuture<F>
where
    F: Future + GracefulShutdown,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        if !this.shutdown.is_terminated() && this.shutdown.poll(cx).is_ready() {
            this.inner.as_mut().graceful_shutdown();
        }

        this.inner.poll(cx)
    }
}

struct State {
    connections: usize,
    waker: Option<Waker>,
}

struct Shared {
    cancellation_token: CancellationToken,
    state: Mutex<State>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::test_util::service_fn;
    use std::time::Duration;
    use tokio::task;
    use tokio::time::{self, Instant, Sleep};

    #[pin_project]
    struct TestFuture {
        #[pin]
        sleep: Sleep,
        shutdown_delay: Duration,
    }

    impl TestFuture {
        fn new(normal_delay: Duration, shutdown_delay: Duration) -> Self {
            TestFuture {
                sleep: time::sleep(normal_delay),
                shutdown_delay,
            }
        }
    }

    impl Future for TestFuture {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            self.project().sleep.poll(cx)
        }
    }

    impl GracefulShutdown for TestFuture {
        fn graceful_shutdown(self: Pin<&mut Self>) {
            let this = self.project();
            this.sleep.reset(Instant::now() + *this.shutdown_delay);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn basic() {
        let mut hooks = ShutdownHooks::new();

        let service = GracefulShutdownLayer::new(&mut hooks).layer(service_fn(|f| f));

        let a = task::spawn(service.call(TestFuture::new(
            Duration::from_secs(1),
            Duration::from_secs(1000),
        )));
        let b = task::spawn(service.call(TestFuture::new(
            Duration::from_secs(1000),
            Duration::from_secs(1),
        )));
        let c = task::spawn(service.call(TestFuture::new(
            Duration::from_secs(1000),
            Duration::from_secs(2),
        )));

        let start = Instant::now();
        a.await.unwrap();
        assert_eq!(start.elapsed(), Duration::from_secs(1));

        let start = Instant::now();
        hooks.await;
        assert_eq!(start.elapsed(), Duration::from_secs(2));

        let start = Instant::now();
        b.await.unwrap();
        c.await.unwrap();
        assert_eq!(start.elapsed(), Duration::from_secs(0));
    }
}
