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
use crate::endpoint::WitchcraftEndpoint;
use crate::service::{Layer, Service};
use conjure_http::server::PathSegment;
use conjure_http::PathParams;
use http::{Method, Request};
use itertools::Itertools;
use regex::{Regex, RegexSet};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;

// Our behavior here follows a subset of the JAX-RS spec:
// https://jakarta.ee/specifications/restful-ws/3.0/jakarta-restful-ws-spec-3.0.html#request_matching

const DEFAULT_REGEX: &str = "[^/]+?";

pub struct Endpoint {
    endpoint: Arc<dyn WitchcraftEndpoint + Sync + Send>,
    regex: Regex,
    literal_chars: usize,
    path_params: Vec<Cow<'static, str>>,
    custom_path_params: usize,
}

impl Endpoint {
    pub fn new(endpoint: Box<dyn WitchcraftEndpoint + Sync + Send>) -> Self {
        let mut regex = "^".to_string();
        for segment in endpoint.path() {
            regex.push('/');

            match segment {
                PathSegment::Literal(s) => regex.push_str(&regex::escape(s)),
                PathSegment::Parameter {
                    name,
                    regex: segment_regex,
                } => {
                    let segment_regex = segment_regex.as_deref().unwrap_or(DEFAULT_REGEX);
                    write!(regex, "(?P<{}>{})", name, segment_regex).unwrap();
                }
            }
        }
        regex.push('$');

        let mut literal_chars = 0;
        let mut path_params = vec![];
        let mut custom_path_params = 0;

        for segment in endpoint.path() {
            match segment {
                // +1 for the `/`
                PathSegment::Literal(s) => literal_chars += s.len() + 1,
                PathSegment::Parameter { name, regex } => {
                    path_params.push(name.clone());
                    if regex.as_deref().unwrap_or(DEFAULT_REGEX) != DEFAULT_REGEX {
                        custom_path_params += 1;
                    }
                }
            }
        }

        Endpoint {
            endpoint: Arc::from(endpoint),
            regex: Regex::new(&regex).unwrap(),
            literal_chars,
            path_params,
            custom_path_params,
        }
    }

    fn cmp_priority(&self, other: &Self) -> Ordering {
        self.literal_chars
            .cmp(&other.literal_chars)
            .then_with(|| self.path_params.len().cmp(&other.path_params.len()))
            .then_with(|| self.custom_path_params.cmp(&other.custom_path_params))
            .reverse()
    }
}

pub enum Route {
    Resolved(Arc<dyn WitchcraftEndpoint + Sync + Send>),
    MethodNotAllowed(Vec<Method>),
    StarOptions,
    Options(Vec<Method>),
    Unresolved,
}

/// A layer which performs the routing for a request.
///
/// It will add a [`Route`] to the request's extensions which determines the response behavior of the request. If the
/// request was routed to an endpoint, it will also add a [`PathParams`] to the request's extensions with the parsed
/// path parameters of the request's URI.
pub struct RoutingLayer {
    endpoints: HashMap<Method, Routes>,
}

impl RoutingLayer {
    pub fn new(endpoints: Vec<Box<dyn WitchcraftEndpoint + Sync + Send>>) -> Self {
        let endpoints_by_method = endpoints
            .into_iter()
            .map(Endpoint::new)
            .into_group_map_by(|e| e.endpoint.method());

        RoutingLayer {
            endpoints: endpoints_by_method
                .into_iter()
                .map(|(method, endpoints)| (method, Routes::new(endpoints)))
                .collect(),
        }
    }
}

impl<S> Layer<S> for RoutingLayer {
    type Service = RoutingService<S>;

    fn layer(self, inner: S) -> Self::Service {
        RoutingService {
            inner,
            endpoints: self.endpoints,
        }
    }
}

struct Routes {
    set: RegexSet,
    endpoints: Vec<Endpoint>,
}

impl Routes {
    fn new(mut endpoints: Vec<Endpoint>) -> Self {
        endpoints.sort_by(Endpoint::cmp_priority);

        Routes {
            set: RegexSet::new(endpoints.iter().map(|e| e.regex.as_str())).unwrap(),
            endpoints,
        }
    }

    fn is_match(&self, path: &str) -> bool {
        self.set.is_match(path)
    }

    fn route(&self, path: &str) -> Option<&Endpoint> {
        self.set
            .matches(path)
            .iter()
            .next()
            .map(|idx| &self.endpoints[idx])
    }
}

