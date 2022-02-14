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
use openssl::x509::{X509Ref, X509};

/// A client's identity provided during the TLS handshake.
///
/// If client authentication is enabled and a client provides a certificate during the TLS handshake, this will be added
/// to the extensions of each request made on that connection.
#[derive(Clone)]
pub struct ClientCertificate {
    cert: X509,
}

// FIXME(sfackler) what accessors should we expose here? We probably want to avoid exposing `openssl` APIs directly.
impl ClientCertificate {
    pub(crate) fn new(cert: X509) -> Self {
        ClientCertificate { cert }
    }

    pub(crate) fn x509(&self) -> &X509Ref {
        &self.cert
    }
}
