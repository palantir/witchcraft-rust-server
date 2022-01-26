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
use futures_util::future::BoxFuture;
use futures_util::stream::FuturesUnordered;
use futures_util::{ready, Stream};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[pin_project]
pub struct ShutdownHooks {
    #[pin]
    hooks: FuturesUnordered<BoxFuture<'static, ()>>,
}

impl ShutdownHooks {
    pub fn new() -> Self {
        ShutdownHooks {
            hooks: FuturesUnordered::new(),
        }
    }

    pub fn push<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static + Send,
    {
        self.hooks.push(Box::pin(future));
    }
}

impl Future for ShutdownHooks {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        while let Some(()) = ready!(self.as_mut().project().hooks.poll_next(cx)) {}

        Poll::Ready(())
    }
}
