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
use http::HeaderValue;
use tikv_jemalloc_ctl::stats_print;

/// A diagnostic which returns heap statistics.
///
/// Requires jemalloc.
pub struct HeapStatsDiagnostic;

impl Diagnostic for HeapStatsDiagnostic {
    fn type_(&self) -> &str {
        "rust.heap.stats.v1"
    }

    fn content_type(&self) -> HeaderValue {
        HeaderValue::from_static("text/plain")
    }

    fn safe_loggable(&self) -> bool {
        true
    }

    fn result(&self) -> Result<Bytes, Error> {
        let mut buf = vec![];
        stats_print::stats_print(&mut buf, stats_print::Options::default())
            .map_err(Error::internal_safe)?;
        Ok(Bytes::from(buf))
    }
}
