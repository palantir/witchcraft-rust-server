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
use crate::service::Service;
use std::cell::RefCell;
use std::future::Future;
use std::mem;
use zipkin::{Endpoint, Report, Sample, Span, TraceId};

pub fn service_fn<F>(f: F) -> ServiceFn<F> {
    ServiceFn(f)
}

pub struct ServiceFn<F>(F);

impl<T, F, I, O> Service<I> for ServiceFn<T>
where
    T: Fn(I) -> F + Sync,
    F: Future<Output = O> + Send,
    I: Send,
{
    type Response = O;

    async fn call(&self, req: I) -> Self::Response {
        self.0(req).await
    }
}

thread_local! {
    static SPANS: RefCell<Vec<Span>> = RefCell::new(vec![]);
}

pub fn setup_tracer() {
    SPANS.with(|c| c.borrow_mut().clear());
    let _ = zipkin::set_tracer(TestSampler, TestReporter, Endpoint::builder().build());
}

pub fn spans() -> Vec<Span> {
    SPANS.with(|c| mem::take(&mut *c.borrow_mut()))
}

struct TestSampler;

impl Sample for TestSampler {
    fn sample(&self, _: TraceId) -> bool {
        true
    }
}

struct TestReporter;

impl Report for TestReporter {
    fn report(&self, span: Span) {
        SPANS.with(|c| c.borrow_mut().push(span));
    }
}
