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
use bytes::Bytes;
use conjure_object::Any;
use http::{HeaderMap, HeaderValue};
use hyper::body::HttpBody;
use hyper::{body, Body, Request, StatusCode};
use server::Server;
use std::pin::Pin;
use std::str;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::time;

mod server;

#[tokio::test]
async fn safe_params() {
    Server::with(|server| async move {
        let request = Request::builder()
            .uri("/witchcraft-ete/api/test/safeParams/expected%20safe%20path/expected%20unsafe%20path?safeQueryId=\
                  expected%20safe%20query&unsafeQueryId=expected%20unsafe%20query")
            .header("Safe-Header", "expected safe header")
            .header("Unsafe-Header", "expected unsafe header")
            .body(Body::empty()).unwrap();
        let response = server.client().await.unwrap().send_request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let logs = server.shutdown().await;
        let request = logs.only_request();

        assert_eq!(
            request.path(),
            "/witchcraft-ete/api/test/safeParams/{safePath}/{unsafePath}",
        );
        assert_eq!(
            request.params()["safePath"],
            Any::new("expected safe path").unwrap(),
        );
        assert_eq!(
            request.params()["safeQuery"],
            Any::new("expected safe query").unwrap(),
        );
        assert_eq!(
            request.params()["safeHeader"],
            Any::new("expected safe header").unwrap(),
        );
        assert_eq!(request.params().get("unsafePath"), None);
        assert_eq!(request.params().get("unsafeQuery"), None);
        assert_eq!(request.params().get("unsafeHeader"), None);
    }).await;
}

#[tokio::test]
async fn keep_alive_slow_headers() {
    // the server is configured with a 2 second idle timeout
    Server::with(|server| async move {
        let mut client = server.client().await.unwrap();

        // 1.5 second delay before headers shouldn't count towards idle time.
        let request = Request::builder()
            .uri("/witchcraft-ete/api/test/slowHeaders?delayMillis=1500")
            .body(Body::empty())
            .unwrap();
        let response = client.send_request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // should only be at 1 second idle, not 2.5 seconds
        time::sleep(Duration::from_secs(1)).await;

        let request = Request::builder()
            .uri("/witchcraft-ete/api/test/slowHeaders?delayMillis=0")
            .body(Body::empty())
            .unwrap();
        let response = client.send_request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // connection should close after 2.25 seconds of idle time
        time::sleep(Duration::from_millis(2250)).await;

        let request = Request::builder()
            .uri("/witchcraft-ete/api/test/slowHeaders?delayMillis=0")
            .body(Body::empty())
            .unwrap();
        client.send_request(request).await.err().unwrap();

        drop(client);
        server.shutdown().await;
    })
    .await;
}

#[tokio::test]
async fn keep_alive_slow_body() {
    // the server is configured with a 2 second idle timeout
    Server::with(|server| async move {
        let mut client = server.client().await.unwrap();

        // 1.5 second delay writing body shouldn't count towards idle time
        let request = Request::builder()
            .uri("/witchcraft-ete/api/test/slowBody?delayMillis=1500")
            .body(Body::empty())
            .unwrap();

        let start = Instant::now();
        let response = client.send_request(request).await.unwrap();
        // make sure we receive headers quickly
        assert!(start.elapsed() < Duration::from_millis(250));
        assert_eq!(response.status(), StatusCode::OK);

        let body = body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&*body, &[0, 0]);

        // should only be at 1 second idle, not 2.5 seconds
        time::sleep(Duration::from_secs(1)).await;

        let request = Request::builder()
            .uri("/witchcraft-ete/api/test/slowHeaders?delayMillis=0")
            .body(Body::empty())
            .unwrap();
        let response = client.send_request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // connection should close after 2.25 seconds of idle time
        time::sleep(Duration::from_millis(2250)).await;

        let request = Request::builder()
            .uri("/witchcraft-ete/api/test/slowHeaders?delayMillis=0")
            .body(Body::empty())
            .unwrap();
        client.send_request(request).await.err().unwrap();

        drop(client);
        server.shutdown().await;
    })
    .await;
}

#[tokio::test]
async fn graceful_shutdown() {
    Server::with(|mut server| async move {
        let request = Request::builder()
            .uri("/witchcraft-ete/api/test/slowBody?delayMillis=1500")
            .body(Body::empty())
            .unwrap();
        let response = server
            .client()
            .await
            .unwrap()
            .send_request(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let mut body = response.into_body();
        let chunk = body.data().await.unwrap().unwrap();
        assert_eq!(&chunk[..], &[0]);

        server.start_shutdown();

        body.data().await.unwrap().unwrap();
        assert_eq!(&chunk[..], &[0]);

        let logs = server.finish_shutdown().await;
        logs.only_request();
    })
    .await;
}

#[tokio::test]
async fn diagnostic_types_diagnostic() {
    Server::with(|server| async move {
        let request = Request::builder()
            .uri("/witchcraft-ete/debug/diagnostic/diagnostic.types.v1")
            .header("Authorization", "Bearer debug")
            .body(Body::empty())
            .unwrap();
        let response = server
            .client()
            .await
            .unwrap()
            .send_request(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("Safe-Loggable").unwrap(), "true");
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );

        let body = body::to_bytes(response.into_body()).await.unwrap();
        let body = str::from_utf8(&body).unwrap();
        assert!(body.contains("\"diagnostic.types.v1\""));

        server.shutdown().await;
    })
    .await;
}

#[tokio::test]
async fn trailers() {
    Server::builder()
        .http2(true)
        .with(|server| async move {
            let request = Request::builder()
                .method("POST")
                .uri("/witchcraft-ete/api/test/trailers")
                .header("Content-Type", "application/octet-stream")
                .body(TrailersBody { sent_body: false })
                .unwrap();
            let mut response = server
                .client()
                .await
                .unwrap()
                .send_request(request)
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let bytes = body::to_bytes(&mut response).await.unwrap();
            assert_eq!(bytes, "expected response body");

            let trailers = response.trailers().await.unwrap().unwrap();
            assert_eq!(
                trailers.get("Response-Trailer").unwrap(),
                "expected response trailer value",
            );

            server.shutdown().await;
        })
        .await;
}

struct TrailersBody {
    sent_body: bool,
}

impl HttpBody for TrailersBody {
    type Data = Bytes;

    type Error = String;

    fn poll_data(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        if self.sent_body {
            Poll::Ready(None)
        } else {
            self.sent_body = true;
            Poll::Ready(Some(Ok(Bytes::from("expected request body"))))
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        let mut trailers = HeaderMap::new();
        trailers.insert(
            "Request-Trailer",
            HeaderValue::from_static("expected request trailer value"),
        );
        Poll::Ready(Ok(Some(trailers)))
    }
}
