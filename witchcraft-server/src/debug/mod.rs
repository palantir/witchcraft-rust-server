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
//! Debug endpoints.
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use bytes::Bytes;
use conjure_error::Error;
use http::HeaderValue;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;

pub(crate) mod diagnostic_types;
pub(crate) mod endpoint;
#[cfg(feature = "jemalloc")]
pub(crate) mod heap_stats;
pub(crate) mod metric_names;
#[cfg(target_os = "linux")]
pub(crate) mod thread_dump;

static TYPE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"([a-z0-9]+\.)+v[0-9]+").unwrap());

/// An SLS diagnostic. See the [SLS debug spec](https://github.palantir.build/deployability/sls-spec/blob/develop/docs/debug.md)
pub trait Diagnostic {
    /// The type of the diagnostic. Must be lower cased, dot delimited, and end with a version.
    ///
    /// Example: "my.diagnostic.v1"
    fn type_(&self) -> &str;

    /// The value of the "Content-Type" header.
    ///
    /// Example: "application/json"
    fn content_type(&self) -> HeaderValue;

    /// Whether the value is safe to log
    fn safe_loggable(&self) -> bool;

    /// The bytes of the response to send
    fn result(&self) -> Result<Bytes, Error>;
}

/// A registry of diagnostics for the server.
pub struct DiagnosticRegistry {
    diagnostics: Mutex<HashMap<String, Arc<dyn Diagnostic + Sync + Send>>>,
}

impl Default for DiagnosticRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticRegistry {
    /// Create a new diagnostic registry
    pub fn new() -> Self {
        DiagnosticRegistry {
            diagnostics: Mutex::new(HashMap::new()),
        }
    }

    /// Register a new diagnostic.
    ///
    /// # Panics
    ///
    /// Panics if the diagnostic type is not valid. See [Diagnostic::type_] for valid diagnostic
    /// types.
    ///
    /// Panics if another diagnostic of the same type is already registered.
    pub fn register<T>(&self, diagnostic: T)
    where
        T: Diagnostic + 'static + Sync + Send,
    {
        self.register_inner(Arc::new(diagnostic));
    }

    fn register_inner(&self, diagnostic: Arc<dyn Diagnostic + Sync + Send>) {
        let type_ = diagnostic.type_();

        assert!(
            TYPE_PATTERN.is_match(type_),
            "{type_} must be `lower.case.dot.delimited.v1`",
        );

        match self.diagnostics.lock().entry(type_.to_string()) {
            Entry::Occupied(_) => {
                panic!("a diagnostic has already been registered for type {type_}")
            }
            Entry::Vacant(e) => {
                e.insert(diagnostic);
            }
        }
    }

    /// Get the diagnostic with the specified type.
    pub fn get(&self, type_: &str) -> Option<Arc<dyn Diagnostic + Sync + Send>> {
        self.diagnostics.lock().get(type_).cloned()
    }
}
