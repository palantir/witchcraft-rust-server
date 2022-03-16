// Copyright 2021 Palantir Technologies, Inc.
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
//! Fixed configuration.
use crate::ConfigError;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use staged_builder::{staged_builder, Validate};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

mod de;

/// The fixed configuration for a Witchcraft server.
#[derive(Clone, PartialEq, Debug)]
#[staged_builder]
#[builder(validate)]
pub struct InstallConfig {
    #[builder(into)]
    product_name: String,
    #[builder(into)]
    product_version: String,
    port: u16,
    #[builder(default)]
    keystore: KeystoreConfig,
    #[builder(default, into)]
    client_auth_truststore: Option<ClientAuthTruststoreConfig>,
    #[builder(into, default = "/".to_string())]
    context_path: String,
    #[builder(default = env::var_os("CONTAINER").is_some())]
    use_console_log: bool,
    #[builder(default)]
    server: ServerConfig,
}

impl Validate for InstallConfig {
    type Error = ConfigError;

    fn validate(&self) -> Result<(), Self::Error> {
        if !(self.context_path == "/"
            || (self.context_path.starts_with('/') && !self.context_path.ends_with('/')))
        {
            return Err(ConfigError(
                "context-path must either be `/` or start but not end with a `/`".to_string(),
            ));
        }

        Ok(())
    }
}

impl<'de> Deserialize<'de> for InstallConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = de::InstallConfig::deserialize(deserializer)?;
        let mut builder = InstallConfig::builder()
            .product_name(raw.product_name)
            .product_version(raw.product_version)
            .port(raw.port);
        if let Some(keystore) = raw.keystore {
            builder = builder.keystore(keystore);
        }
        if let Some(client_auth_truststore) = raw.client_auth_truststore {
            builder = builder.client_auth_truststore(client_auth_truststore);
        }
        if let Some(context_path) = raw.context_path {
            builder = builder.context_path(context_path);
        }
        if let Some(use_console_log) = raw.use_console_log {
            builder = builder.use_console_log(use_console_log);
        }
        if let Some(server) = raw.server {
            builder = builder.server(server);
        }

        builder.build().map_err(Error::custom)
    }
}

impl AsRef<InstallConfig> for InstallConfig {
    #[inline]
    fn as_ref(&self) -> &InstallConfig {
        self
    }
}

impl InstallConfig {
    /// Returns the service's name.
    ///
    /// Required.
    #[inline]
    pub fn product_name(&self) -> &str {
        &self.product_name
    }

    /// Returns the service's version.
    ///
    /// Required.
    #[inline]
    pub fn product_version(&self) -> &str {
        &self.product_version
    }

    /// Returns the port the server will listen on.
    ///
    /// Required.
    #[inline]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the server's TLS key configuration.
    #[inline]
    pub fn keystore(&self) -> &KeystoreConfig {
        &self.keystore
    }

    /// Returns the server's TLS client authentication truststore configuration.
    ///
    /// If set, the server will request (but not require) the client to authenticate itself during the TLS handshake.
    /// If a certificate is present and validated against the trust roots, all requests made over that connection will
    /// include a `ClientCertificate` extension.
    ///
    /// Defaults to `None`.
    #[inline]
    pub fn client_auth_truststore(&self) -> Option<&ClientAuthTruststoreConfig> {
        self.client_auth_truststore.as_ref()
    }

    /// Returns the server's context path.
    ///
    /// This must either be equal to `/` or start but not end with a `/`.
    ///
    /// Defaults to `/`.
    #[inline]
    pub fn context_path(&self) -> &str {
        &self.context_path
    }

    /// If `true`, the server will log to standard output rather than to files.
    ///
    /// Defaults to `true` if running in a container and false otherwise.
    #[inline]
    pub fn use_console_log(&self) -> bool {
        self.use_console_log
    }

    /// Returns advanced server settings.
    #[inline]
    pub fn server(&self) -> &ServerConfig {
        &self.server
    }
}

