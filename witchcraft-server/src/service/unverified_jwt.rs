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
use http::header::AUTHORIZATION;
use http::Request;
use witchcraft_jwt::unverified_jwt::UnverifiedJwt;

/// A layer which parses the request's bearer token (without verifying its validity) and adds it to the request's
/// extensions.
pub struct UnverifiedJwtLayer;

impl<S> Layer<S> for UnverifiedJwtLayer {
    type Service = UnverifiedJwtService<S>;

    fn layer(self, inner: S) -> Self::Service {
        UnverifiedJwtService { inner }
    }
}

pub struct UnverifiedJwtService<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for UnverifiedJwtService<S>
where
    S: Service<Request<B>> + Sync,
    B: Send,
{
    type Response = S::Response;

    async fn call(&self, mut req: Request<B>) -> Self::Response {
        if let Some(jwt) = req
            .headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(UnverifiedJwt::parse)
        {
            req.extensions_mut().insert(jwt);
        }

        self.inner.call(req).await
    }
}

