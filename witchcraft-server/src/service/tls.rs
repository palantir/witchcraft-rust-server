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
use crate::service::{Layer, Service, ServiceBuilder};
use conjure_error::Error;
use futures_util::ready;
use pin_project::pin_project;
use rustls_pemfile::Item;
use std::fs::File;
use std::future::Future;
use std::io::BufReader;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::rustls::crypto::ring::cipher_suite::{
    TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256, TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256, TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384, TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
};
use tokio_rustls::rustls::crypto::ring::kx_group::{SECP256R1, SECP384R1, X25519};
use tokio_rustls::rustls::crypto::{ring, CryptoProvider, SupportedKxGroup};
use tokio_rustls::rustls::server::WebPkiClientVerifier;
use tokio_rustls::rustls::version::{TLS12, TLS13};
use tokio_rustls::rustls::{
    RootCertStore, ServerConfig, SupportedCipherSuite, SupportedProtocolVersion,
};
use tokio_rustls::server::TlsStream;
use tokio_rustls::{Accept, TlsAcceptor};
use webpki::types::{CertificateDer, PrivateKeyDer};
use witchcraft_server_config::install::InstallConfig;

static CIPHER_SUITES: [SupportedCipherSuite; 9] = [
    TLS13_AES_256_GCM_SHA384,
    TLS13_AES_128_GCM_SHA256,
    TLS13_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
];

static KX_GROUPS: [&dyn SupportedKxGroup; 3] = [SECP256R1, SECP384R1, X25519];

static PROTOCOL_VERSIONS: [&SupportedProtocolVersion; 2] = [&TLS12, &TLS13];

/// A layer which wraps streams in a TLS session.
pub struct TlsLayer {
    acceptor: TlsAcceptor,
}

impl TlsLayer {
    pub fn new(config: &InstallConfig) -> Result<Self, Error> {
        let provider = CryptoProvider {
            cipher_suites: CIPHER_SUITES.to_vec(),
            kx_groups: KX_GROUPS.to_vec(),
            ..ring::default_provider()
        };

        let builder = ServerConfig::builder_with_provider(Arc::new(provider))
            .with_protocol_versions(&PROTOCOL_VERSIONS)
            .map_err(Error::internal_safe)?;

        let builder = match config.client_auth_truststore() {
            Some(client_auth_truststore) => {
                let certs = load_certificates(client_auth_truststore.path())?;
                let mut store = RootCertStore::empty();
                store.add_parsable_certificates(certs);
                builder.with_client_cert_verifier(
                    WebPkiClientVerifier::builder(Arc::new(store))
                        .allow_unauthenticated()
                        .build()
                        .map_err(Error::internal_safe)?,
                )
            }
            None => builder.with_no_client_auth(),
        };

        let cert_chain = load_certificates(config.keystore().cert_path())?;
        let key_der = load_private_key(config.keystore().key_path())?;

        let mut server_config = builder
            .with_single_cert(cert_chain, key_der)
            .map_err(Error::internal_safe)?;

        server_config.ignore_client_order = true;
        if config.server().http2() {
            server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        }

        Ok(TlsLayer {
            acceptor: TlsAcceptor::from(Arc::new(server_config)),
        })
    }
}

fn load_certificates(path: &Path) -> Result<Vec<CertificateDer<'static>>, Error> {
    let file = File::open(path).map_err(Error::internal_safe)?;
    let mut reader = BufReader::new(file);
    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Error::internal_safe)
}

fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>, Error> {
    let file = File::open(path).map_err(Error::internal_safe)?;
    let mut reader = BufReader::new(file);

    let mut items = rustls_pemfile::read_all(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Error::internal_safe)?;

    if items.len() != 1 {
        return Err(Error::internal_safe(
            "expected exactly one private key in key file",
        ));
    }

    match items.pop().unwrap() {
        Item::Pkcs1Key(key) => Ok(key.into()),
        Item::Pkcs8Key(key) => Ok(key.into()),
        Item::Sec1Key(key) => Ok(key.into()),
        _ => Err(Error::internal_safe(
            "expected a PKCS#1, PKCS#8, or Sec1 private key",
        )),
    }
}

impl<S> Layer<S> for TlsLayer {
    type Service = TlsService<S>;

    fn layer(self, inner: S) -> Self::Service {
        TlsService {
            inner: Arc::new(inner),
            acceptor: self.acceptor,
        }
    }
}

pub struct TlsService<S> {
    inner: Arc<S>,
    acceptor: TlsAcceptor,
}

impl<S, R, L> Service<NewConnection<R, L>> for TlsService<S>
where
    S: Service<NewConnection<TlsStream<R>, L>, Response = Result<(), Error>>,
    R: AsyncRead + AsyncWrite + Unpin,
{
    type Response = S::Response;

    type Future = TlsFuture<S, R, L>;

    fn call(&self, req: NewConnection<R, L>) -> Self::Future {
        TlsFuture::Handshaking {
            accept: self.acceptor.accept(req.stream),
            service_builder: Some(req.service_builder),
            inner: self.inner.clone(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[pin_project(project = TlsFutureProj)]
pub enum TlsFuture<S, R, L>
where
    S: Service<NewConnection<TlsStream<R>, L>>,
{
    Handshaking {
        accept: Accept<R>,
        service_builder: Option<ServiceBuilder<L>>,
        inner: Arc<S>,
    },
    Inner {
        #[pin]
        future: S::Future,
    },
}

impl<S, R, L> Future for TlsFuture<S, R, L>
where
    S: Service<NewConnection<TlsStream<R>, L>, Response = Result<(), Error>>,
    R: AsyncRead + AsyncWrite + Unpin,
{
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project() {
                TlsFutureProj::Handshaking {
                    inner,
                    accept,
                    service_builder,
                } => match ready!(Pin::new(accept).poll(cx)) {
                    Ok(stream) => {
                        let new = TlsFuture::Inner {
                            future: inner.call(NewConnection {
                                stream,
                                service_builder: service_builder.take().unwrap(),
                            }),
                        };
                        self.set(new);
                    }
                    Err(e) => return Poll::Ready(Err(Error::internal_safe(e))),
                },
                TlsFutureProj::Inner { future } => return future.poll(cx),
            }
        }
    }
}
