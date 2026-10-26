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
use conjure_error::{DueTo, Error, ErrorKind};
use conjure_http::server::UseLegacyErrorSerialization;
use conjure_serde::json;
use http::header::{CONTENT_TYPE, RETRY_AFTER};
use http::{Extensions, HeaderMap, HeaderName, HeaderValue, Response, StatusCode};
use witchcraft_log::error;

#[allow(clippy::declare_interior_mutable_const)]
const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");
const QOS_DUE_TO: HeaderName = HeaderName::from_static("qos-due-to");

pub fn to_response<F, B>(
    response_extensions: &Extensions,
    error: Error,
    body_creator: F,
) -> Response<B>
where
    F: FnOnce(Option<Bytes>) -> B,
{
    let mut response = match error.kind() {
        ErrorKind::Service(service) => {
            let body = if response_extensions
                .get::<UseLegacyErrorSerialization>()
                .is_some()
            {
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
            insert_due_to(response.headers_mut(), throttle.due_to());
            response
        }
        ErrorKind::Unavailable(unavailable) => {
            let mut response = Response::new(body_creator(None));
            *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
            insert_due_to(response.headers_mut(), unavailable.due_to());
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

fn insert_due_to(headers: &mut HeaderMap, due_to: Option<DueTo>) {
    if let Some(due_to) = due_to {
        headers.insert(QOS_DUE_TO, HeaderValue::from_static(due_to.as_str()));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    fn response(error: Error) -> Response<()> {
        to_response(&Extensions::new(), error, |_| ())
    }

    #[test]
    fn throttle_due_to() {
        let response =
            response(Error::throttle_for("", Duration::from_secs(5)).with_due_to(DueTo::Custom));
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[RETRY_AFTER], "5");
        assert_eq!(response.headers()["Qos-Due-To"], "custom");
    }

    #[test]
    fn unavailable_due_to() {
        let response = response(Error::unavailable("").with_due_to(DueTo::Custom));
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.headers()["Qos-Due-To"], "custom");
    }

    #[test]
    fn no_due_to() {
        assert!(!response(Error::throttle(""))
            .headers()
            .contains_key(QOS_DUE_TO));
        assert!(!response(Error::unavailable(""))
            .headers()
            .contains_key(QOS_DUE_TO));
    }
}