pub struct RoutingService<S> {
    inner: S,
    endpoints: HashMap<Method, Routes>,
}

impl<S> RoutingService<S> {
    fn supported_methods(&self, path: &str) -> Vec<Method> {
        self.endpoints
            .iter()
            .filter(|(_, routes)| routes.is_match(path))
            .map(|(method, _)| method)
            .sorted_by_key(|m| m.as_str())
            .cloned()
            .collect()
    }
}

impl<S, B> Service<Request<B>> for RoutingService<S>
where
    S: Service<Request<B>> + Sync,
    B: Send,
{
    type Response = S::Response;

    async fn call(&self, mut req: Request<B>) -> Self::Response {
        let (route, endpoint) = if req.method() == Method::OPTIONS && req.uri() == "*" {
            (Route::StarOptions, None)
        } else {
            match self
                .endpoints
                .get(req.method())
                .and_then(|r| r.route(req.uri().path()))
            {
                Some(endpoint) => (Route::Resolved(endpoint.endpoint.clone()), Some(endpoint)),
                None if req.method() == Method::OPTIONS => (
                    Route::Options(self.supported_methods(req.uri().path())),
                    None,
                ),
                None => {
                    let methods = self.supported_methods(req.uri().path());
                    if methods.is_empty() {
                        (Route::Unresolved, None)
                    } else {
                        (Route::MethodNotAllowed(methods), None)
                    }
                }
            }
        };

        if let Some(endpoint) = endpoint {
            if !endpoint.path_params.is_empty() {
                let captures = endpoint.regex.captures(req.uri().path()).unwrap();

                let mut path_params = PathParams::new();
                for name in &endpoint.path_params {
                    path_params.insert(&**name, captures.name(name).unwrap().as_str());
                }
                req.extensions_mut().insert(path_params);
            }
        }

        req.extensions_mut().insert(route);
        self.inner.call(req).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::health::endpoint_500s::EndpointHealth;
    use crate::server::RawBody;
    use crate::service::endpoint_metrics::EndpointMetrics;
    use crate::service::handler::BodyWriteAborted;
    use crate::service::test_util::service_fn;
    use async_trait::async_trait;
    use bytes::Bytes;
    use conjure_http::server::EndpointMetadata;
    use http::Response;
    use http_body::combinators::BoxBody;

    struct TestEndpoint {
        method: Method,
        path: Vec<PathSegment>,
        name: &'static str,
    }

    impl EndpointMetadata for TestEndpoint {
        fn method(&self) -> Method {
            self.method.clone()
        }

        fn path(&self) -> &[PathSegment] {
            &self.path
        }

        fn template(&self) -> &str {
            ""
        }

        fn service_name(&self) -> &str {
            ""
        }

        fn name(&self) -> &str {
            self.name
        }

        fn deprecated(&self) -> Option<&str> {
            None
        }
    }

    #[async_trait]
    impl WitchcraftEndpoint for TestEndpoint {
        fn metrics(&self) -> Option<&EndpointMetrics> {
            None
        }

        fn health(&self) -> Option<&Arc<EndpointHealth>> {
            None
        }

        async fn handle(&self, _: Request<RawBody>) -> Response<BoxBody<Bytes, BodyWriteAborted>> {
            unimplemented!()
        }
    }

    fn endpoint(
        method: Method,
        path: Vec<PathSegment>,
        name: &'static str,
    ) -> Box<dyn WitchcraftEndpoint + Sync + Send> {
        Box::new(TestEndpoint { method, path, name })
    }

    #[tokio::test]
    async fn empty() {
        let service = RoutingLayer::new(vec![]).layer(service_fn(|req| async { req }));

        let req = service
            .call(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("*")
                    .body(())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::StarOptions) => {}
            _ => panic!("bad route"),
        }

        let req = service
            .call(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/foo/bar?a=b")
                    .body(())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Options(methods)) => assert!(methods.is_empty()),
            _ => panic!("bad route"),
        }

        let req = service
            .call(
                Request::builder()
                    .method(Method::GET)
                    .uri("/foo/bar?a=b")
                    .body(())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Unresolved) => {}
            _ => panic!("bad route"),
        }
    }

    #[tokio::test]
    async fn nonempty() {
        let service = RoutingLayer::new(vec![
            endpoint(
                Method::GET,
                vec![
                    PathSegment::Literal(Cow::Borrowed("foo")),
                    PathSegment::Literal(Cow::Borrowed("bar")),
                ],
                "a",
            ),
            endpoint(
                Method::POST,
                vec![
                    PathSegment::Literal(Cow::Borrowed("foo")),
                    PathSegment::Parameter {
                        name: Cow::Borrowed("arg"),
                        regex: None,
                    },
                ],
                "b",
            ),
        ])
        .layer(service_fn(|req| async { req }));

        let req = service
            .call(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/foo/bar?a=b")
                    .body(())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Options(methods)) => assert_eq!(*methods, [Method::GET, Method::POST]),
            _ => panic!("bad route"),
        }

        let req = service
            .call(
                Request::builder()
                    .method(Method::GET)
                    .uri("/foo/bar?a=b")
                    .body(())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Resolved(endpoint)) => assert_eq!(endpoint.name(), "a"),
            _ => panic!("bad route"),
        }

        let req = service
            .call(
                Request::builder()
                    .method(Method::POST)
                    .uri("/foo/bar?a=b")
                    .body(())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Resolved(endpoint)) => assert_eq!(endpoint.name(), "b"),
            _ => panic!("bad route"),
        }
        assert_eq!(&req.extensions().get::<PathParams>().unwrap()["arg"], "bar");

        let req = service
            .call(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/foo/bar?a=b")
                    .body(())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::MethodNotAllowed(methods)) => {
                assert_eq!(*methods, [Method::GET, Method::POST])
            }
            _ => panic!("bad route"),
        }

        let req = service
            .call(
                Request::builder()
                    .method(Method::POST)
                    .uri("/foo/bar/baz?a=b")
                    .body(())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Unresolved) => {}
            _ => panic!("bad route"),
        }
    }

    #[tokio::test]
    async fn custom_regex() {
        let service = RoutingLayer::new(vec![endpoint(
            Method::GET,
            vec![
                PathSegment::Literal(Cow::Borrowed("foo")),
                PathSegment::Parameter {
                    name: Cow::Borrowed("arg"),
                    regex: Some(Cow::Borrowed(".*")),
                },
            ],
            "a",
        )])
        .layer(service_fn(|req: Request<hyper::Body>| async { req }));

        let req = service
            .call(
                Request::builder()
                    .method(Method::GET)
                    .uri("/foo/bar/baz?a=b")
                    .body(hyper::Body::empty())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Resolved(endpoint)) => assert_eq!(endpoint.name(), "a"),
            _ => panic!("bad route"),
        }
        assert_eq!(
            &req.extensions().get::<PathParams>().unwrap()["arg"],
            "bar/baz"
        );
    }

    #[tokio::test]
    async fn ambiguity() {
        let service = RoutingLayer::new(vec![
            endpoint(
                Method::GET,
                vec![
                    PathSegment::Literal(Cow::Borrowed("foo")),
                    PathSegment::Literal(Cow::Borrowed("bar")),
                ],
                "a",
            ),
            endpoint(
                Method::GET,
                vec![
                    PathSegment::Literal(Cow::Borrowed("foo")),
                    PathSegment::Parameter {
                        name: Cow::Borrowed("arg"),
                        regex: None,
                    },
                ],
                "b",
            ),
        ])
        .layer(service_fn(|req: Request<hyper::Body>| async { req }));

        let req = service
            .call(
                Request::builder()
                    .method(Method::GET)
                    .uri("/foo/bar?a=b")
                    .body(hyper::Body::empty())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Resolved(endpoint)) => assert_eq!(endpoint.name(), "a"),
            _ => panic!("bad route"),
        }
    }

    #[tokio::test]
    async fn absolute_form() {
        let service = RoutingLayer::new(vec![endpoint(
            Method::GET,
            vec![PathSegment::Literal(Cow::Borrowed("foo"))],
            "a",
        )])
        .layer(service_fn(|req: Request<hyper::Body>| async { req }));

        let req = service
            .call(
                Request::builder()
                    .method(Method::GET)
                    .uri("https://foobar.com/foo?a=b")
                    .body(hyper::Body::empty())
                    .unwrap(),
            )
            .await;
        match req.extensions().get() {
            Some(Route::Resolved(endpoint)) => assert_eq!(endpoint.name(), "a"),
            _ => panic!("bad route"),
        }
    }
}
