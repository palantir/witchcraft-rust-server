use api::{LogLevel, RequestLogV2, ServiceLogV1};
use conjure_serde::json;
use hyper::body::HttpBody;
use hyper::client::conn::{self, SendRequest};
use hyper::Request;
use openssl::ssl::{SslConnector, SslMethod};
use std::error::{self, Error};
use std::fs::File;
use std::future::Future;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::{env, fs, thread};
use tempfile::TempDir;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::{task, time};
use tokio_openssl::SslStream;

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

#[allow(warnings)]
#[path = "../../../../witchcraft-server/src/logging/api/mod.rs"]
pub mod api;

// this is a bit racy, but should work in practice
fn open_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn setup_config(dir: &Path, builder: &Builder, port: u16) {
    let conf = dir.join("var/conf");
    fs::create_dir_all(&conf).unwrap();
    fs::write(
        conf.join("encrypted-config-value.key"),
        include_str!("encrypted-config-value.key"),
    )
    .unwrap();
    fs::write(
        conf.join("install.yml"),
        include_str!("install.yml")
            .replace("<PORT>", &port.to_string())
            .replace("<HTTP2>", &builder.http2.to_string()),
    )
    .unwrap();
    fs::write(conf.join("runtime.yml"), include_str!("runtime.yml")).unwrap();

    let security = dir.join("var/security");
    fs::create_dir_all(&security).unwrap();
    fs::write(security.join("cert.cer"), include_str!("cert.cer")).unwrap();
    fs::write(security.join("key.pem"), include_str!("key.pem")).unwrap();
}

pub struct Server {
    dir: PathBuf,
    child: Child,
    stdout_rx: Option<oneshot::Receiver<String>>,
    ctx: SslConnector,
    port: u16,
    shutdown: bool,
    http2: bool,
}

impl Drop for Server {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("server dir: {}", self.dir.display());
            if !self.shutdown {
                let _ = self.child.kill();
            }
        } else {
            assert!(self.shutdown, "test did not shut down the server");
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

impl Server {
    pub fn builder() -> Builder {
        Builder { http2: false }
    }

    pub async fn with<F, G>(test: F)
    where
        F: Fn(Server) -> G,
        G: Future<Output = ()>,
    {
        Self::builder().with(test).await
    }

    async fn new(builder: &Builder, handler_type: &str) -> Self {
        let dir = TempDir::new().unwrap();
        let port = open_port();

        setup_config(dir.path(), builder, port);

        let binary = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("../witchcraft-server-ete");
        let mut child = Command::new(binary)
            .current_dir(dir.path())
            .env("HANDLER_TYPE", handler_type)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();

        let mut stdout = child.stdout.take().unwrap();
        let (tx, stdout_rx) = oneshot::channel();
        thread::spawn(move || {
            let mut buf = String::new();
            stdout.read_to_string(&mut buf).unwrap();
            let _ = tx.send(buf);
        });

        let mut ctx = SslConnector::builder(SslMethod::tls()).unwrap();
        ctx.set_ca_file(dir.path().join("var/security/cert.cer"))
            .unwrap();
        if builder.http2 {
            ctx.set_alpn_protos(b"\x02h2").unwrap();
        }
        let file = File::create(dir.path().join("keylog")).unwrap();
        ctx.set_keylog_callback(move |_, b| {
            let _ = (&file).write_all(b.as_bytes());
            let _ = (&file).write_all(b"\n");
        });
        let ctx = ctx.build();

        let server = Server {
            dir: dir.into_path(),
            child,
            stdout_rx: Some(stdout_rx),
            ctx,
            port,
            shutdown: false,
            http2: builder.http2,
        };

        server.wait_for_ready().await;

        server
    }

    async fn wait_for_ready(&self) {
        for _ in 0..50 {
            let mut client = match self.client().await {
                Ok(client) => client,
                Err(_) => {
                    time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            };

            let request = Request::builder()
                .uri("/witchcraft-ete/status/readiness")
                .body(hyper::Body::empty())
                .unwrap();

            match client.send_request(request).await {
                Ok(response) if response.status().is_success() => return,
                _ => time::sleep(Duration::from_millis(100)).await,
            }
        }

        panic!("timed out waiting for the server to report readiness");
    }

    pub async fn client<B>(&self) -> Result<SendRequest<B>, Box<dyn Error + Sync + Send>>
    where
        B: HttpBody + 'static + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn error::Error + Sync + Send>>,
    {
        let stream = TcpStream::connect(("127.0.0.1", self.port)).await?;
        let ssl = self.ctx.configure()?.into_ssl("localhost")?;
        let mut stream = SslStream::new(ssl, stream)?;
        Pin::new(&mut stream).connect().await?;
        let (client, connection) = conn::Builder::new()
            .http2_only(self.http2)
            .handshake(stream)
            .await
            .unwrap();

        task::spawn(async {
            let _ = connection.await;
        });

        Ok(client)
    }

    pub async fn shutdown(mut self) -> ServerLogs {
        self.start_shutdown();
        self.finish_shutdown().await
    }

    pub fn start_shutdown(&mut self) {
        unsafe {
            libc::kill(self.child.id() as libc::pid_t, libc::SIGINT);
        }
    }

    pub async fn finish_shutdown(mut self) -> ServerLogs {
        self.shutdown = true;

        let mut logs = ServerLogs {
            service: vec![],
            request: vec![],
        };

        let stdout = self.stdout_rx.take().unwrap().await.unwrap();
        for line in stdout.lines() {
            if line.contains("service.1") {
                logs.service.push(json::server_from_str(line).unwrap());
            } else if line.contains("request.2") {
                logs.request.push(json::server_from_str(line).unwrap());
            }
        }

        logs.assert_no_errors();
        logs
    }
}

pub struct Builder {
    http2: bool,
}

impl Builder {
    pub fn http2(mut self, http2: bool) -> Self {
        self.http2 = http2;
        self
    }

    pub async fn with<F, G>(self, test: F)
    where
        F: Fn(Server) -> G,
        G: Future<Output = ()>,
    {
        for handler_type in ["blocking", "async"] {
            let server = Server::new(&self, handler_type).await;
            test(server).await;
        }
    }
}

pub struct ServerLogs {
    pub service: Vec<ServiceLogV1>,
    pub request: Vec<RequestLogV2>,
}

impl ServerLogs {
    fn assert_no_errors(&self) {
        for line in &self.service {
            match line.level() {
                LogLevel::Trace | LogLevel::Debug | LogLevel::Info => {}
                _ => panic!("service error: {:#?}", line),
            }
        }
    }

    pub fn only_request(&self) -> &RequestLogV2 {
        // filter out readiness calls
        let mut real_logs = self
            .request
            .iter()
            .filter(|l| l.path().starts_with("/witchcraft-ete/api/"));
        let log = real_logs.next().expect("should have a request log");
        if real_logs.next().is_some() {
            panic!("expected only one request log");
        }
        log
    }
}
