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
use crate::debug::Diagnostic;
use bytes::Bytes;
use conjure_error::Error;
use conjure_serde::json;
use http::HeaderValue;

const DIAGNOSTIC_TYPES_V1: &str = "diagnostic.types.v1";

/// A diagnostic which returns a list of all registered diagnostics.
pub struct DiagnosticTypesDiagnostic {
    types: Bytes,
}

impl DiagnosticTypesDiagnostic {
    pub fn new(mut types: Vec<String>) -> Self {
        types.push(DIAGNOSTIC_TYPES_V1.to_string());
        types.sort_unstable();
        let types = Bytes::from(json::to_vec(&types).unwrap());

        DiagnosticTypesDiagnostic { types }
    }
}

impl Diagnostic for DiagnosticTypesDiagnostic {
    fn type_(&self) -> &str {
        DIAGNOSTIC_TYPES_V1
    }

    fn content_type(&self) -> HeaderValue {
        HeaderValue::from_static("application/json")
    }

    fn safe_loggable(&self) -> bool {
        true
    }

    fn result(&self) -> Result<Bytes, Error> {
        Ok(self.types.clone())
    }
}
