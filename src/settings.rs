// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use config::{Config, ConfigError, Environment, File, FileFormat};
use openidconnect::{ClientId, ClientSecret, IssuerUrl};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
pub struct Settings {
    pub auth: AuthSettings,
    pub controller: ControllerSettings,
    pub sip: SipSettings,
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
                    format!("K3K_OBLSK_{}", env_var).yellow(),
                    format!("OPENTALK_OBLSK_{}", env_var).green().bold(),
                );
            }
        }

        Ok(values)
    }
}

impl Settings {
    pub fn load(file_name: &str) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::new(file_name, FileFormat::Toml))
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
    }
}

#[derive(Deserialize)]
pub struct AuthSettings {
    pub issuer: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(Deserialize)]
pub struct ControllerSettings {
    pub domain: String,
    #[serde(default)]
    pub insecure: bool,
}

#[derive(Deserialize)]
pub struct SipSettings {
    pub addr: String,
    pub port: u16,

    pub id: Option<String>,
    pub username: Option<String>,

    #[serde(flatten)]
    pub registrar: Option<SipRegistrarSettings>,

    pub stun_server: Option<String>,

    #[serde(default)]
    pub rtp_port_range: RtpPortRange,
}

#[derive(Clone, Deserialize)]
pub struct SipRegistrarSettings {
    pub password: Option<String>,
    pub realm: Option<String>,

    pub registrar: String,

    #[serde(default)]
    pub enforce_qop: bool,

    pub outbound_proxy: Option<String>,

    #[serde(default = "default_nat_ping_delta", deserialize_with = "duration_secs")]
    pub nat_ping_delta: Duration,
}

#[derive(Deserialize)]
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
    fn settings_env_vars_overwite_config() -> Result<(), ConfigError> {
        // Sanity check
        let settings = Settings::load("./extra/example.toml")?;

        assert_eq!(settings.controller.domain, "localhost:8000");
        assert_eq!(settings.sip.port, 5060u16);

        // Set environment variables to overwrite default config file
        let env_controller_domain = "localhost:8080".to_string();
        let env_sip_port: u16 = 5070;
        env::set_var("OPENTALK_OBLSK_CONTROLLER__DOMAIN", &env_controller_domain);
        env::set_var("OPENTALK_OBLSK_SIP__PORT", env_sip_port.to_string());

        let settings = Settings::load("./extra/example.toml")?;

        assert_eq!(settings.controller.domain, env_controller_domain);
        assert_eq!(settings.sip.port, env_sip_port);

        Ok(())
    }
}
