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
pub mod audit_log;
pub mod cancellation;
pub mod catch_unwind;
pub mod client_certificate;
pub mod connection_limit;
pub mod connection_metrics;
pub mod deprecation_header;
pub mod endpoint_health;
pub mod endpoint_metrics;
pub mod error_log;
pub mod graceful_shutdown;
pub mod gzip;
pub mod handler;
pub mod hyper;
pub mod idle_connection;
pub mod keep_alive_header;
pub mod mdc;
pub mod no_caching;
pub mod peer_addr;
pub mod request_id;
pub mod request_log;
pub mod routing;
pub mod server_header;
pub mod server_metrics;
pub mod spans;
#[cfg(test)]
mod test_util;
pub mod tls;
pub mod tls_metrics;
pub mod trace_id_header;
pub mod trace_propagation;
pub mod unverified_jwt;
pub mod web_security;
pub mod witchcraft_mdc;

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

    fn call(&self, req: R) -> impl Future<Output = Self::Response> + Send;
}

impl<S, R> Service<R> for Arc<S>
where
    S: ?Sized + Service<R>,
{
    type Response = S::Response;

    #[inline]
    fn call(&self, req: R) -> impl Future<Output = Self::Response> + Send {
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
