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
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};
use std::sync::Arc;
use witchcraft_metrics::{MetricId, MetricRegistry, Metrics, Tags};

/// A diagnostic which returns the JSON-formatted names of every metric in the server's registry.
pub struct MetricNamesDiagnostic {
    metrics: Arc<MetricRegistry>,
}

impl MetricNamesDiagnostic {
    pub fn new(metrics: &Arc<MetricRegistry>) -> Self {
        MetricNamesDiagnostic {
            metrics: metrics.clone(),
        }
    }
}

impl Diagnostic for MetricNamesDiagnostic {
    fn type_(&self) -> &str {
        "metric.names.v1"
    }

    fn content_type(&self) -> HeaderValue {
        HeaderValue::from_static("application/json")
    }

    fn safe_loggable(&self) -> bool {
        true
    }

    fn result(&self) -> Result<Bytes, Error> {
        let metrics = self.metrics.metrics();
        let body = json::to_vec(&MetricNames(metrics)).unwrap();
        Ok(Bytes::from(body))
    }
}

struct MetricNames(Metrics);

impl Serialize for MetricNames {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_seq(None)?;
        for (id, _) in &self.0 {
            s.serialize_element(&MetricName(id))?;
        }
        s.end()
    }
}

struct MetricName<'a>(&'a MetricId);

impl Serialize for MetricName<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(None)?;
        s.serialize_entry("name", &self.0.name())?;
        if self.0.tags().iter().next().is_some() {
            s.serialize_entry("tags", &MetricTags(self.0.tags()))?;
        }
        s.end()
    }
}

struct MetricTags<'a>(&'a Tags);

impl Serialize for MetricTags<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(None)?;
        for (key, value) in self.0 {
            s.serialize_entry(key, value)?;
        }
        s.end()
    }
}
