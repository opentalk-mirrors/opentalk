// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::IpAddr;

use anyhow::Context;
use url::Url;

use crate::settings::settings_file;

#[derive(Debug, Clone)]
pub struct Metrics {
    pub port: u16,
    pub allowlist: Vec<cidr::IpInet>,
}

impl From<settings_file::telemetry::Metrics> for Metrics {
    fn from(value: settings_file::telemetry::Metrics) -> Self {
        Self {
            port: value.port,
            allowlist: value.allowlist,
        }
    }
}

/// Configuration for the ready, startup, liveness probe.
#[derive(Debug, Clone)]
pub struct Monitoring {
    /// Port on which the probe can be reached.
    pub port: u16,

    /// Address which is used to listen for new connections.
    pub addr: IpAddr,
}

impl Monitoring {
    pub fn url(&self) -> anyhow::Result<Url> {
        format!("http://{}:{}", self.addr, self.port)
            .parse()
            .context("Failed to build endpoint from monitoring settings")
    }
}

impl From<settings_file::telemetry::Monitoring> for Monitoring {
    fn from(value: settings_file::telemetry::Monitoring) -> Self {
        Self {
            port: value.port,
            addr: value.addr,
        }
    }
}

/// Configure a logging target.
#[derive(Debug, Clone)]
pub struct Tracing {
    default_directives: Option<Vec<String>>,

    pub otlp_tracing_endpoint: String,

    pub service_name: Option<String>,

    pub service_namespace: Option<String>,

    pub service_instance_id: Option<String>,
}

impl Tracing {
    pub fn log_filter(&self) -> Option<String> {
        self.default_directives
            .as_ref()
            .map(|filter| filter.join(","))
    }
}

impl From<settings_file::telemetry::Tracing> for Tracing {
    fn from(value: settings_file::telemetry::Tracing) -> Self {
        Self {
            default_directives: value.default_directives,
            otlp_tracing_endpoint: value.otlp_tracing_endpoint,
            service_name: value.service_name,
            service_namespace: value.service_namespace,
            service_instance_id: value.service_instance_id,
        }
    }
}
