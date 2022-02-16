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
use crate::endpoint::WitchcraftEndpoint;
use crate::health::endpoint_500s::EndpointHealth;
use crate::server::RawBody;
use crate::service::endpoint_metrics::EndpointMetrics;
use crate::service::handler::BodyWriteAborted;
use bytes::Bytes;
use conjure_http::server::{EndpointMetadata, PathSegment};
use futures_util::future::BoxFuture;
use http::{Method, Request, Response};
use http_body::combinators::BoxBody;
use std::borrow::Cow;
use std::sync::Arc;

/// A [`WitchcraftEndpoint`] which prepends path components to an inner endpoint.
pub struct ExtendedPathEndpoint<T> {
    inner: T,
    path: Vec<PathSegment>,
    template: String,
}

impl<T> ExtendedPathEndpoint<T>
where
    T: EndpointMetadata,
{
    pub fn new(inner: T, path_prefix: &str) -> Self {
        debug_assert!(path_prefix.starts_with('/') && !path_prefix.ends_with('/'));

        let path = path_prefix
            .strip_prefix('/')
            .unwrap()
            .split('/')
            .map(|s| PathSegment::Literal(Cow::Owned(s.to_string())))
            .chain(inner.path().iter().cloned())
            .collect();

        let template = format!("{path_prefix}{}", inner.template());

        ExtendedPathEndpoint {
            inner,
            path,
            template,
        }
    }
}

impl<T> EndpointMetadata for ExtendedPathEndpoint<T>
where
    T: EndpointMetadata,
{
    fn method(&self) -> Method {
        self.inner.method()
    }

    fn path(&self) -> &[PathSegment] {
        &self.path
    }

    fn template(&self) -> &str {
        &self.template
    }

    fn service_name(&self) -> &str {
        self.inner.service_name()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn deprecated(&self) -> Option<&str> {
        self.inner.deprecated()
    }
}

impl<T> WitchcraftEndpoint for ExtendedPathEndpoint<T>
where
    T: WitchcraftEndpoint,
{
    fn metrics(&self) -> Option<&EndpointMetrics> {
        self.inner.metrics()
    }

    fn health(&self) -> Option<&Arc<EndpointHealth>> {
        self.inner.health()
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
        self.inner.handle(req)
    }
}
