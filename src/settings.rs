use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::{path::Path, time::Duration};

#[derive(Deserialize)]
pub struct Settings {
    pub controller: ControllerSettings,
    pub sip: SipSettings,
}

impl Settings {
    pub fn load<P: AsRef<Path>>(file_name: P) -> Result<Self, ConfigError> {
        let mut cfg = Config::new();

        cfg.merge(File::from(file_name.as_ref()))?;

        let env = Environment::with_prefix("K3K_OBLSK").separator("__");

        cfg.merge(env)?;

        cfg.try_into()
    }
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
