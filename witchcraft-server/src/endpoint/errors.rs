use std::sync::Arc;

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
use bytes::Bytes;
use conjure_error::{Error, ErrorKind};
use conjure_http::server::UseLegacyErrorSerialization;
use conjure_serde::json;
use http::header::{CONTENT_TYPE, RETRY_AFTER};
use http::{Extensions, HeaderMap, HeaderName, HeaderValue, Response, StatusCode};
use witchcraft_log::error;

const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");
const ACCEPT_CONJURE_ERROR_PARAMETER_FORMAT: HeaderName =
    HeaderName::from_static("accept-conjure-error-parameter-format");
const JSON_FORMAT: HeaderValue = HeaderValue::from_static("JSON");

pub struct ErrorConverter {
    client_requested_json: bool,
}

impl ErrorConverter {
    pub fn new(request_headers: &HeaderMap) -> Self {
        ErrorConverter {
            client_requested_json: Self::client_requested_json(request_headers),
        }
    }

    fn client_requested_json(request_headers: &HeaderMap) -> bool {
        if let Some(format) = request_headers.get(ACCEPT_CONJURE_ERROR_PARAMETER_FORMAT) {
            if format
                .as_bytes()
                .eq_ignore_ascii_case(JSON_FORMAT.as_bytes())
            {
                return true;
            }
        }

        false
    }

    fn should_stringify_parameters(&self, response_extensions: &Extensions) -> bool {
        if self.client_requested_json {
            return false;
        }

        response_extensions
            .get::<UseLegacyErrorSerialization>()
            .is_some()
    }

    pub fn convert<F, B>(
        self,
        response_extensions: &Extensions,
        error: Error,
        body_creator: F,
    ) -> Response<B>
    where
        F: FnOnce(Option<Bytes>) -> B,
    {
        let mut response = match error.kind() {
            ErrorKind::Service(service) => {
                let body = if self.should_stringify_parameters(response_extensions) {
                    let service = conjure_error::stringify_parameters(service.clone());
                    json::to_vec(&service).unwrap()
                } else {
                    json::to_vec(service).unwrap()
                };
                let mut response = Response::new(body_creator(Some(Bytes::from(body))));
                *response.status_mut() =
                    StatusCode::from_u16(service.error_code().status_code()).unwrap();
                response
                    .headers_mut()
                    .insert(CONTENT_TYPE, APPLICATION_JSON);
                response
            }
            ErrorKind::Throttle(throttle) => {
                let mut response = Response::new(body_creator(None));
                *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                if let Some(duration) = throttle.duration() {
                    let header = HeaderValue::from(duration.as_secs());
                    response.headers_mut().insert(RETRY_AFTER, header);
                }
                response
            }
            ErrorKind::Unavailable(_) => {
                let mut response = Response::new(body_creator(None));
                *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
                response
            }
            _ => {
                error!("unknown error kind");
                let mut response = Response::new(body_creator(None));
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                response
            }
        };

        response.extensions_mut().insert(Arc::new(error));
        response
    }
}
