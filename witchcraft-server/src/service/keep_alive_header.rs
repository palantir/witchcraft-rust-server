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
use http::header::HeaderName;
use http::{HeaderValue, Request, Response, Version};
use witchcraft_server_config::install::InstallConfig;

const KEEP_ALIVE: HeaderName = HeaderName::from_static("keep-alive");

/// A layer which adds a `Keep-Alive` header to responses.
pub struct KeepAliveHeaderLayer {
    value: Option<HeaderValue>,
}

impl KeepAliveHeaderLayer {
    pub fn new(config: &InstallConfig) -> Self {
        KeepAliveHeaderLayer {
            value: config
                .server()
                .idle_connection_timeout()
                .map(|d| HeaderValue::try_from(format!("timeout={}", d.as_secs())).unwrap()),
        }
    }
}

impl<S> Layer<S> for KeepAliveHeaderLayer {
    type Service = KeepAliveHeaderService<S>;

    fn layer(self, inner: S) -> Self::Service {
        KeepAliveHeaderService {
            inner,
            value: self.value,
        }
    }
}

pub struct KeepAliveHeaderService<S> {
    inner: S,
    value: Option<HeaderValue>,
}

impl<S, B1, B2> Service<Request<B1>> for KeepAliveHeaderService<S>
where
    S: Service<Request<B1>, Response = Response<B2>>,
{
    type Response = S::Response;

    async fn call(&self, req: Request<B1>) -> Self::Response {
        // https://datatracker.ietf.org/doc/html/rfc7540#section-8.1.2.2
        let value = match req.version() {
            Version::HTTP_09 | Version::HTTP_10 | Version::HTTP_11 => self.value.clone(),
            _ => None,
        };

        let mut response = self.inner.call(req).await;
        if let Some(value) = value {
            response.headers_mut().insert(KEEP_ALIVE, value);
        }

        response
    }
}
