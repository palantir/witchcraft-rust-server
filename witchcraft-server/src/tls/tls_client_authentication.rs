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
use crate::tls::ClientCertificate;
use async_trait::async_trait;
use conjure_error::{Error, PermissionDenied};
use conjure_http::server::{
    AsyncEndpoint, AsyncResponseBody, AsyncService, Endpoint, EndpointMetadata, PathSegment,
    ResponseBody, Service,
};
use http::{Extensions, Method, Request, Response};
use openssl::nid::Nid;
use openssl::stack::Stack;
use openssl::x509::{GeneralName, X509NameRef};
use refreshable::Refreshable;
use std::collections::HashSet;
use std::str;
use std::sync::Arc;

/// A service adapter which validates a client's certificate against a collection of allowed common names.
///
/// Requests will be rejected if the client did not provide a certificate or if the certificate does not have an allowed
/// subject name. The validation logic prioritizes subject alternate name entries - the certificate's subject name will
/// only be inspected if no SAN entries are present. Only DNSname entries are supported.
pub struct TlsClientAuthenticationService<T> {
    inner: T,
    trusted_subject_names: Arc<Refreshable<HashSet<String>, Error>>,
}

impl<T> TlsClientAuthenticationService<T> {
    /// Creates a new service which will validate the subject name of a client's certificate for each request.
    ///
    /// The inner service can implement either the [`Service`] or [`AsyncService`] trait.
    pub fn new(inner: T, trusted_subject_names: Arc<Refreshable<HashSet<String>, Error>>) -> Self {
        TlsClientAuthenticationService {
            inner,
            trusted_subject_names,
        }
    }
}

impl<T, I, O> Service<I, O> for TlsClientAuthenticationService<T>
where
    T: Service<I, O>,
    I: 'static,
    O: 'static,
{
    fn endpoints(&self) -> Vec<Box<dyn Endpoint<I, O> + Sync + Send>> {
        self.inner
            .endpoints()
            .into_iter()
            .map(|inner| {
                Box::new(TlsClientAuthenticationEndpoint {
                    inner,
                    trusted_subject_names: self.trusted_subject_names.clone(),
                }) as _
            })
            .collect()
    }
}

impl<T, I, O> AsyncService<I, O> for TlsClientAuthenticationService<T>
where
    T: AsyncService<I, O>,
    I: 'static + Send,
    O: 'static,
{
    fn endpoints(&self) -> Vec<Box<dyn AsyncEndpoint<I, O> + Sync + Send>> {
        self.inner
            .endpoints()
            .into_iter()
            .map(|inner| {
                Box::new(TlsClientAuthenticationEndpoint {
                    inner,
                    trusted_subject_names: self.trusted_subject_names.clone(),
                }) as _
            })
            .collect()
    }
}

struct TlsClientAuthenticationEndpoint<T> {
    inner: T,
    trusted_subject_names: Arc<Refreshable<HashSet<String>, Error>>,
}

impl<T> TlsClientAuthenticationEndpoint<T> {
    fn check_request<I>(&self, req: &Request<I>) -> Result<(), Error> {
        let client_cert = match req.extensions().get::<ClientCertificate>() {
            Some(client_cert) => client_cert,
            None => {
                return Err(Error::service_safe(
                    "client did not provide a certificate",
                    PermissionDenied::new(),
                ))
            }
        };

        let matches = match client_cert.x509().subject_alt_names() {
            Some(sans) => self.check_subject_alternate_names(sans),
            None => self.check_subject_name(client_cert.x509().subject_name()),
        };

        if matches {
            Ok(())
        } else {
            Err(Error::service_safe(
                "client certificate's subject name(s) not allowed",
                PermissionDenied::new(),
            ))
        }
    }

    fn check_subject_alternate_names(&self, sans: Stack<GeneralName>) -> bool {
        let trusted_subject_names = self.trusted_subject_names.get();

        for san in sans {
            let dnsname = match san.dnsname() {
                Some(dnsname) => dnsname,
                None => continue,
            };

            if trusted_subject_names.contains(dnsname) {
                return true;
            }
        }

        false
    }

    fn check_subject_name(&self, subject_name: &X509NameRef) -> bool {
        let trusted_subject_names = self.trusted_subject_names.get();

        for common_name in subject_name.entries_by_nid(Nid::COMMONNAME) {
            let common_name = match str::from_utf8(common_name.data().as_slice()) {
                Ok(common_name) => common_name,
                Err(_) => continue,
            };

            if trusted_subject_names.contains(common_name) {
                return true;
            }
        }

        false
    }
}

impl<T> EndpointMetadata for TlsClientAuthenticationEndpoint<T>
where
    T: EndpointMetadata,
{
    fn method(&self) -> Method {
        self.inner.method()
    }

    fn path(&self) -> &[PathSegment] {
        self.inner.path()
    }

    fn template(&self) -> &str {
        self.inner.template()
    }

    fn service_name(&self) -> &str {
        self.inner.service_name()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn deprecated(&self) -> Option<&str> {
        self.inner.deprecated()
    }
}

impl<T, I, O> Endpoint<I, O> for TlsClientAuthenticationEndpoint<T>
where
    T: Endpoint<I, O>,
{
    fn handle(
        &self,
        req: Request<I>,
        response_extensions: &mut Extensions,
    ) -> Result<Response<ResponseBody<O>>, Error> {
        self.check_request(&req)?;
        self.inner.handle(req, response_extensions)
    }
}

#[async_trait]
impl<T, I, O> AsyncEndpoint<I, O> for TlsClientAuthenticationEndpoint<T>
where
    T: AsyncEndpoint<I, O> + Sync + Send,
    I: Send,
{
    async fn handle(
        &self,
        req: Request<I>,
        response_extensions: &mut Extensions,
    ) -> Result<Response<AsyncResponseBody<O>>, Error>
    where
        I: 'async_trait,
    {
        self.check_request(&req)?;
        self.inner.handle(req, response_extensions).await
    }
}
