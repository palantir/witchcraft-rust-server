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
use crate::health::{HealthCheck, HealthCheckResult, HealthState};
use conjure_runtime::HostMetricsRegistry;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// A health check which reports failures in calls from this service to others.
pub struct ServiceDependencyHealthCheck {
    host_metrics: Arc<HostMetricsRegistry>,
}

impl ServiceDependencyHealthCheck {
    pub fn new(host_metrics: &Arc<HostMetricsRegistry>) -> Self {
        ServiceDependencyHealthCheck {
            host_metrics: host_metrics.clone(),
        }
    }
}

impl HealthCheck for ServiceDependencyHealthCheck {
    fn type_(&self) -> &str {
        "SERVICE_DEPENDENCY"
    }

    fn result(&self) -> HealthCheckResult {
        let cutoff = Instant::now() - IDLE_TIMEOUT;
        let hosts = self.host_metrics.hosts();

        let bad_hosts_by_service = hosts
            .iter()
            .filter(|m| m.last_update() > cutoff)
            .filter(|m| {
                m.response_5xx().five_minute_rate() > m.response_2xx().five_minute_rate()
                    || m.io_error().five_minute_rate() > m.response_2xx().five_minute_rate()
            })
            .map(|m| (m.service_name(), format!("{}:{}", m.hostname(), m.port())))
            .into_grouping_map()
            .collect::<BTreeSet<_>>();

        if bad_hosts_by_service.is_empty() {
            return HealthCheckResult::builder()
                .state(HealthState::Healthy)
                .message("All remote services are healthy".to_string())
                .build();
        }

        let num_hosts_by_service = hosts.iter().map(|m| m.service_name()).counts();

        let num_unhealthy_services = bad_hosts_by_service
            .iter()
            .filter(|(service, hosts)| hosts.len() == num_hosts_by_service[**service])
            .count();

        let builder = if num_unhealthy_services > 0 {
            HealthCheckResult::builder()
                .state(HealthState::Warning)
                .message("All nodes of a remote service have a high failure rate".to_string())
        } else {
            HealthCheckResult::builder()
                .state(HealthState::Healthy)
                .message("Some nodes of a remote service have a high failure rate".to_string())
        };

        builder.params(bad_hosts_by_service).build()
    }
}
