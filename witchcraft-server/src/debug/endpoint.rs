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
use bytes::Bytes;
use conjure_error::{Error, NotFound, PermissionDenied};
use conjure_http::server::{AsyncResponseBody, AsyncSerializeResponse, ConjureRuntime};
use conjure_http::{conjure_endpoints, endpoint};
use conjure_object::BearerToken;
use http::header::{HeaderName, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue, Response};
use refreshable::Refreshable;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tokio::task;
use witchcraft_server_config::runtime::RuntimeConfig;

#[allow(clippy::declare_interior_mutable_const)]
const SAFE_LOGGABLE: HeaderName = HeaderName::from_static("safe-loggable");
#[allow(clippy::declare_interior_mutable_const)]
const TRUE_VALUE: HeaderValue = HeaderValue::from_static("true");
#[allow(clippy::declare_interior_mutable_const)]
const FALSE_VALUE: HeaderValue = HeaderValue::from_static("false");

#[conjure_endpoints]
pub trait DebugService {
    #[endpoint(path = "/debug/diagnostic/{diagnostic_type}", method = GET, produces = DiagnosticResponseSerializer)]
    async fn diagnostic(
        &self,
        #[auth] token: BearerToken,
        #[path(safe)] diagnostic_type: String,
    ) -> Result<DiagnosticResponse, Error>;
}

pub struct DiagnosticResponse {
    conent_type: HeaderValue,
    safe_loggable: bool,
    body: Bytes,
}

enum DiagnosticResponseSerializer {}

impl<W> AsyncSerializeResponse<DiagnosticResponse, W> for DiagnosticResponseSerializer {
    fn serialize(
        _: &ConjureRuntime,
        _: &HeaderMap,
        value: DiagnosticResponse,
    ) -> Result<Response<AsyncResponseBody<W>>, Error> {
        let mut response = Response::new(AsyncResponseBody::Fixed(value.body));
        response
            .headers_mut()
            .insert(CONTENT_TYPE, value.conent_type);
        let safe_loggable = if value.safe_loggable {
            TRUE_VALUE
        } else {
            FALSE_VALUE
        };
        response.headers_mut().insert(SAFE_LOGGABLE, safe_loggable);

        Ok(response)
    }
}

pub struct DebugResource {
    debug_secret: Refreshable<String, Error>,
    diagnostics: Arc<DiagnosticRegistry>,
}

impl DebugResource {
    pub fn new<R>(runtime: &Refreshable<R, Error>, diagnostics: &Arc<DiagnosticRegistry>) -> Self
    where
        R: AsRef<RuntimeConfig> + PartialEq + 'static + Sync + Send,
    {
        DebugResource {
            debug_secret: runtime
                .map(|c| c.as_ref().diagnostics().debug_shared_secret().to_string()),
            diagnostics: diagnostics.clone(),
        }
    }
}

impl DebugService for DebugResource {
    async fn diagnostic(
        &self,
        token: BearerToken,
        diagnostic_type: String,
    ) -> Result<DiagnosticResponse, Error> {
        let expected = self.debug_secret.get();
        if !bool::from(token.as_str().as_bytes().ct_eq(expected.as_bytes())) {
            return Err(Error::service_safe(
                "invalid diagnostic check secret",
                PermissionDenied::new(),
            ));
        }

        let diagnostic = match self.diagnostics.get(&diagnostic_type) {
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

        Ok(DiagnosticResponse {
            conent_type: diagnostic.content_type(),
            safe_loggable: diagnostic.safe_loggable(),
            body,
        })
    }
}
