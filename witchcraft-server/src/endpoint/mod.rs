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
use crate::health::endpoint_500s::EndpointHealth;
use crate::server::RawBody;
use crate::service::endpoint_metrics::EndpointMetrics;
use crate::service::handler::BodyWriteAborted;
use async_trait::async_trait;
use bytes::Bytes;
use conjure_http::server::EndpointMetadata;
use futures_util::future::BoxFuture;
use http::{Request, Response};
use http_body::combinators::BoxBody;
use std::sync::Arc;

pub mod conjure;
pub mod errors;
pub mod extended_path;

#[async_trait]
pub trait WitchcraftEndpoint: EndpointMetadata {
    fn metrics(&self) -> Option<&EndpointMetrics>;

    fn health(&self) -> Option<&Arc<EndpointHealth>>;

    async fn handle(&self, req: Request<RawBody>) -> Response<BoxBody<Bytes, BodyWriteAborted>>;
}

impl<T> WitchcraftEndpoint for Box<T>
where
    T: ?Sized + WitchcraftEndpoint,
{
    fn metrics(&self) -> Option<&EndpointMetrics> {
        (**self).metrics()
    }

    fn health(&self) -> Option<&Arc<EndpointHealth>> {
        (**self).health()
    }

    // manually implementing to avoid double boxing the inner future
    fn handle<'life0, 'async_trait>(
        &'life0 self,
        req: Request<RawBody>,
    ) -> BoxFuture<Response<BoxBody<Bytes, BodyWriteAborted>>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        (**self).handle(req)
    }
}
