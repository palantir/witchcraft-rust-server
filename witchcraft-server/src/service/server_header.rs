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
use conjure_error::Error;
use http::header::SERVER;
use http::{HeaderValue, Response};
use witchcraft_server_config::install::InstallConfig;

/// A layer which adds a `Server` header to responses.
pub struct ServerHeaderLayer {
    value: HeaderValue,
}

impl ServerHeaderLayer {
    pub fn new(config: &InstallConfig) -> Result<Self, Error> {
        let value = HeaderValue::try_from(format!(
            "{}/{}",
            config.product_name(),
            config.product_version()
        ))
        .map_err(Error::internal_safe)?;

        Ok(ServerHeaderLayer { value })
    }
}

impl<S> Layer<S> for ServerHeaderLayer {
    type Service = ServerHeaderService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ServerHeaderService {
            inner,
            value: self.value,
        }
    }
}

pub struct ServerHeaderService<S> {
    inner: S,
    value: HeaderValue,
}

impl<S, R, B> Service<R> for ServerHeaderService<S>
where
    S: Service<R, Response = Response<B>>,
{
    type Response = S::Response;

    async fn call(&self, req: R) -> Self::Response {
        let mut response = self.inner.call(req).await;
        response.headers_mut().insert(SERVER, self.value.clone());
        response
    }
}
