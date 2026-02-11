// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result, bail};
use config::{Config, Environment, File, FileFormat, FileSourceFile};
use itertools::Itertools;
use openidconnect::{ClientId, ClientSecret, IssuerUrl};
use opentalk_compositor::ClockFormat;
use owo_colors::OwoColorize as _;
use serde::Deserialize;
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    time::Duration,
};

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub clock_format: ClockFormat,
    #[serde(default)]
    pub language: Language,

    pub auth: AuthSettings,
    pub monitoring: Option<MonitoringSettings>,
    pub controller: ControllerSettings,
    pub sip: SipSettings,
}

#[derive(Default, Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    EN,
    #[default]
    DE,
}

#[derive(Debug, Clone)]
struct WarningSource<T: Clone>(T);

impl<T> config::Source for WarningSource<T>
where
    T: config::Source + Send + Sync + Clone + 'static,
{
    fn clone_into_box(&self) -> Box<dyn config::Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
        let values = self.0.collect()?;
        if !values.is_empty() {
            use owo_colors::OwoColorize as _;

            anstream::eprintln!(
                "{}: The following environment variables have been deprecated and \
                will not work in a future release. Please change them as suggested below:",
                "DEPRECATION WARNING".yellow().bold(),
            );

            for key in values.keys() {
                let env_var = key.replace('.', "__").to_uppercase();
                anstream::eprintln!(
                    "{}: rename environment variable {} to {}",
                    "DEPRECATION WARNING".yellow().bold(),
                    format!("K3K_OBLSK_{env_var}").yellow(),
                    format!("OPENTALK_OBLSK_{env_var}").green().bold(),
                );
            }
        }

        Ok(values)
    }
}

