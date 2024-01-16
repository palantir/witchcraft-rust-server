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
use crate::debug::DiagnosticRegistry;
use crate::{RequestBody, ResponseWriter};
use async_trait::async_trait;
use conjure_error::{Error, NotFound, PermissionDenied};
use conjure_http::server::{
    AsyncEndpoint, AsyncResponseBody, AsyncService, ConjureRuntime, EndpointMetadata, PathSegment,
};
use conjure_http::{PathParams, SafeParams};
use http::header::{HeaderName, AUTHORIZATION, CONTENT_TYPE};
use http::{Extensions, HeaderValue, Method, Request, Response};
use once_cell::sync::Lazy;
use refreshable::Refreshable;
use std::borrow::Cow;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::task;
use witchcraft_server_config::runtime::RuntimeConfig;

static SAFE_LOGGABLE: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("safe-loggable"));
#[allow(clippy::declare_interior_mutable_const)]
const TRUE_VALUE: HeaderValue = HeaderValue::from_static("true");
#[allow(clippy::declare_interior_mutable_const)]
const FALSE_VALUE: HeaderValue = HeaderValue::from_static("false");

struct State {
    auth: Refreshable<String, Error>,
    diagnostics: Arc<DiagnosticRegistry>,
}

pub struct DebugEndpoints {
    state: Arc<State>,
}

impl DebugEndpoints {
    pub fn new<R>(
        runtime_config: &Refreshable<R, Error>,
        diagnostics: Arc<DiagnosticRegistry>,
    ) -> Self
    where
        R: AsRef<RuntimeConfig> + PartialEq + 'static + Sync + Send,
    {
        DebugEndpoints {
            state: Arc::new(State {
                auth: runtime_config
                    .map(|c| format!("Bearer {}", c.as_ref().diagnostics().debug_shared_secret())),
                diagnostics,
            }),
        }
    }
}

impl AsyncService<RequestBody, ResponseWriter> for DebugEndpoints {
    fn endpoints(
        &self,
        _: &Arc<ConjureRuntime>,
    ) -> Vec<Box<dyn AsyncEndpoint<RequestBody, ResponseWriter> + Sync + Send>> {
        vec![Box::new(DiagnosticEndpoint {
            state: self.state.clone(),
        })]
    }
}

struct DiagnosticEndpoint {
    state: Arc<State>,
}

impl EndpointMetadata for DiagnosticEndpoint {
    fn method(&self) -> Method {
        Method::GET
    }

    fn path(&self) -> &[PathSegment] {
        &[
            PathSegment::Literal(Cow::Borrowed("debug")),
            PathSegment::Literal(Cow::Borrowed("diagnostic")),
            PathSegment::Parameter {
                name: Cow::Borrowed("diagnosticType"),
                regex: None,
            },
        ]
    }

    fn template(&self) -> &str {
        "/debug/diagnostic/{diagnosticType}"
    }

    fn service_name(&self) -> &str {
        "DebugService"
    }

    fn name(&self) -> &str {
        "diagnostic"
    }

    fn deprecated(&self) -> Option<&str> {
        None
    }
}

#[async_trait]
impl AsyncEndpoint<RequestBody, ResponseWriter> for DiagnosticEndpoint {
    async fn handle(
        &self,
        req: Request<RequestBody>,
        response_extensions: &mut Extensions,
    ) -> Result<Response<AsyncResponseBody<ResponseWriter>>, Error> {
        response_extensions.insert(SafeParams::new());
        let safe_params = response_extensions.get_mut::<SafeParams>().unwrap();

        let diagnostic_type = &req
            .extensions()
            .get::<PathParams>()
            .expect("PathParams missing from request extensions")["diagnosticType"];
        safe_params.insert("diagnosticType", &diagnostic_type);

        let authorization = match req.headers().get(AUTHORIZATION) {
            Some(authorization) => authorization,
            None => {
                return Err(Error::service_safe(
                    "diagnostic check secret missing",
                    PermissionDenied::new(),
                ))
            }
        };

        let expected = self.state.auth.get();
        if !bool::from(authorization.as_bytes().ct_eq(expected.as_bytes())) {
            return Err(Error::service_safe(
                "invalid diagnostic check secret",
                PermissionDenied::new(),
            ));
        }

        let diagnostic = match self.state.diagnostics.get(diagnostic_type) {
            Some(diagnostic) => diagnostic,
            None => {
                return Err(Error::service_safe(
                    "unsupported diagnostic",
                    NotFound::new(),
                ))
            }
        };

        let body = task::spawn_blocking({
            let diagnostic = diagnostic.clone();
            move || diagnostic.result()
        })
        .await
        .unwrap()?;

        let mut response = Response::new(AsyncResponseBody::Fixed(body));
        response
            .headers_mut()
            .insert(CONTENT_TYPE, diagnostic.content_type());
        let safe_loggable = if diagnostic.safe_loggable() {
            TRUE_VALUE
        } else {
            FALSE_VALUE
        };
        response
            .headers_mut()
            .insert(SAFE_LOGGABLE.clone(), safe_loggable);

        Ok(response)
    }
}
