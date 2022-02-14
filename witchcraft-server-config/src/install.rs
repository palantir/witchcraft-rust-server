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
use serde::{de, Deserialize, Deserializer};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The fixed configuration for a Witchcraft server.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct InstallConfig {
    product_name: String,
    product_version: String,
    port: u16,
    #[serde(default)]
    keystore: KeystoreConfig,
    client_auth_truststore: Option<ClientAuthTruststoreConfig>,
    #[serde(
        default = "default_install_config_context_path",
        deserialize_with = "deserialize_install_config_context_path"
    )]
    context_path: String,
    #[serde(default = "default_install_config_use_console_log")]
    use_console_log: bool,
    #[serde(default)]
    server: ServerConfig,
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

#[inline]
fn default_install_config_context_path() -> String {
    "/".to_string()
}

#[inline]
fn deserialize_install_config_context_path<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let context_path = String::deserialize(deserializer)?;
    if !(context_path == "/" || (context_path.starts_with('/') && !context_path.ends_with('/'))) {
        return Err(de::Error::invalid_value(
            de::Unexpected::Str(&context_path),
            &"a valid context path",
        ));
    }

    Ok(context_path)
}

#[inline]
fn default_install_config_use_console_log() -> bool {
    env::var_os("CONTAINER").is_some()
}

/// TLS key configuration.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct KeystoreConfig {
    key_path: PathBuf,
    cert_path: PathBuf,
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

impl Default for KeystoreConfig {
    #[inline]
    fn default() -> Self {
        KeystoreConfig {
            key_path: PathBuf::from("var/security/key.pem"),
            cert_path: PathBuf::from("var/security/cert.cer"),
        }
    }
}

/// TLS client authentication configuration.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct ClientAuthTruststoreConfig {
    path: PathBuf,
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

impl Default for ClientAuthTruststoreConfig {
    #[inline]
    fn default() -> Self {
        ClientAuthTruststoreConfig {
            path: PathBuf::from("var/security/ca.cer"),
        }
    }
}

/// Advanced server configuration.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct ServerConfig {
    processors: usize,
    min_threads: Option<usize>,
    max_threads: Option<usize>,
    max_connections: Option<usize>,
    io_threads: Option<usize>,
    #[serde(with = "humantime_serde")]
    idle_thread_timeout: Duration,
    shutdown_timeout: Duration,
    gzip: bool,
    http2: bool,
    #[serde(with = "humantime_serde")]
    idle_connection_timeout: Option<Duration>,
}

impl Default for ServerConfig {
    #[inline]
    fn default() -> Self {
        ServerConfig {
            processors: num_cpus::get(),
            min_threads: None,
            max_threads: None,
            max_connections: None,
            io_threads: None,
            idle_thread_timeout: Duration::from_secs(5 * 60),
            shutdown_timeout: Duration::from_secs(15),
            gzip: true,
            http2: false,
            idle_connection_timeout: None,
        }
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
