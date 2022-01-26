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
use crate::logging::api::WrappedLogV1;
use crate::logging::format::LogFormat;
use futures_sink::Sink;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use witchcraft_server_config::install::InstallConfig;

#[pin_project]
pub struct WrappingAppender<S> {
    #[pin]
    inner: S,
    product_name: String,
    product_version: String,
}

impl<S> WrappingAppender<S> {
    pub fn new(inner: S, config: &InstallConfig) -> Self {
        WrappingAppender {
            inner,
            product_name: config.product_name().to_string(),
            product_version: config.product_version().to_string(),
        }
    }
}

impl<S, T> Sink<T> for WrappingAppender<S>
where
    S: Sink<WrappedLogV1>,
    T: LogFormat,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let this = self.project();

        let item = WrappedLogV1::builder()
            .type_("wrapped.1")
            .payload(item.wrap())
            .entity_name(&**this.product_name)
            .entity_version(&**this.product_version)
            .build();

        this.inner.start_send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}
