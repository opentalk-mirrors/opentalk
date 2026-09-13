// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::net::{IpAddr, Ipv4Addr};

use crate::settings_file;

pub const DEFAULT_MONITORING_PORT: u16 = 11411;
pub const DEFAULT_MONITORING_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

/// The monitoring service configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monitoring {
    /// The tcp port listening for monitoring requests.
    pub port: u16,

    /// The IP addrsess on which to listen for monitoring requests.
    pub addr: IpAddr,
}

impl From<settings_file::MonitoringSettings> for Monitoring {
    fn from(
        settings_file::MonitoringSettings { port, addr }: settings_file::MonitoringSettings,
    ) -> Self {
        Self {
            port: port.unwrap_or(DEFAULT_MONITORING_PORT),
            addr: addr.unwrap_or(DEFAULT_MONITORING_ADDR),
        }
    }
}
