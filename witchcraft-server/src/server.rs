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
use crate::logging::api::RequestLogV2;
use crate::logging::Appender;
use crate::service::accept::AcceptService;
use crate::service::catch_unwind::CatchUnwindLayer;
use crate::service::client_certificate::ClientCertificateLayer;
use crate::service::connection_limit::ConnectionLimitLayer;
use crate::service::connection_metrics::ConnectionMetricsLayer;
use crate::service::deprecation_header::DeprecationHeaderLayer;
use crate::service::endpoint_metrics::EndpointMetricsLayer;
use crate::service::error_log::ErrorLogLayer;
use crate::service::gzip::GzipLayer;
use crate::service::handler::HandlerService;
use crate::service::hyper::{HyperService, NewConnection};
use crate::service::idle_connection::IdleConnectionLayer;
use crate::service::keep_alive_header::KeepAliveHeaderLayer;
use crate::service::mdc::MdcLayer;
use crate::service::no_caching::NoCachingLayer;
use crate::service::request_id::RequestIdLayer;
use crate::service::request_log::{RequestLogLayer, RequestLogRequestBody};
use crate::service::routing::RoutingLayer;
use crate::service::server_header::ServerHeaderLayer;
use crate::service::server_metrics::ServerMetricsLayer;
use crate::service::spans::{SpannedBody, SpansLayer};
use crate::service::tls::TlsLayer;
use crate::service::tls_metrics::TlsMetricsLayer;
use crate::service::trace_id_header::TraceIdHeaderLayer;
use crate::service::trace_propagation::TracePropagationLayer;
use crate::service::unverified_jwt::UnverifiedJwtLayer;
use crate::service::web_security::WebSecurityLayer;
use crate::service::witchcraft_mdc::WitchcraftMdcLayer;
use crate::service::{Service, ServiceBuilder};
use crate::Witchcraft;
use conjure_error::Error;
use std::mem;
use std::sync::Arc;
use tokio::task;
use witchcraft_log::debug;

pub type RawBody = RequestLogRequestBody<SpannedBody<hyper::Body>>;

pub async fn start(
    witchcraft: &mut Witchcraft,
    request_logger: Appender<RequestLogV2>,
) -> Result<(), Error> {
    // This service handles individual HTTP requests, each running concurrently.
    let request_service = ServiceBuilder::new()
        .layer(RoutingLayer::new(mem::take(&mut witchcraft.endpoints)))
        .layer(RequestIdLayer)
        .layer(TracePropagationLayer)
        .layer(SpansLayer)
        .layer(UnverifiedJwtLayer)
        .layer(MdcLayer)
        .layer(WitchcraftMdcLayer)
        .layer(RequestLogLayer::new(request_logger))
        .layer(GzipLayer::new(&witchcraft.install_config))
        .layer(DeprecationHeaderLayer)
        .layer(KeepAliveHeaderLayer::new(&witchcraft.install_config))
        .layer(ServerHeaderLayer::new(&witchcraft.install_config)?)
        .layer(NoCachingLayer)
        .layer(WebSecurityLayer)
        .layer(TraceIdHeaderLayer)
        .layer(ServerMetricsLayer::new(&witchcraft.metrics))
        .layer(EndpointMetricsLayer)
        .layer(ErrorLogLayer)
        .layer(CatchUnwindLayer)
        .service(HandlerService);

    // This layer handles individual TCP connections, each running concurrently.
    let handle_service = ServiceBuilder::new()
        .layer(TlsLayer::new(&witchcraft.install_config)?)
        .layer(TlsMetricsLayer::new(&witchcraft.metrics))
        .layer(ClientCertificateLayer)
        .layer(IdleConnectionLayer::new(&witchcraft.install_config))
        .service(HyperService::new(request_service));
    let handle_service = Arc::new(handle_service);

    // This layer produces TCP connections, running serially.
    let accept_service = ServiceBuilder::new()
        .layer(ConnectionLimitLayer::new(&witchcraft.install_config))
        .layer(ConnectionMetricsLayer::new(
            &witchcraft.install_config,
            &witchcraft.metrics,
        ))
        .service(AcceptService::new(&witchcraft.install_config)?);

    task::spawn(async move {
        loop {
            let stream = accept_service.call(()).await;
            let connection = NewConnection {
                stream,
                service_builder: ServiceBuilder::new(),
            };

            task::spawn({
                let handle_service = handle_service.clone();
                async move {
                    if let Err(e) = handle_service.call(connection).await {
                        debug!("http connection terminated", error: e);
                    }
                }
            });
        }
    });

    Ok(())
}
