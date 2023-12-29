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
use crate::service::{Layer, Service};
use http::Request;
use std::fmt;

#[derive(Copy, Clone)]
pub struct RequestId {
    id: [u8; 8],
}

impl RequestId {
    pub fn random() -> Self {
        RequestId { id: rand::random() }
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.id {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// A layer which adds a unique [`RequestId`] to each request's extensions.
pub struct RequestIdLayer;

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

pub struct RequestIdService<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for RequestIdService<S>
where
    S: Service<Request<B>> + Sync,
    B: Send,
{
    type Response = S::Response;

    async fn call(&self, mut req: Request<B>) -> Self::Response {
        req.extensions_mut().insert(RequestId::random());
        self.inner.call(req).await
    }
}
