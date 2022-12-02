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

impl Settings {
    pub fn load(file_name: &str) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::new(file_name, FileFormat::Toml))
            .add_source(
                Environment::with_prefix("K3K_OBLSK")
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

    pub username: String,
    pub password: String,
    pub realm: String,

    pub registrar: String,
    pub stun_server: Option<String>,

    #[serde(default = "default_nat_ping_delta", deserialize_with = "duration_secs")]
    pub nat_ping_delta: Duration,

    #[serde(default)]
    pub enforce_qop: bool,

    #[serde(default)]
    pub rtp_port_range: RtpPortRange,
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
        env::set_var("K3K_OBLSK_CONTROLLER__DOMAIN", &env_controller_domain);
        env::set_var("K3K_OBLSK_SIP__PORT", env_sip_port.to_string());

        let settings = Settings::load("./extra/example.toml")?;

        assert_eq!(settings.controller.domain, env_controller_domain);
        assert_eq!(settings.sip.port, env_sip_port);

        Ok(())
    }
}
