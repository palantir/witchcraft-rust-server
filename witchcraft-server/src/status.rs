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
use crate::health::HealthCheckRegistry;
use crate::readiness::ReadinessCheckRegistry;
use crate::{RequestBody, ResponseWriter};
use async_trait::async_trait;
use bytes::Bytes;
use conjure_error::{Error, PermissionDenied};
use conjure_http::server::{
    AsyncEndpoint, AsyncResponseBody, AsyncService, ConjureRuntime, EndpointMetadata, PathSegment,
};
use conjure_serde::json;
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::{Extensions, HeaderValue, Method, Request, Response, StatusCode};
use refreshable::Refreshable;
use std::borrow::Cow;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::task;
use witchcraft_server_config::runtime::RuntimeConfig;

#[allow(clippy::declare_interior_mutable_const)]
const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");

struct State {
    health_check_auth: Refreshable<String, Error>,
    health_checks: Arc<HealthCheckRegistry>,
    readiness_checks: Arc<ReadinessCheckRegistry>,
}

// Manually implemented because readiness isn't fully definable in Conjure.
pub struct StatusEndpoints {
    state: Arc<State>,
}

impl StatusEndpoints {
    pub fn new<R>(
        runtime_config: &Refreshable<R, Error>,
        health_checks: &Arc<HealthCheckRegistry>,
        readiness_checks: &Arc<ReadinessCheckRegistry>,
    ) -> Self
    where
        R: AsRef<RuntimeConfig> + PartialEq + 'static + Sync + Send,
    {
        StatusEndpoints {
            state: Arc::new(State {
                health_check_auth: runtime_config
                    .map(|c| format!("Bearer {}", c.as_ref().health_checks().shared_secret())),
                health_checks: health_checks.clone(),
                readiness_checks: readiness_checks.clone(),
            }),
        }
    }
}

impl AsyncService<RequestBody, ResponseWriter> for StatusEndpoints {
    fn endpoints(
        &self,
        _: &Arc<ConjureRuntime>,
    ) -> Vec<Box<dyn AsyncEndpoint<RequestBody, ResponseWriter> + Sync + Send>> {
        vec![
            Box::new(LivenessEndpoint),
            Box::new(HealthEndpoint {
                state: self.state.clone(),
            }),
            Box::new(ReadinessEndpoint {
                state: self.state.clone(),
            }),
        ]
    }
}

struct LivenessEndpoint;

impl EndpointMetadata for LivenessEndpoint {
    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> &[PathSegment] {
        &[
            PathSegment::Literal(Cow::Borrowed("status")),
            PathSegment::Literal(Cow::Borrowed("liveness")),
        ]
    }

    fn template(&self) -> &str {
        "/status/liveness"
    }

    fn service_name(&self) -> &str {
        "StatusService"
    }

    fn name(&self) -> &str {
        "liveness"
    }

    fn deprecated(&self) -> Option<&str> {
        None
    }
}

#[async_trait]
impl AsyncEndpoint<RequestBody, ResponseWriter> for LivenessEndpoint {
    async fn handle(
        &self,
        _: Request<RequestBody>,
        _: &mut Extensions,
    ) -> Result<Response<AsyncResponseBody<ResponseWriter>>, Error> {
        let mut response = Response::new(AsyncResponseBody::Empty);
        *response.status_mut() = StatusCode::NO_CONTENT;
        Ok(response)
    }
}

struct HealthEndpoint {
    state: Arc<State>,
}

impl EndpointMetadata for HealthEndpoint {
    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> &[PathSegment] {
        &[
            PathSegment::Literal(Cow::Borrowed("status")),
            PathSegment::Literal(Cow::Borrowed("health")),
        ]
    }

    fn template(&self) -> &str {
        "/status/health"
    }

    fn service_name(&self) -> &str {
        "StatusService"
    }

    fn name(&self) -> &str {
        "health"
    }

    fn deprecated(&self) -> Option<&str> {
        None
    }
}

#[async_trait]
impl AsyncEndpoint<RequestBody, ResponseWriter> for HealthEndpoint {
    async fn handle(
        &self,
        req: Request<RequestBody>,
        _: &mut Extensions,
    ) -> Result<Response<AsyncResponseBody<ResponseWriter>>, Error> {
        let authorization = match req.headers().get(AUTHORIZATION) {
            Some(authorization) => authorization,
            None => {
                return Err(Error::service_safe(
                    "health check secret missing",
                    PermissionDenied::new(),
                ))
            }
        };

        let expected = self.state.health_check_auth.get();
        if !bool::from(authorization.as_bytes().ct_eq(expected.as_bytes())) {
            return Err(Error::service_safe(
                "invalid health check secret",
                PermissionDenied::new(),
            ));
        }

        let health_checks = self.state.health_checks.run_checks();
        let health_checks = json::to_vec(&health_checks).unwrap();
        let mut response = Response::new(AsyncResponseBody::Fixed(Bytes::from(health_checks)));
        response
            .headers_mut()
            .insert(CONTENT_TYPE, APPLICATION_JSON);

        Ok(response)
    }
}

struct ReadinessEndpoint {
    state: Arc<State>,
}

impl EndpointMetadata for ReadinessEndpoint {
    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> &[PathSegment] {
        &[
            PathSegment::Literal(Cow::Borrowed("status")),
            PathSegment::Literal(Cow::Borrowed("readiness")),
        ]
    }

    fn template(&self) -> &str {
        "/status/readiness"
    }

    fn service_name(&self) -> &str {
        "StatusService"
    }

    fn name(&self) -> &str {
        "readiness"
    }

    fn deprecated(&self) -> Option<&str> {
        None
    }
}

#[async_trait]
impl AsyncEndpoint<RequestBody, ResponseWriter> for ReadinessEndpoint {
    async fn handle(
        &self,
        _: Request<RequestBody>,
        _: &mut Extensions,
    ) -> Result<Response<AsyncResponseBody<ResponseWriter>>, Error> {
        let readiness_checks = task::spawn_blocking({
            let readiness_checks = self.state.readiness_checks.clone();
            move || readiness_checks.run_checks()
        })
        .await
        .unwrap();

        let status = if readiness_checks.values().all(|r| r.successful) {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };

        let readiness_checks = json::to_vec(&readiness_checks).unwrap();
        let mut response = Response::new(AsyncResponseBody::Fixed(Bytes::from(readiness_checks)));
        *response.status_mut() = status;
        response
            .headers_mut()
            .insert(CONTENT_TYPE, APPLICATION_JSON);

        Ok(response)
    }
}
