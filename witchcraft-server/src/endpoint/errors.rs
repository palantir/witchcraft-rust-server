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
use conjure_serde::json;
use http::header::{CONTENT_TYPE, RETRY_AFTER};
use http::{HeaderValue, Response, StatusCode};
use witchcraft_log::error;

#[allow(clippy::declare_interior_mutable_const)]
const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");

pub fn to_response<F, B>(error: Error, body_creator: F) -> Response<B>
where
    F: FnOnce(Option<Bytes>) -> B,
{
    let mut response = match error.kind() {
        ErrorKind::Service(service) => {
            let body = conjure_error::encode(service);
            let body = json::to_vec(&body).unwrap();
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

    response.extensions_mut().insert(error);
    response
}
