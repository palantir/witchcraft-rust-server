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
use crate::service::peer_addr::GetPeerAddr;
use crate::service::Service;
use conjure_error::Error;
use futures_util::ready;
use pin_project::pin_project;
use socket2::{Domain, SockAddr, SockRef, Socket, TcpKeepalive, Type};
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{fs, io};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{self, Sleep};
use witchcraft_log::warn;
use witchcraft_server_config::install::InstallConfig;

// This is pretty arbitrary - I just copied it from some Cloudflare blog post.
const TCP_KEEPALIVE: Duration = Duration::from_secs(3 * 60);

/// The root service of the socket service stack which accept raw TCP connections.
pub struct AcceptService {
    listener: Arc<TcpListener>,
}

impl AcceptService {
    pub fn new(config: &InstallConfig) -> Result<Self, Error> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port());

        let listener =
            Socket::new(Domain::IPV4, Type::STREAM, None).map_err(Error::internal_safe)?;
        listener
            .set_nonblocking(true)
            .map_err(Error::internal_safe)?;
        listener
            .set_reuse_address(true)
            .map_err(Error::internal_safe)?;
        listener
            .bind(&SockAddr::from(addr))
            .map_err(Error::internal_safe)?;
        listener.listen(somaxconn()).map_err(Error::internal_safe)?;

        let listener = TcpListener::from_std(listener.into()).map_err(Error::internal_safe)?;

        Ok(AcceptService {
            listener: Arc::new(listener),
        })
    }
}

impl Service<()> for AcceptService {
    type Response = TcpStream;

    type Future = AcceptFuture;

    fn call(&self, _: ()) -> Self::Future {
        AcceptFuture {
            listener: self.listener.clone(),
            sleep: None,
        }
    }
}

#[pin_project]
pub struct AcceptFuture {
    listener: Arc<TcpListener>,
    #[pin]
    sleep: Option<Sleep>,
}

impl Future for AcceptFuture {
    type Output = TcpStream;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            if let Some(sleep) = this.sleep.as_mut().as_pin_mut() {
                ready!(sleep.poll(cx));
                this.sleep.set(None);
            }

            match ready!(this.listener.poll_accept(cx)) {
                Ok((socket, _)) => match setup_socket(&socket) {
                    Ok(()) => return Poll::Ready(socket),
                    Err(e) => warn!("error configuring socket", error: Error::internal_safe(e)),
                },
                // There are 3 broad categories of error we can encounter from accept:
                // 1. The call was interrupted due to a signal, and we should just retry. This won't normally happen if
                //      SA_RESTART is set, but we shouldn't assume that's the case.
                // 2. We hit a system resource limit. We want to wait a bit before retrying so we don't hot loop.
                // 3. We hit some other error. This could be something we don't expect to be possible like the listener
                //      socket being closed, or (on Linux) an error that the connection we're accepting entered into
                //      before leaving the accept queue. We just want to log it and keep going.
                Err(e) => match e.raw_os_error() {
                    Some(libc::EINTR) => {}
                    Some(libc::EMFILE | libc::ENFILE | libc::ENOBUFS | libc::ENOMEM) => {
                        warn!(
                            "hit resource limit accepting socket",
                            error: Error::internal_safe(e),
                        );
                        this.sleep.set(Some(time::sleep(Duration::from_secs(1))));
                    }
                    _ => warn!("error accepting socket", error: Error::internal_safe(e)),
                },
            }
        }
    }
}

fn somaxconn() -> i32 {
    fs::read_to_string("/proc/sys/net/core/somaxconn")
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(128)
}

fn setup_socket(stream: &TcpStream) -> io::Result<()> {
    stream.set_nodelay(true)?;
    SockRef::from(stream).set_tcp_keepalive(&TcpKeepalive::new().with_time(TCP_KEEPALIVE))?;
    Ok(())
}

impl GetPeerAddr for TcpStream {
    fn peer_addr(&self) -> Result<SocketAddr, Error> {
        self.peer_addr().map_err(Error::internal_safe)
    }
}
