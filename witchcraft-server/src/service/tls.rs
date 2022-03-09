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
use openssl::rand;
use openssl::ssl::{
    self, AlpnError, Ssl, SslContext, SslFiletype, SslMethod, SslOptions, SslVerifyMode, SslVersion,
};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_openssl::SslStream;
use witchcraft_server_config::install::InstallConfig;

const CIPHER_SUITES: &[&str] = &[
    "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA384",
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
    "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
    "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA384",
];

const TLS13_CIPHER_SUITES: &[&str] = &[
    "TLS_AES_256_GCM_SHA384",
    "TLS_AES_128_GCM_SHA256",
    "TLS_CHACHA20_POLY1305_SHA256",
];

/// A layer which wraps streams in a TLS session.
pub struct TlsLayer {
    context: SslContext,
}

impl TlsLayer {
    pub fn new(config: &InstallConfig) -> Result<Self, Error> {
        let mut builder = SslContext::builder(SslMethod::tls()).map_err(Error::internal_safe)?;

        builder
            .set_certificate_chain_file(config.keystore().cert_path())
            .map_err(Error::internal_safe)?;
        builder
            .set_private_key_file(config.keystore().key_path(), SslFiletype::PEM)
            .map_err(Error::internal_safe)?;
        builder.check_private_key().map_err(Error::internal_safe)?;

        if let Some(client_auth) = config.client_auth_truststore() {
            builder
                .set_ca_file(client_auth.path())
                .map_err(Error::internal_safe)?;

            builder.set_verify(SslVerifyMode::PEER);
        }

        let cipher_list = convert_suites(CIPHER_SUITES);
        builder
            .set_cipher_list(&cipher_list)
            .map_err(Error::internal_safe)?;

        let ciphersuites = convert_suites(TLS13_CIPHER_SUITES);
        builder
            .set_ciphersuites(&ciphersuites)
            .map_err(Error::internal_safe)?;

        if config.server().http2() {
            builder.set_alpn_select_callback(|_, client| {
                ssl::select_next_proto(b"\x02h2\x08http/1.1", client).ok_or(AlpnError::ALERT_FATAL)
            });
        }

        builder.set_options(SslOptions::CIPHER_SERVER_PREFERENCE);
        builder
            .set_min_proto_version(Some(SslVersion::TLS1_2))
            .map_err(Error::internal_safe)?;
        builder
            .set_max_proto_version(Some(SslVersion::TLS1_3))
            .map_err(Error::internal_safe)?;

        let mut session_id_ctx = [0; 32];
        rand::rand_bytes(&mut session_id_ctx).map_err(Error::internal_safe)?;
        builder
            .set_session_id_context(&session_id_ctx)
            .map_err(Error::internal_safe)?;

        Ok(TlsLayer {
            context: builder.build(),
        })
    }
}

impl<S> Layer<S> for TlsLayer {
    type Service = TlsService<S>;

    fn layer(self, inner: S) -> Self::Service {
        TlsService {
            inner: Arc::new(inner),
            context: self.context,
        }
    }
}

pub struct TlsService<S> {
    inner: Arc<S>,
    context: SslContext,
}

impl<S, R, L> Service<NewConnection<R, L>> for TlsService<S>
where
    S: Service<NewConnection<SslStream<R>, L>, Response = Result<(), Error>>,
    R: AsyncRead + AsyncWrite + Unpin,
{
    type Response = S::Response;

    type Future = TlsFuture<S, R, L>;

    fn call(&self, req: NewConnection<R, L>) -> Self::Future {
        match Ssl::new(&self.context).and_then(|ssl| SslStream::new(ssl, req.stream)) {
            Ok(stream) => TlsFuture::Handshaking {
                stream: Some(stream),
                service_builder: Some(req.service_builder),
                inner: self.inner.clone(),
            },
            Err(e) => TlsFuture::Error {
                error: Some(Error::internal_safe(e)),
            },
        }
    }
}

#[pin_project(project = TlsFutureProj)]
pub enum TlsFuture<S, R, L>
where
    S: Service<NewConnection<SslStream<R>, L>>,
{
    Handshaking {
        stream: Option<SslStream<R>>,
        service_builder: Option<ServiceBuilder<L>>,
        inner: Arc<S>,
    },
    Inner {
        #[pin]
        future: S::Future,
    },
    Error {
        error: Option<Error>,
    },
}

impl<S, R, L> Future for TlsFuture<S, R, L>
where
    S: Service<NewConnection<SslStream<R>, L>, Response = Result<(), Error>>,
    R: AsyncRead + AsyncWrite + Unpin,
{
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project() {
                TlsFutureProj::Handshaking {
                    inner,
                    stream,
                    service_builder,
                } => match ready!(Pin::new(stream.as_mut().unwrap()).poll_accept(cx)) {
                    Ok(()) => {
                        let new = TlsFuture::Inner {
                            future: inner.call(NewConnection {
                                stream: stream.take().unwrap(),
                                service_builder: service_builder.take().unwrap(),
                            }),
                        };
                        self.set(new);
                    }
                    Err(e) => return Poll::Ready(Err(Error::internal_safe(e))),
                },
                TlsFutureProj::Inner { future } => return future.poll(cx),
                TlsFutureProj::Error { error } => return Poll::Ready(Err(error.take().unwrap())),
            }
        }
    }
}

fn convert_suites(suites: &[&str]) -> String {
    suites
        .iter()
        .filter_map(|s| match ssl::cipher_name(s) {
            "(NONE)" => None,
            s => Some(s),
        })
        .collect::<Vec<_>>()
        .join(":")
}