impl Settings {
    pub fn load(config_arg_path: Option<&String>) -> Result<Self> {
        Config::builder()
            .add_source(discover_config_file(config_arg_path)?)
            .add_source(WarningSource(
                Environment::with_prefix("K3K_OBLSK")
                    .prefix_separator("_")
                    .separator("__"),
            ))
            .add_source(
                Environment::with_prefix("OPENTALK_OBLSK")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?
            .try_deserialize()
            .context("Failed to parse configuration")
    }
}

fn discover_config_file(
    config_arg_path: Option<&String>,
) -> Result<File<FileSourceFile, FileFormat>> {
    if let Some(path) = config_arg_path {
        return Ok(File::new(path, FileFormat::Toml));
    }

    let mut paths = vec![
        ConfigSearchPath {
            path: "config.toml".into(),
            deprecated: true,
        },
        ConfigSearchPath {
            path: "obelisk.toml".into(),
            deprecated: false,
        },
    ];

    if let Some(dirs) = directories::BaseDirs::new() {
        paths.push(ConfigSearchPath {
            path: dirs.config_dir().join("opentalk/obelisk.toml"),
            deprecated: false,
        });
    }

    paths.push(ConfigSearchPath {
        path: "/etc/opentalk/obelisk.toml".into(),
        deprecated: false,
    });

    for ConfigSearchPath { path, deprecated } in &paths {
        if !path.exists() {
            continue;
        }

        if *deprecated {
            let supported_paths = paths
                .iter()
                .filter_map(ConfigSearchPath::display_non_deprecated)
                .join(", ");

            anstream::eprintln!(
                "{}: You're using the deprecated configuration path \"{}\", please use one of these instead: {}.",
                "DEPRECATION WARNING".yellow().bold(),
                path.to_string_lossy(),
                supported_paths
            );
        }

        return Ok(File::from(path.as_path()).format(FileFormat::Toml));
    }

    let searched_paths = paths.iter().map(|path| path.path.display()).join(", ");

    bail!("Failed to find a configuration file, searched: {searched_paths}",);
}

struct ConfigSearchPath {
    path: PathBuf,
    deprecated: bool,
}

impl ConfigSearchPath {
    fn display_non_deprecated(&self) -> Option<String> {
        if self.deprecated {
            return None;
        }
        Some(format!("\"{}\"", self.path.display()))
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthSettings {
    pub issuer: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(Debug, Deserialize)]
pub struct ControllerSettings {
    pub domain: String,
    #[serde(default)]
    pub insecure: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MonitoringSettings {
    #[serde(default = "default_monitoring_port")]
    pub(crate) port: u16,
    #[serde(default = "default_monitoring_addr")]
    pub(crate) addr: IpAddr,
}

fn default_monitoring_port() -> u16 {
    11411
}

fn default_monitoring_addr() -> IpAddr {
    [0, 0, 0, 0].into()
}

#[derive(Debug, Deserialize)]
pub struct SipSettings {
    pub addr: String,
    pub port: u16,

    #[serde(default)]
    pub transport: SipTransport,

    pub inbound_addr: Option<SocketAddr>,
    pub inbound_media_ip: Option<IpAddr>,

    pub id: Option<String>,
    pub contact: Option<String>,

    pub username: Option<String>,

    #[serde(flatten)]
    pub registrar: Option<SipRegistrarSettings>,

    pub stun_server: Option<String>,

    #[serde(default)]
    pub rtp_port_range: RtpPortRange,

    #[serde(default)]
    pub offer_srtp: bool,

    /// Maximum video bitrate in bits/s
    #[serde(default = "default_max_video_bitrate")]
    pub max_video_bitrate: u32,

    #[serde(default)]
    pub encode_video_at_half_bitrate: bool,

    #[serde(default)]
    pub display_name_source: DisplayNameSource,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SipTransport {
    Udp,
    Tcp,
    #[default]
    UdpTcp,
}

impl SipTransport {
    pub fn listen_udp(&self) -> bool {
        matches!(self, Self::Udp | Self::UdpTcp)
    }

    pub fn listen_tcp(&self) -> bool {
        matches!(self, Self::Tcp | Self::UdpTcp)
    }
}

#[derive(Debug, Default, Deserialize)]
pub enum DisplayNameSource {
    #[default]
    Contact,
    From,
}

fn default_max_video_bitrate() -> u32 {
    6_000_000
}

#[derive(Debug, Clone, Deserialize)]
pub struct SipRegistrarSettings {
    pub password: Option<String>,
    pub realm: Option<String>,

    pub registrar: String,
    pub outbound_proxy: Option<String>,

    #[serde(default)]
    pub enforce_qop: bool,

    #[serde(default = "default_nat_ping_delta", deserialize_with = "duration_secs")]
    pub nat_ping_delta: Duration,
}

#[derive(Debug, Deserialize)]
pub struct RtpPortRange {
    pub start: u16,
    pub end: u16,
}

impl Default for RtpPortRange {
    fn default() -> Self {
        Self {
            start: 40000,
            end: 49999,
        }
    }
}

fn default_nat_ping_delta() -> Duration {
    Duration::from_secs(30)
}

pub fn duration_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Duration::from_secs(<u64>::deserialize(deserializer)?))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;

    #[test]
    fn settings_env_vars_overwite_config() -> anyhow::Result<()> {
        // Sanity check
        let settings = Settings::load(Some(&"./extra/example.toml".to_string()))?;

        assert_eq!(settings.controller.domain, "localhost:8000");
        assert_eq!(settings.sip.port, 5060u16);

        // Set environment variables to overwrite default config file
        let env_controller_domain = "localhost:8080".to_string();
        let env_sip_port: u16 = 5070;
        unsafe {
            env::set_var("OPENTALK_OBLSK_CONTROLLER__DOMAIN", &env_controller_domain);
            env::set_var("OPENTALK_OBLSK_SIP__PORT", env_sip_port.to_string());
        }
        let settings = Settings::load(Some(&"./extra/example.toml".to_string()))?;

        assert_eq!(settings.controller.domain, env_controller_domain);
        assert_eq!(settings.sip.port, env_sip_port);

        Ok(())
    }
}
