// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::net::{IpAddr, Ipv4Addr};

use serde::Deserialize;

/// Configuration for the ready, startup, liveness probe.
#[derive(Debug, Clone, Deserialize)]
pub struct Monitoring {
    /// Port on which the probe can be reached.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Address which is used to listen for new connections.
    #[serde(default = "default_bind_address")]
    pub address: IpAddr,
}

pub(crate) const fn default_bind_address() -> IpAddr {
    IpAddr::V4(Ipv4Addr::UNSPECIFIED)
}

const fn default_port() -> u16 {
    11221
}