/// TLS key configuration.
#[derive(Clone, PartialEq, Debug)]
#[staged_builder]
pub struct KeystoreConfig {
    #[builder(into, default = PathBuf::from("var/security/key.pem"))]
    key_path: PathBuf,
    #[builder(into, default = PathBuf::from("var/security/cert.cer"))]
    cert_path: PathBuf,
}

impl Default for KeystoreConfig {
    #[inline]
    fn default() -> Self {
        KeystoreConfig::builder().build()
    }
}

impl<'de> Deserialize<'de> for KeystoreConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = de::KeystoreConfig::deserialize(deserializer)?;
        let mut builder = KeystoreConfig::builder();
        if let Some(key_path) = raw.key_path {
            builder = builder.key_path(key_path);
        }
        if let Some(cert_path) = raw.cert_path {
            builder = builder.cert_path(cert_path);
        }
        Ok(builder.build())
    }
}

impl KeystoreConfig {
    /// Returns the path to the server's PEM-encoded private key.
    ///
    /// Defaults to `var/security/key.pem`.
    #[inline]
    pub fn key_path(&self) -> &Path {
        &self.key_path
    }

    /// Returns the path to the server's PEM-encoded certificate chain.
    ///
    /// The file should contain a sequence of certificates starting with the leaf certificate corresponding to the key
    /// in [`Self::key_path`] followed by the rest of the certificate chain up to a trusted root.
    ///
    /// Defaults to `var/security/cert.cer`.
    #[inline]
    pub fn cert_path(&self) -> &Path {
        &self.cert_path
    }
}

/// TLS client authentication configuration.
#[derive(Clone, PartialEq, Debug)]
#[staged_builder]
pub struct ClientAuthTruststoreConfig {
    #[builder(into, default = PathBuf::from("var/security/ca.cer"))]
    path: PathBuf,
}

impl Default for ClientAuthTruststoreConfig {
    #[inline]
    fn default() -> Self {
        ClientAuthTruststoreConfig::builder().build()
    }
}

impl<'de> Deserialize<'de> for ClientAuthTruststoreConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = de::ClientAuthTruststoreConfig::deserialize(deserializer)?;
        let mut builder = ClientAuthTruststoreConfig::builder();
        if let Some(path) = raw.path {
            builder = builder.path(path);
        }
        Ok(builder.build())
    }
}

impl ClientAuthTruststoreConfig {
    /// Returns the path to a file containg PEM-encoded certificates (i.e. blocks of `-----BEGIN CERTIFICATE-----`)
    /// that will act as the trust roots for validating the client's identity.
    ///
    /// Defaults to `var/security/ca.cer`.
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Advanced server configuration.
#[derive(Clone, PartialEq, Debug)]
#[staged_builder]
pub struct ServerConfig {
    #[builder(default = num_cpus::get())]
    processors: usize,
    #[builder(default, custom(type = usize, convert = Some))]
    min_threads: Option<usize>,
    #[builder(default, custom(type = usize, convert = Some))]
    max_threads: Option<usize>,
    #[builder(default, custom(type = usize, convert = Some))]
    max_connections: Option<usize>,
    #[builder(default, custom(type = usize, convert = Some))]
    io_threads: Option<usize>,
    #[builder(default = Duration::from_secs(5 * 60))]
    idle_thread_timeout: Duration,
    #[builder(default = Duration::from_secs(15))]
    shutdown_timeout: Duration,
    #[builder(default = true)]
    gzip: bool,
    #[builder(default = false)]
    http2: bool,
    #[builder(default, into)]
    idle_connection_timeout: Option<Duration>,
}

impl Default for ServerConfig {
    #[inline]
    fn default() -> Self {
        ServerConfig::builder().build()
    }
}

impl<'de> Deserialize<'de> for ServerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = de::ServerConfig::deserialize(deserializer)?;
        let mut builder = ServerConfig::builder();
        if let Some(processors) = raw.processors {
            builder = builder.processors(processors);
        }
        if let Some(min_threads) = raw.min_threads {
            builder = builder.min_threads(min_threads);
        }
        if let Some(max_threads) = raw.max_threads {
            builder = builder.max_threads(max_threads);
        }
        if let Some(max_connections) = raw.max_connections {
            builder = builder.max_connections(max_connections);
        }
        if let Some(io_threads) = raw.io_threads {
            builder = builder.io_threads(io_threads);
        }
        if let Some(idle_thread_timeout) = raw.idle_thread_timeout {
            builder = builder.idle_thread_timeout(idle_thread_timeout);
        }
        if let Some(shutdown_timeout) = raw.shutdown_timeout {
            builder = builder.shutdown_timeout(shutdown_timeout);
        }
        if let Some(gzip) = raw.gzip {
            builder = builder.gzip(gzip);
        }
        if let Some(http2) = raw.http2 {
            builder = builder.http2(http2);
        }
        if let Some(idle_connection_timeout) = raw.idle_connection_timeout {
            builder = builder.idle_connection_timeout(idle_connection_timeout);
        }

