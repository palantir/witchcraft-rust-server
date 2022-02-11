use api::{LogLevel, RequestLogV2, ServiceLogV1};
use conjure_serde::json;
use hyper::client::conn::{self, SendRequest};
use hyper::{Body, Request};
use openssl::ssl::{SslConnector, SslMethod};
use std::error::Error;
use std::future::Future;
use std::io::Read;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use std::{env, fs, thread};
use tempfile::TempDir;
use tokio::net::TcpStream;
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

fn setup_config(dir: &Path, port: u16) {
    let conf = dir.join("var/conf");
    fs::create_dir_all(&conf).unwrap();
    fs::write(
        conf.join("encrypted-config-value.key"),
        include_str!("encrypted-config-value.key"),
    )
    .unwrap();
    fs::write(
        conf.join("install.yml"),
        include_str!("install.yml").replace("<PORT>", &port.to_string()),
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
    stdout_rx: mpsc::Receiver<String>,
    ctx: SslConnector,
    port: u16,
    shutdown: bool,
}

impl Drop for Server {
    fn drop(&mut self) {
        println!("server dir: {}", self.dir.display());
        if thread::panicking() {
            if !self.shutdown {
                let _ = self.child.kill();
            }
        } else {
            if !self.shutdown {
                self.shutdown();
            }
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

impl Server {
    pub async fn with<F, G>(test: F)
    where
        F: Fn(Server) -> G,
        G: Future<Output = ()>,
    {
        for handler_type in ["blocking", "async"] {
            let server = Server::new(handler_type).await;
            test(server).await;
        }
    }

    async fn new(handler_type: &str) -> Self {
        let dir = TempDir::new().unwrap();
        let port = open_port();

        setup_config(dir.path(), port);

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
        let (tx, stdout_rx) = mpsc::channel();
        thread::spawn(move || {
            let mut buf = String::new();
            stdout.read_to_string(&mut buf).unwrap();
            let _ = tx.send(buf);
        });

        let mut ctx = SslConnector::builder(SslMethod::tls()).unwrap();
        ctx.set_ca_file(dir.path().join("var/security/cert.cer"))
            .unwrap();
        let ctx = ctx.build();

        let server = Server {
            dir: dir.into_path(),
            child,
            stdout_rx,
            ctx,
            port,
            shutdown: false,
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
                .body(Body::empty())
                .unwrap();

            match client.send_request(request).await {
                Ok(response) if response.status().is_success() => return,
                _ => time::sleep(Duration::from_millis(100)).await,
            }
        }

        panic!(
            "timed out waiting for the server to report readiness:\n{}",
            self.stdout_rx.try_iter().collect::<Vec<_>>().join("\n"),
        );
    }

    pub async fn client(&self) -> Result<SendRequest<Body>, Box<dyn Error + Sync + Send>> {
        let stream = TcpStream::connect(("127.0.0.1", self.port)).await?;
        let ssl = self.ctx.configure()?.into_ssl("localhost")?;
        let mut stream = SslStream::new(ssl, stream)?;
        Pin::new(&mut stream).connect().await?;
        let (client, connection) = conn::handshake(stream).await?;

        task::spawn(async {
            let _ = connection.await;
        });

        Ok(client)
    }

    pub fn shutdown(&mut self) -> ServerLogs {
        self.start_shutdown();
        self.finish_shutdown()
    }

    pub fn start_shutdown(&mut self) {
        self.shutdown = true;
        unsafe {
            libc::kill(self.child.id() as libc::pid_t, libc::SIGINT);
        }
    }

    pub fn finish_shutdown(&mut self) -> ServerLogs {
        let mut logs = ServerLogs {
            service: vec![],
            request: vec![],
        };

        let stdout = self.stdout_rx.recv().unwrap();
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
