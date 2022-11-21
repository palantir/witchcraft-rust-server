// Copyright 2021 Palantir Technologies, Inc.
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
use crate::logging::format::{LogFormat, ReportLog};
use crate::logging::logger::Payload;
use futures_sink::Sink;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use witchcraft_metrics::MetricRegistry;

#[pin_project]
pub struct MetricsAppender<S, T>
where
    T: LogFormat,
{
    #[pin]
    inner: S,
    reporter: T::Reporter,
}

impl<S, T> MetricsAppender<S, T>
where
    T: LogFormat,
{
    pub fn new(inner: S, metrics: &MetricRegistry) -> Self {
        MetricsAppender {
            inner,
            reporter: <T::Reporter as ReportLog<T>>::new(metrics),
        }
    }
}

impl<S, T> Sink<Payload<T>> for MetricsAppender<S, T>
where
    S: Sink<Payload<T>>,
    T: LogFormat,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Payload<T>) -> Result<(), Self::Error> {
        let this = self.project();
        this.reporter.report(&item.value);
        this.inner.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}
