// Copyright 2026 Palantir Technologies, Inc.
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
use crate::service::{Layer, Service};
use http::{HeaderName, HeaderValue, Response};

// The standard header consumed by clients supporting server-recommended node selection.
const NODE_SELECTION_STRATEGY: HeaderName = HeaderName::from_static("node-selection-strategy");

/// A layer which adds the configured node selection recommendation to responses lacking one.
pub struct NodeSelectionStrategyLayer {
    value: Option<HeaderValue>,
}

impl NodeSelectionStrategyLayer {
    pub fn new(value: Option<HeaderValue>) -> Self {
        NodeSelectionStrategyLayer { value }
    }
}

impl<S> Layer<S> for NodeSelectionStrategyLayer {
    type Service = NodeSelectionStrategyService<S>;

    fn layer(self, inner: S) -> Self::Service {
        NodeSelectionStrategyService {
            inner,
            value: self.value,
        }
    }
}

pub struct NodeSelectionStrategyService<S> {
    inner: S,
    value: Option<HeaderValue>,
}

impl<S, R, B> Service<R> for NodeSelectionStrategyService<S>
where
    S: Service<R, Response = Response<B>> + Sync,
    R: Send,
{
    type Response = S::Response;

    async fn call(&self, req: R) -> Self::Response {
        let mut response = self.inner.call(req).await;
        if let Some(value) = &self.value {
            response
                .headers_mut()
                .entry(NODE_SELECTION_STRATEGY)
                .or_insert_with(|| value.clone());
        }
        response
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::test_util::service_fn;

    #[tokio::test]
    async fn response_header_precedence() {
        for (configured, existing, expected) in [
            (None, &[][..], &[][..]),
            (None, &["PIN_UNTIL_ERROR"][..], &["PIN_UNTIL_ERROR"][..]),
            (Some("BALANCED"), &[][..], &["BALANCED"][..]),
            (
                Some("BALANCED"),
                &["PIN_UNTIL_ERROR"][..],
                &["PIN_UNTIL_ERROR"][..],
            ),
            (Some("BALANCED"), &[""][..], &[""][..]),
            (
                Some("BALANCED"),
                &["PIN_UNTIL_ERROR", "PIN_UNTIL_ERROR_WITHOUT_RESHUFFLE"][..],
                &["PIN_UNTIL_ERROR", "PIN_UNTIL_ERROR_WITHOUT_RESHUFFLE"][..],
            ),
        ] {
            let service = NodeSelectionStrategyLayer::new(configured.map(HeaderValue::from_static))
                .layer(service_fn(|response| async move { response }));
            let mut response = Response::new(());
            for value in existing {
                response
                    .headers_mut()
                    .append(NODE_SELECTION_STRATEGY, HeaderValue::from_static(value));
            }

            let response = service.call(response).await;

            let actual = response
                .headers()
                .get_all(NODE_SELECTION_STRATEGY)
                .iter()
                .map(|value| value.to_str().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                actual, expected,
                "configured: {configured:?}, existing: {existing:?}"
            );
        }
    }
}
