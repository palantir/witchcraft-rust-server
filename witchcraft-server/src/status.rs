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
use crate::health::api::HealthStatus;
use crate::health::HealthCheckRegistry;
use crate::readiness::{ReadinessCheckMetadata, ReadinessCheckRegistry};
use conjure_error::{Error, PermissionDenied};
use conjure_http::server::{
    AsyncResponseBody, AsyncSerializeResponse, ConjureRuntime, StdResponseSerializer,
};
use conjure_http::{conjure_endpoints, endpoint};
use conjure_object::BearerToken;
use http::{HeaderMap, Response, StatusCode};
use refreshable::Refreshable;
use std::collections::BTreeMap;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::task;
use witchcraft_server_config::runtime::RuntimeConfig;

#[conjure_endpoints]
pub trait StatusService {
    #[endpoint(path = "/status/liveness", method = GET)]
    async fn liveness(&self) -> Result<(), Error>;

    #[endpoint(path = "/status/readiness", method = GET, produces = LivenessResponseSerializer)]
    async fn readiness(&self) -> Result<BTreeMap<String, ReadinessCheckMetadata>, Error>;

    #[endpoint(path = "/status/health", method = GET, produces = StdResponseSerializer)]
    async fn health(&self, #[auth] token: BearerToken) -> Result<HealthStatus, Error>;
}

pub struct StatusResource {
    health_check_secret: Refreshable<String, Error>,
    health_checks: Arc<HealthCheckRegistry>,
    readiness_checks: Arc<ReadinessCheckRegistry>,
}

impl StatusResource {
    pub fn new<R>(
        runtime_config: &Refreshable<R, Error>,
        health_checks: &Arc<HealthCheckRegistry>,
        readiness_checks: &Arc<ReadinessCheckRegistry>,
    ) -> Self
    where
        R: AsRef<RuntimeConfig> + PartialEq + 'static + Sync + Send,
    {
        StatusResource {
            health_check_secret: runtime_config
                .map(|c| c.as_ref().health_checks().shared_secret().to_string()),
            health_checks: health_checks.clone(),
            readiness_checks: readiness_checks.clone(),
        }
    }
}

impl StatusService for StatusResource {
    async fn liveness(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn readiness(&self) -> Result<BTreeMap<String, ReadinessCheckMetadata>, Error> {
        let readiness_checks = task::spawn_blocking({
            let readiness_checks = self.readiness_checks.clone();
            move || readiness_checks.run_checks()
        })
        .await
        .unwrap();

        Ok(readiness_checks)
    }

    async fn health(&self, token: BearerToken) -> Result<HealthStatus, Error> {
        let expected = self.health_check_secret.get();
        if !bool::from(token.as_str().as_bytes().ct_eq(expected.as_bytes())) {
            return Err(Error::service_safe(
                "invalid health check secret",
                PermissionDenied::new(),
            ));
        }

        Ok(self.health_checks.run_checks())
    }
}

enum LivenessResponseSerializer {}

impl<W> AsyncSerializeResponse<BTreeMap<String, ReadinessCheckMetadata>, W>
    for LivenessResponseSerializer
{
    fn serialize(
        runtime: &ConjureRuntime,
        request_headers: &HeaderMap,
        value: BTreeMap<String, ReadinessCheckMetadata>,
    ) -> Result<Response<AsyncResponseBody<W>>, Error> {
        let status = if value.values().all(|r| r.successful) {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };

        let mut response = StdResponseSerializer::serialize(runtime, request_headers, value)?;
        *response.status_mut() = status;

        Ok(response)
    }
}
