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
use std::future::Future;
use std::sync::Arc;

pub mod accept;
pub mod connection_limit;
pub mod connection_metrics;
pub mod handler;
pub mod hyper;
pub mod idle_connection;
pub mod routing;
pub mod tls;
pub mod tls_metrics;

// This infrastructure is adapted from `tower`, with a few changes:
//
// * Layer::layer takes self by value.
// * Service::Error has been removed.
// * Service::poll_ready has been removed.
// * Service::call takes self by shared reference rather than mutable reference.
pub trait Layer<S> {
    type Service;

    fn layer(self, inner: S) -> Self::Service;
}

pub trait Service<R> {
    type Response;

    type Future: Future<Output = Self::Response>;

    fn call(&self, req: R) -> Self::Future;
}

impl<S, R> Service<R> for Arc<S>
where
    S: ?Sized + Service<R>,
{
    type Response = S::Response;

    type Future = S::Future;

    #[inline]
    fn call(&self, req: R) -> Self::Future {
        (**self).call(req)
    }
}

pub struct Identity;

impl<S> Layer<S> for Identity {
    type Service = S;

    fn layer(self, inner: S) -> Self::Service {
        inner
    }
}

pub struct Stack<T, U> {
    inner: U,
    outer: T,
}

impl<T, U, S> Layer<S> for Stack<T, U>
where
    T: Layer<U::Service>,
    U: Layer<S>,
{
    type Service = T::Service;

    fn layer(self, inner: S) -> Self::Service {
        let inner = self.inner.layer(inner);
        self.outer.layer(inner)
    }
}

pub struct ServiceBuilder<L> {
    layer: L,
}

impl ServiceBuilder<Identity> {
    pub fn new() -> Self {
        ServiceBuilder { layer: Identity }
    }
}

impl<L> ServiceBuilder<L> {
    pub fn layer<T>(self, layer: T) -> ServiceBuilder<Stack<L, T>> {
        ServiceBuilder {
            layer: Stack {
                inner: layer,
                outer: self.layer,
            },
        }
    }

    pub fn service<S>(self, service: S) -> L::Service
    where
        L: Layer<S>,
    {
        self.layer.layer(service)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn service_fn<F>(f: F) -> ServiceFn<F> {
    ServiceFn(f)
}

struct ServiceFn<F>(F);

impl<T, F, I, O> Service<I> for ServiceFn<T>
where
    T: Fn(I) -> F,
    F: Future<Output = O>,
{
    type Response = O;

    type Future = F;

    fn call(&self, req: I) -> Self::Future {
        self.0(req)
    }
}
