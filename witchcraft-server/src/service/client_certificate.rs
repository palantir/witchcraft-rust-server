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
use crate::service::hyper::NewConnection;
use crate::service::{Layer, Service, Stack};
use crate::tls::ClientCertificate;
use http::Request;
use tokio_rustls::server::TlsStream;
use webpki::types::CertificateDer;

/// A layer which injects a [`ClientCertificate`] extension into all requests made over the connection.
pub struct ClientCertificateLayer;

impl<S> Layer<S> for ClientCertificateLayer {
    type Service = ClientCertificateService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ClientCertificateService { inner }
    }
}

pub struct ClientCertificateService<S> {
    inner: S,
}

impl<S, T, L> Service<NewConnection<TlsStream<T>, L>> for ClientCertificateService<S>
where
    S: Service<NewConnection<TlsStream<T>, Stack<L, ClientCertificateRequestLayer>>>,
{
    type Response = S::Response;

    type Future = S::Future;

    fn call(&self, req: NewConnection<TlsStream<T>, L>) -> Self::Future {
        let cert = req
            .stream
            .get_ref()
            .1
            .peer_certificates()
            .and_then(|c| c.first())
            .cloned()
            .map(CertificateDer::into_owned)
            .map(ClientCertificate::new);

        self.inner.call(NewConnection {
            stream: req.stream,
            service_builder: req
                .service_builder
                .layer(ClientCertificateRequestLayer { cert }),
        })
    }
}

pub struct ClientCertificateRequestLayer {
    cert: Option<ClientCertificate>,
}

impl<S> Layer<S> for ClientCertificateRequestLayer {
    type Service = ClientCertificateRequestService<S>;

    fn layer(self, inner: S) -> Self::Service {
        ClientCertificateRequestService {
            inner,
            cert: self.cert,
        }
    }
}

pub struct ClientCertificateRequestService<S> {
    inner: S,
    cert: Option<ClientCertificate>,
}

impl<S, B> Service<Request<B>> for ClientCertificateRequestService<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;

    type Future = S::Future;

    fn call(&self, mut req: Request<B>) -> Self::Future {
        if let Some(cert) = &self.cert {
            req.extensions_mut().insert(cert.clone());
        }

        self.inner.call(req)
    }
}
