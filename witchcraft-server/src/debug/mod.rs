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
use crate::debug::diagnostic_types::DiagnosticTypesDiagnostic;
use bytes::Bytes;
use conjure_error::Error;
use http::HeaderValue;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

mod diagnostic_types;
pub mod endpoint;
#[cfg(feature = "jemalloc")]
pub mod heap_stats;
pub mod metric_names;
#[cfg(all(target_os = "linux", feature = "minidump"))]
pub mod thread_dump;

static TYPE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"([a-z0-9]+\.)+v[0-9]+").unwrap());

pub trait Diagnostic {
    fn type_(&self) -> &str;

    fn content_type(&self) -> HeaderValue;

    fn safe_loggable(&self) -> bool;

    fn result(&self) -> Result<Bytes, Error>;
}

pub struct DiagnosticRegistry {
    diagnostics: HashMap<String, Arc<dyn Diagnostic + Sync + Send>>,
}

impl DiagnosticRegistry {
    pub fn new() -> Self {
        DiagnosticRegistry {
            diagnostics: HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, diagnostic: T)
    where
        T: Diagnostic + 'static + Sync + Send,
    {
        self.register_inner(Arc::new(diagnostic));
    }

    fn register_inner(&mut self, diagnostic: Arc<dyn Diagnostic + Sync + Send>) {
        let type_ = diagnostic.type_();

        assert!(
            TYPE_PATTERN.is_match(type_),
            "{type_} must be `lower.case.dot.delimited.v1`",
        );

        match self.diagnostics.entry(type_.to_string()) {
            Entry::Occupied(_) => {
                panic!("a diagnostic has already been registered for type {type_}")
            }
            Entry::Vacant(e) => {
                e.insert(diagnostic);
            }
        }
    }

    pub fn finalize(&mut self) {
        let diagnostic_types =
            DiagnosticTypesDiagnostic::new(self.diagnostics.keys().cloned().collect());
        self.register(diagnostic_types);
    }

    fn get(&self, type_: &str) -> Option<&Arc<dyn Diagnostic + Sync + Send>> {
        self.diagnostics.get(type_)
    }
}
