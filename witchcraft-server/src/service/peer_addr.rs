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

use crate::extensions::PeerAddr;
use crate::service::hyper::NewConnection;
use crate::service::{Layer, Service, Stack};
use conjure_error::Error;
use http::Request;
use std::net::SocketAddr;

pub trait GetPeerAddr {
    fn peer_addr(&self) -> Result<SocketAddr, Error>;
}

/// A layer which injects the peer's socket address into all requests made over the connection.
pub struct PeerAddrLayer;

impl<S> Layer<S> for PeerAddrLayer {
    type Service = PeerAddrService<S>;

    fn layer(self, inner: S) -> Self::Service {
        PeerAddrService { inner }
    }
}

pub struct PeerAddrService<S> {
    inner: S,
}

impl<S, T, L, R> Service<NewConnection<T, L>> for PeerAddrService<S>
where
    S: Service<NewConnection<T, Stack<L, PeerAddrRequestLayer>>, Response = Result<R, Error>>
        + Sync,
    T: GetPeerAddr + Send,
    L: Send,
{
    type Response = S::Response;

    async fn call(&self, req: NewConnection<T, L>) -> Self::Response {
        let addr = req.stream.peer_addr()?;

        self.inner
            .call(NewConnection {
                stream: req.stream,
                service_builder: req.service_builder.layer(PeerAddrRequestLayer { addr }),
            })
            .await
    }
}

pub struct PeerAddrRequestLayer {
    addr: SocketAddr,
}

impl<S> Layer<S> for PeerAddrRequestLayer {
    type Service = PeerAddrRequestService<S>;

    fn layer(self, inner: S) -> Self::Service {
        PeerAddrRequestService {
            inner,
            addr: self.addr,
        }
    }
}

pub struct PeerAddrRequestService<S> {
    inner: S,
    addr: SocketAddr,
}

impl<S, B> Service<Request<B>> for PeerAddrRequestService<S>
where
    S: Service<Request<B>> + Sync,
    B: Send,
{
    type Response = S::Response;

    async fn call(&self, mut req: Request<B>) -> Self::Response {
        req.extensions_mut().insert(PeerAddr(self.addr));

        self.inner.call(req).await
    }
}
