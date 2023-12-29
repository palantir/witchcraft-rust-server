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
use witchcraft_log::info;

/// A layer which logs a message when the inner service's future is dropped before completion.
pub struct CancellationLayer;

impl<S> Layer<S> for CancellationLayer {
    type Service = CancellationService<S>;

    fn layer(self, inner: S) -> Self::Service {
        CancellationService { inner }
    }
}

pub struct CancellationService<S> {
    inner: S,
}

impl<S, R> Service<R> for CancellationService<S>
where
    S: Service<R> + Sync,
    R: Send,
{
    type Response = S::Response;

    async fn call(&self, req: R) -> Self::Response {
        let mut guard = DropGuard { complete: false };
        let r = self.inner.call(req).await;
        guard.complete = true;
        r
    }
}

struct DropGuard {
    complete: bool,
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        if !self.complete {
            info!("request cancelled during processing");
        }
    }
}
