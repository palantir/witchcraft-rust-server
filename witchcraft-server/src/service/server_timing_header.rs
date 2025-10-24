// Copyright 2025 Palantir Technologies, Inc.
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

use http::{HeaderName, HeaderValue, Response};
use tokio::time::Instant;

use crate::service::{Layer, Service};

const SERVER_TIMING: HeaderName = HeaderName::from_static("server-timing");

/// A layer which adds a `Server-Timing` header into responses.
pub struct ServerTimingHeaderLayer;

impl<S> Layer<S> for ServerTimingHeaderLayer {
    type Service = ServerTimingHeaderService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ServerTimingHeaderService { inner }
    }
}

pub struct ServerTimingHeaderService<S> {
    inner: S,
}

impl<S, R, B> Service<R> for ServerTimingHeaderService<S>
where
    S: Service<R, Response = Response<B>> + Sync,
    R: Send,
{
    type Response = Response<B>;

    async fn call(&self, req: R) -> Self::Response {
        let start = Instant::now();
        let mut response = self.inner.call(req).await;

        // not going through float conversion and formatting for better efficiency
        // similarly, going to assume we haven't spent more than 2^64 microseconds handling the request
        let dur_micros = start.elapsed().as_micros() as u64;
        let value = format!(
            "server;dur={}.{}{}{}",
            dur_micros / 1_000,
            (dur_micros % 1_000) / 100,
            (dur_micros % 100) / 10,
            (dur_micros % 10),
        );
        let value = HeaderValue::try_from(value).unwrap();

        response.headers_mut().insert(SERVER_TIMING, value);

        response
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::time;

    use crate::service::test_util::service_fn;

    use super::*;

    #[tokio::test(start_paused = true)]
    async fn basics() {
        let service = ServerTimingHeaderLayer.layer(service_fn(|delay| async move {
            time::advance(delay).await;
            Response::new(())
        }));

        let response = service.call(Duration::from_micros(1234)).await;
        assert_eq!(
            response.headers().get(SERVER_TIMING).unwrap(),
            "server;dur=1.234"
        );
    }
}
