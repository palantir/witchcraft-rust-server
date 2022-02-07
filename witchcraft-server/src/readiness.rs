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
//! Readiness checks.
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use serde::Serialize;
use staged_builder::staged_builder;
use std::collections::btree_map;
use std::collections::hash_map;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

static TYPE_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new("^[A-Z_]+$").unwrap());

/// A readiness check.
pub trait ReadinessCheck {
    /// Returns the check's type.
    ///
    /// The type must be `SCREAMING_SNAKE_CASE`.
    fn type_(&self) -> &str;

    /// Performs the check, returning its result.
    fn result(&self) -> ReadinessCheckResult;
}

/// The result of a readiness check.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[staged_builder]
pub struct ReadinessCheckResult {
    successful: bool,
}

impl ReadinessCheckResult {
    /// The success of the check.
    #[inline]
    pub fn successful(&self) -> bool {
        self.successful
    }
}

/// A registry of readiness checks for the server.
pub struct ReadinessCheckRegistry {
    checks: Mutex<HashMap<String, Arc<dyn ReadinessCheck + Sync + Send>>>,
}

impl ReadinessCheckRegistry {
    pub(crate) fn new() -> Self {
        ReadinessCheckRegistry {
            checks: Mutex::new(HashMap::new()),
        }
    }

    /// Registers a new readiness check.
    ///
    /// # Panics
    ///
    /// Panics if the check's type is not `SCREAMING_SNAKE_CASE` or if a check with the same type is already registered.
    pub fn register<T>(&self, check: T)
    where
        T: ReadinessCheck + 'static + Sync + Send,
    {
        self.register_inner(Arc::new(check))
    }

    fn register_inner(&self, check: Arc<dyn ReadinessCheck + Sync + Send>) {
        let type_ = check.type_();

        assert!(
            TYPE_PATTERN.is_match(type_),
            "{type_} must `SCREAMING_SNAKE_CASE",
        );

        match self.checks.lock().entry(type_.to_string()) {
            hash_map::Entry::Occupied(_) => {
                panic!("a check has already been registered for type {type_}")
            }
            hash_map::Entry::Vacant(e) => {
                e.insert(check);
            }
        }
    }

    pub(crate) fn run_checks(&self) -> BTreeMap<String, ReadinessCheckMetadata> {
        // A bit of extra complexity to allow registration while we're running checks.
        let mut results = BTreeMap::new();

        let mut progress = true;
        while progress {
            progress = false;
            let checks = self.checks.lock().clone();

            for (type_, check) in checks {
                if let btree_map::Entry::Vacant(e) = results.entry(type_.clone()) {
                    let result = check.result();
                    e.insert(ReadinessCheckMetadata {
                        r#type: type_,
                        successful: result.successful,
                    });
                    progress = true;
                }
            }
        }

        results
    }
}

#[derive(Serialize)]
pub(crate) struct ReadinessCheckMetadata {
    r#type: String,
    pub(crate) successful: bool,
}