        Ok(builder.build())
    }
}

impl ServerConfig {
    /// Returns the number of processors the server is allocated.
    ///
    /// This is only used to derive default values for other settings in this type.
    ///
    /// Defaults to the number of logical CPUs.
    #[inline]
    pub fn processors(&self) -> usize {
        self.processors
    }

    /// Returns the minimum number of threads in the pool used to process blocking endpoints.
    ///
    /// Defaults to 8 times the number of processors.
    #[inline]
    pub fn min_threads(&self) -> usize {
        self.min_threads.unwrap_or_else(|| self.processors * 8)
    }

    /// Returns the maximum number of threads in the pool used to process blocking endpoints.
    ///
    /// Defaults to the maximum of 32 times the number of processors and 256.
    #[inline]
    pub fn max_threads(&self) -> usize {
        self.max_threads
            .unwrap_or_else(|| usize::max(self.processors * 32, 256))
    }

    /// Returns the maximum number of live TCP connections the server will allow at any time.
    ///
    /// Defaults to 10 times the value of [`Self::max_threads`].
    #[inline]
    pub fn max_connections(&self) -> usize {
        self.max_connections
            .unwrap_or_else(|| self.max_threads() * 10)
    }

    /// Returns the number of threads used for nonblocking operations in the server's Tokio runtime.
    ///
    /// Defaults to half the number of processors.
    #[inline]
    pub fn io_threads(&self) -> usize {
        self.io_threads
            .unwrap_or_else(|| usize::max(1, self.processors / 2))
    }

    /// Returns the amount of time a thread in the blocking request pool will sit idle before shutting down.
    ///
    /// Defaults to 5 minutes.
    #[inline]
    pub fn idle_thread_timeout(&self) -> Duration {
        self.idle_thread_timeout
    }

    /// Returns the amount of time the server will wait for pending requests to complete when shutting down.
    ///
    /// Defaults to 15 seconds.
    #[inline]
    pub fn shutdown_timeout(&self) -> Duration {
        self.shutdown_timeout
    }

    /// Determines if responses larger than 1 MiB will be compressed with gzip.
    ///
    /// Defaults to `true`.
    #[inline]
    pub fn gzip(&self) -> bool {
        self.gzip
    }

    /// Determines if the server will support the HTTP2 protocol.
    ///
    /// Defaults to `false`.
    #[inline]
    pub fn http2(&self) -> bool {
        self.http2
    }

    /// Returns the amount of time the server allows TCP connections to remain idle before shutting them down.
    ///
    /// If `None`, defaults to 1 minute. If `Some`, the time will be included in HTTP responses in a `Keep-Alive`
    /// header.
    #[inline]
    pub fn idle_connection_timeout(&self) -> Option<Duration> {
        self.idle_connection_timeout
    }
}
