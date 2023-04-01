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
use tokio_rustls::rustls::cipher_suite::{
    TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
    TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256, TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
    TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256, TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
    TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384, TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
};
use tokio_rustls::rustls::kx_group::{SECP256R1, SECP384R1, X25519};
use tokio_rustls::rustls::server::AllowAnyAnonymousOrAuthenticatedClient;
use tokio_rustls::rustls::version::{TLS12, TLS13};
use tokio_rustls::rustls::{
    Certificate, PrivateKey, RootCertStore, ServerConfig, SupportedCipherSuite, SupportedKxGroup,
    SupportedProtocolVersion,
};
use tokio_rustls::server::TlsStream;
use tokio_rustls::{Accept, TlsAcceptor};
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

static KX_GROUPS: [&SupportedKxGroup; 3] = [&SECP256R1, &SECP384R1, &X25519];

static PROTOCOL_VERSIONS: [&SupportedProtocolVersion; 2] = [&TLS12, &TLS13];

/// A layer which wraps streams in a TLS session.
pub struct TlsLayer {
    acceptor: TlsAcceptor,
}

impl TlsLayer {
    pub fn new(config: &InstallConfig) -> Result<Self, Error> {
        let builder = ServerConfig::builder()
            .with_cipher_suites(&CIPHER_SUITES)
            .with_kx_groups(&KX_GROUPS)
            .with_protocol_versions(&PROTOCOL_VERSIONS)
            .map_err(Error::internal_safe)?;

        let builder = match config.client_auth_truststore() {
            Some(client_auth_truststore) => {
                let certs = load_certificates(client_auth_truststore.path())?;
                let mut store = RootCertStore::empty();
                store.add_parsable_certificates(&certs);
                builder.with_client_cert_verifier(Arc::new(
                    AllowAnyAnonymousOrAuthenticatedClient::new(store),
                ))
            }
            None => builder.with_no_client_auth(),
        };

        let cert_chain = load_certificates(config.keystore().cert_path())?
            .into_iter()
            .map(Certificate)
            .collect();

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

fn load_certificates(path: &Path) -> Result<Vec<Vec<u8>>, Error> {
    let file = File::open(path).map_err(Error::internal_safe)?;
    let mut reader = BufReader::new(file);
    rustls_pemfile::certs(&mut reader).map_err(Error::internal_safe)
}

fn load_private_key(path: &Path) -> Result<PrivateKey, Error> {
    let file = File::open(path).map_err(Error::internal_safe)?;
    let mut reader = BufReader::new(file);

    let mut items = rustls_pemfile::read_all(&mut reader).map_err(Error::internal_safe)?;

    if items.len() != 1 {
        return Err(Error::internal_safe(
            "expected exactly one private key in key file",
        ));
    }

    match items.pop().unwrap() {
        Item::RSAKey(buf) | Item::PKCS8Key(buf) | Item::ECKey(buf) => Ok(PrivateKey(buf)),
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
