use crate::logging;
use crate::service::request_id::RequestId;
use crate::service::unverified_jwt::UnverifiedJwt;
use crate::service::{Layer, Service};
use http::Request;
use witchcraft_log::mdc;

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
    S: Service<Request<B>>,
{
    type Response = S::Response;

    type Future = S::Future;

    fn call(&self, req: Request<B>) -> Self::Future {
        if let Some(jwt) = req.extensions().get::<UnverifiedJwt>() {
            mdc::insert_safe(logging::UID_MDC_KEY, jwt.unverified_user_id());
            if let Some(session_id) = jwt.unverified_session_id() {
                mdc::insert_safe(logging::SID_MDC_KEY, session_id);
            }
            if let Some(token_id) = jwt.unverified_token_id() {
                mdc::insert_safe(logging::TOKEN_ID_MDC_KEY, token_id);
            }
        }

        let context = zipkin::current().expect("zipkin trace not initialized");
        mdc::insert_safe(logging::TRACE_ID_MDC_KEY, context.trace_id().to_string());
        if let Some(sampled) = context.sampled() {
            mdc::insert_safe("_sampled", sampled);
        }

        let request_id = req
            .extensions()
            .get::<RequestId>()
            .expect("RequestId missing from request extensions");
        mdc::insert_safe("_requestId", request_id.to_string());

        self.inner.call(req)
    }
}
