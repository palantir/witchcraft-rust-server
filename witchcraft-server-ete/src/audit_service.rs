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

use conjure_error::Error;
use conjure_http::server::{
    AsyncEndpoint, AsyncResponseBody, AsyncService, BoxAsyncEndpoint, ConjureRuntime, Endpoint,
    EndpointMetadata, PathSegment, ResponseBody, Service,
};
use conjure_object::{Utc, Uuid};
use http::{Extensions, Method, Request, Response};
use std::borrow::Cow;
use std::sync::Arc;
use witchcraft_server::extensions::AuditLogEntry;
use witchcraft_server::logging::api::{AuditLogV3, AuditProducer, AuditResult};

pub struct AuditService;

impl<I, O> Service<I, O> for AuditService {
    fn endpoints(&self, _: &Arc<ConjureRuntime>) -> Vec<Box<dyn Endpoint<I, O> + Sync + Send>> {
        vec![Box::new(AuditEndpoint)]
    }
}

impl<I, O> AsyncService<I, O> for AuditService
where
    I: 'static + Send,
{
    fn endpoints(&self, _: &Arc<ConjureRuntime>) -> Vec<BoxAsyncEndpoint<'static, I, O>> {
        vec![BoxAsyncEndpoint::new(AuditEndpoint)]
    }
}

struct AuditEndpoint;

impl AuditEndpoint {
    fn audit_log_entry(&self) -> AuditLogEntry {
        let log = AuditLogV3::builder()
            .type_("audit.3")
            .deployment("deployment")
            .host("host")
            .product("product")
            .product_version("product_version")
            .producer_type(AuditProducer::Server)
            .event_id(Uuid::nil())
            .time(Utc::now())
            .name("TEST")
            .result(AuditResult::Success)
            .build();
        AuditLogEntry::v3(log)
    }
}

impl EndpointMetadata for AuditEndpoint {
    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> &[PathSegment] {
        &[PathSegment::Literal(Cow::Borrowed("audit"))]
    }

    fn template(&self) -> &str {
        "/audit"
    }

    fn service_name(&self) -> &str {
        "AuditService"
    }

    fn name(&self) -> &str {
        "audit"
    }

    fn deprecated(&self) -> Option<&str> {
        None
    }
}

impl<I, O> Endpoint<I, O> for AuditEndpoint {
    fn handle(
        &self,
        _: Request<I>,
        response_extensions: &mut Extensions,
    ) -> Result<Response<ResponseBody<O>>, Error> {
        response_extensions.insert(self.audit_log_entry());
        Ok(Response::new(ResponseBody::Empty))
    }
}

impl<I, O> AsyncEndpoint<I, O> for AuditEndpoint
where
    I: 'static + Send,
{
    async fn handle(
        &self,
        _: Request<I>,
        response_extensions: &mut Extensions,
    ) -> Result<Response<AsyncResponseBody<O>>, Error> {
        response_extensions.insert(self.audit_log_entry());
        Ok(Response::new(AsyncResponseBody::Empty))
    }
}
