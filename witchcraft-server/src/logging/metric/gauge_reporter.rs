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
use crate::logging::api::MetricLogV1;
use crate::logging::metric;
use futures_util::stream::FuturesUnordered;
use futures_util::{ready, Stream};
use pin_project::pin_project;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::task::{self, JoinError, JoinHandle};
use tokio::time::{self, Sleep};
use tokio_util::sync::PollSemaphore;
use witchcraft_metrics::{Gauge, MetricId};

const TARGET_CONCURRENT_TASKS: usize = 2;
const TOO_LONG_THRESHOLD: Duration = Duration::from_millis(500);

/// A structure to manage asynchronous processing of gauge metrics.
///
/// We expect most gauges to be very fast to compute but also need to gracefully handle arbitrarily slow gauges without
/// negatively impacting the emission of other metrics. Gauges run as blocking tasks on the Tokio runtime, which will
/// spawn more threads as needed. To avoid spawning tons of threads just to run a few fast gauges, we take a "partially
/// throttled" approach. We use a semaphore to only allow 2 gauges to evaluate at a time, but if we detect that the
/// gauge is slow (i.e. it's taking at least .5 seconds to run), we release its permit early. This should allow us to
/// end up with the slow gauges running in parallel while the fast gauges are throttled by the concurrency limit.
pub struct GaugeReporter {
    ids: HashSet<MetricId>,
    results: FuturesUnordered<GaugeFuture>,
    semaphore: Arc<Semaphore>,
}

impl GaugeReporter {
    pub fn new() -> Self {
        GaugeReporter {
            ids: HashSet::new(),
            results: FuturesUnordered::new(),
            semaphore: Arc::new(Semaphore::new(TARGET_CONCURRENT_TASKS)),
        }
    }

    pub fn insert(&mut self, id: &MetricId, gauge: &Arc<dyn Gauge>) -> bool {
        if !self.ids.insert(id.clone()) {
            return false;
        }

        self.results.push(GaugeFuture::Acquiring {
            id: Some(id.clone()),
            gauge: Some(gauge.clone()),
            semaphore: PollSemaphore::new(self.semaphore.clone()),
        });

        true
    }

    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }
}

impl Stream for GaugeReporter {
    type Item = Result<MetricLogV1, JoinError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item = ready!(Pin::new(&mut self.results).poll_next(cx));
        if let Some((id, _)) = &item {
            self.ids.remove(id);
        }
        Poll::Ready(item.map(|(_, result)| result))
    }
}

#[pin_project(project = GaugeFutureProj)]
#[allow(clippy::large_enum_variant)]
enum GaugeFuture {
    Acquiring {
        id: Option<MetricId>,
        gauge: Option<Arc<dyn Gauge>>,
        semaphore: PollSemaphore,
    },
    Running {
        id: Option<MetricId>,
        handle: JoinHandle<MetricLogV1>,
        permit: Option<OwnedSemaphorePermit>,
        #[pin]
        timeout: Sleep,
    },
}

impl Future for GaugeFuture {
    type Output = (MetricId, Result<MetricLogV1, JoinError>);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project() {
                GaugeFutureProj::Acquiring {
                    id,
                    gauge,
                    semaphore,
                } => {
                    let permit = ready!(semaphore.poll_acquire(cx));
                    let id = id.take().unwrap();
                    let gauge = gauge.take().unwrap();
                    let handle = task::spawn_blocking({
                        let id = id.clone();
                        move || {
                            let _span = zipkin::new_trace()
                                .with_name(&format!("record-gauge: {}", metric::metric_name(&id)));

                            let value = gauge.value();
                            let builder = metric::builder(&id)
                                .metric_type("gauge")
                                .insert_values("value", value);
                            metric::finish_log(&id, builder)
                        }
                    });

                    self.set(GaugeFuture::Running {
                        id: Some(id),
                        handle,
                        permit,
                        timeout: time::sleep(TOO_LONG_THRESHOLD),
                    });
                }
                GaugeFutureProj::Running {
                    id,
                    handle,
                    permit,
                    timeout,
                } => {
                    if permit.is_some() && timeout.poll(cx).is_ready() {
                        permit.take();
                    }

                    let result = ready!(Pin::new(handle).poll(cx));
                    return Poll::Ready((id.take().unwrap(), result));
                }
            }
        }
    }
}
