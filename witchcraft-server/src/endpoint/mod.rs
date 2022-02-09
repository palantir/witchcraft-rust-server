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
use crate::service::handler::BodyWriteAborted;
use async_trait::async_trait;
use bytes::Bytes;
use conjure_http::server::EndpointMetadata;
use http::{Request, Response};
use http_body::combinators::BoxBody;

#[async_trait]
pub trait WitchcraftEndpoint: EndpointMetadata {
    async fn handle(&self, req: Request<hyper::Body>)
        -> Response<BoxBody<Bytes, BodyWriteAborted>>;
}
