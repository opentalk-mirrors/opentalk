// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::net::IpAddr;

use serde::Deserialize;

#[derive(Default, Debug, Clone, Deserialize)]
pub(crate) struct Metrics {
    #[serde(default = "default_metrics_port")]
    pub port: u16,
    #[serde(default)]
    pub allowlist: Vec<cidr::IpInet>,
}

const fn default_metrics_port() -> u16 {
    11412
}

/// Configuration for the ready, startup, liveness probe.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Monitoring {
    /// Port on which the probe can be reached.
    #[serde(default = "default_monitor_port")]
    pub port: u16,

    /// Address which is used to listen for new connections.
    #[serde(default = "default_monitor_addr")]
    pub addr: IpAddr,
}

const fn default_monitor_port() -> u16 {
    11411
}

const fn default_monitor_addr() -> IpAddr {
    IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
}

/// Tracing configuration shared across the OpenTalk services.
pub(crate) use opentalk_tracing::TracingSettings as Tracing;
