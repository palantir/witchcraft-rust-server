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

use crate::logging;
use crate::service::request_id::RequestId;
use crate::service::unverified_jwt::UnverifiedJwt;
use crate::service::{Layer, Service};
use http::Request;
use witchcraft_log::mdc;

/// A layer which injects Witchcraft-managed request state into the MDC.
///
/// It must be installed after MDC tracking, JWT extraction, request ID generation, and trace propagation.
pub struct WitchcraftMdcLayer;

impl<S> Layer<S> for WitchcraftMdcLayer {
    type Service = WitchcraftMdcService<S>;

    fn layer(self, inner: S) -> Self::Service {
        WitchcraftMdcService { inner }
    }
}

pub struct WitchcraftMdcService<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for WitchcraftMdcService<S>
where
    S: Service<Request<B>> + Sync,
    B: Send,
{
    type Response = S::Response;

    async fn call(&self, req: Request<B>) -> Self::Response {
        if let Some(jwt) = req.extensions().get::<UnverifiedJwt>() {
            mdc::insert_safe(logging::mdc::UID_KEY, jwt.unverified_user_id());
            if let Some(session_id) = jwt.unverified_session_id() {
                mdc::insert_safe(logging::mdc::SID_KEY, session_id);
            }
            if let Some(token_id) = jwt.unverified_token_id() {
                mdc::insert_safe(logging::mdc::TOKEN_ID_KEY, token_id);
            }
            if let Some(org_id) = jwt.unverified_organization_id() {
                mdc::insert_safe(logging::mdc::ORG_ID_KEY, org_id);
            }
        }

        let context = zipkin::current().expect("zipkin trace not initialized");
        mdc::insert_safe(logging::mdc::TRACE_ID_KEY, context.trace_id().to_string());
        if let Some(sampled) = context.sampled() {
            mdc::insert_safe(logging::SAMPLED_KEY, sampled);
        }

        let request_id = req
            .extensions()
            .get::<RequestId>()
            .expect("RequestId missing from request extensions");
        mdc::insert_safe(logging::REQUEST_ID_KEY, request_id.to_string());

        self.inner.call(req).await
    }
}
