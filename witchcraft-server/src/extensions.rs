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

//! Types used with the extensions maps of requests or responses in a Witchcraft server.

use crate::audit::AuditLogV3;
use std::net::SocketAddr;
use std::ops::Deref;

/// An extension containing the peer's socket address.
///
/// It will be present in the extensions of every request.
#[derive(Copy, Clone)]
pub struct PeerAddr(pub(crate) SocketAddr);

impl Deref for PeerAddr {
    type Target = SocketAddr;

    #[inline]
    fn deref(&self) -> &SocketAddr {
        &self.0
    }
}

/// An extension containing an audit log entry for a request.
///
/// If this is present in the response extensions of a request, it will be written to the audit log before the server
/// sends the response to the client. An error logging the entry will cause the request to fail.
pub struct AuditLogEntry(pub(crate) AuditLogV3);

impl AuditLogEntry {
    /// Creates a new `AuditLogEntry` containing a v3 audit log.
    #[inline]
    pub fn v3(entry: AuditLogV3) -> Self {
        AuditLogEntry(entry)
    }
}
