// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    net::IpAddr,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::{bail, Context, Result};
use config::{Config, Environment, File, FileFormat, FileSourceFile};
use itertools::Itertools;
use opentalk_client_signaling::BaseUrl;
use opentalk_compositor::{ClockFormat, EncoderType};
use opentalk_orchestrator_client::OrchestratorConfig;
use opentalk_service_auth::{service::ApiKeys, ApiKey};
use owo_colors::OwoColorize;
use serde::{Deserialize, Deserializer};

const S3_MINIMUM_CHUNK_SIZE: usize = 5 * 1024 * 1024;
const S3_MAXIMUM_CHUNK_SIZE: usize = S3_MINIMUM_CHUNK_SIZE * 1024;

static FOUND_UNKNOWN_KEY_WITH_UNDERSCORE_PREFIX: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    pub(crate) controller: ControllerSettings,
    pub(crate) monitoring: Option<MonitoringSettings>,
    pub(crate) http: HttpSettings,
    pub(crate) orchestrator: Option<OrchestratorConfig>,
    pub(crate) recorder: Option<RecorderSettings>,
}

impl Settings {
    pub(crate) fn load(config_arg_path: Option<&String>) -> Result<Self> {
        let config = Config::builder()
            .add_source(discover_config_file(config_arg_path)?)
            .add_source(
                // deprecated double underscore. keeping it for backwards compatibility
                Environment::with_prefix("OPENTALK_REC")
                    .separator("__")
                    .try_parsing(true)
                    .list_separator(",")
                    .with_list_parse_key("http.api_keys"),
            )
            .add_source(
                // correct way to set environment variables
                Environment::with_prefix("OPENTALK_REC")
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true)
                    .list_separator(",")
                    .with_list_parse_key("http.api_keys"),
            )
            .build()
            .context("Failed to build configuration loader")?;

        let mut warn_unknown_key = Self::warn_unused_key;
        let ignored_deserializer = serde_ignored::Deserializer::new(config, &mut warn_unknown_key);
        let settings = serde_path_to_error::deserialize(ignored_deserializer)
            .context("invalid configuration")?;

        // Migration warning for the correct environment config
        if FOUND_UNKNOWN_KEY_WITH_UNDERSCORE_PREFIX.load(Ordering::Relaxed) {
            anstream::eprintln!(
                r"{}:
    Found deprecated environment variable configuration, this may result in some misleading config warnings above.
    To fix this, replace the double underscore in:
        {}{}{}
        {}{}
    with a single underscore:
        {}{}{}
        {}{}",
                "FIXME".yellow().bold(),
                "OPENTALK_REC".yellow().bold(),
                "__".red().bold(),
                "EXAMPLE__CONFIG_KEY".yellow().bold(),
                " ".repeat("OPENTALK_REC".len()),
                "^^".red().bold(),
                "OPENTALK_REC".yellow().bold(),
                "_".green().bold(),
                "EXAMPLE__CONFIG_KEY".yellow().bold(),
                " ".repeat("OPENTALK_REC".len()),
                "^".green().bold(),
            );
        }

        Ok(settings)
    }

    // the function signature is dictated by `serde_ignored::Deserializer::new`.
    #[allow(clippy::needless_pass_by_value)]
    fn warn_unused_key(path: serde_ignored::Path) {
        // Be aware that this might get called before the logger is initialized. Don't use
        // tracing/log crates.
        use owo_colors::OwoColorize as _;

        // When an unused key starts with an underscore, it is a strong indicator that the deprecated double underscore
        // prefix has been used for the configuration with environment variables.
        if !FOUND_UNKNOWN_KEY_WITH_UNDERSCORE_PREFIX.load(Ordering::Relaxed)
            && path.to_string().starts_with('_')
        {
            FOUND_UNKNOWN_KEY_WITH_UNDERSCORE_PREFIX.store(true, Ordering::Relaxed);
        }

        anstream::eprintln!(
            "{}: Unknown configuration key {}",
            "WARNING".yellow().bold(),
            path.bold(),
        );
    }

    #[must_use]
    pub(crate) fn encoder_type(&self) -> EncoderType {
        self.recorder
            .as_ref()
            .and_then(|settings| settings.hardware_acceleration.as_ref())
            .map_or(
                EncoderType::CPU,
                |hardware_acceleration| match hardware_acceleration {
                    HardwareAcceleration::Intel(_) => EncoderType::VAAPI,
                },
            )
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
            path: "recorder.toml".into(),
            deprecated: false,
        },
    ];

    if let Some(dirs) = directories::BaseDirs::new() {
        paths.push(ConfigSearchPath {
            path: dirs.config_dir().join("opentalk/recorder.toml"),
            deprecated: false,
        });
    }

    paths.push(ConfigSearchPath {
        path: "/etc/opentalk/recorder.toml".into(),
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

    bail!("Failed to find a configuration file, searched: {searched_paths}");
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

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HttpSettings {
    #[serde(default = "default_http_port")]
    pub(crate) port: u16,
    #[serde(default = "default_http_address")]
    pub(crate) addr: IpAddr,

    pub(crate) api_keys: ApiKeys,
}

fn default_http_port() -> u16 {
    11511
}

fn default_http_address() -> IpAddr {
    [0, 0, 0, 0].into()
}

#[derive(Debug, Deserialize)]
pub(crate) struct ControllerSettings {
    pub(crate) url: BaseUrl,
    pub(crate) api_key: ApiKey,
    #[serde(deserialize_with = "clamp_chunk_size", default = "default_chunk_size")]
    pub(crate) upload_chunk_size: usize,
}

fn default_chunk_size() -> usize {
    S3_MINIMUM_CHUNK_SIZE
}

fn clamp_chunk_size<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = usize::deserialize(deserializer)?;
    let clamped = value.clamp(S3_MINIMUM_CHUNK_SIZE, S3_MAXIMUM_CHUNK_SIZE);

    if value != clamped {
        log::warn!(
            "Chunk size is {value}, expected chunk size to be between {S3_MINIMUM_CHUNK_SIZE} and {S3_MAXIMUM_CHUNK_SIZE}"
        );
    }

    Ok(clamped)
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MonitoringSettings {
    #[serde(default = "default_monitoring_port")]
    pub(crate) port: u16,
    #[serde(default = "default_monitoring_address")]
    pub(crate) addr: IpAddr,
}

fn default_monitoring_port() -> u16 {
    11411
}

fn default_monitoring_address() -> IpAddr {
    [0, 0, 0, 0].into()
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub(crate) struct RecorderSettings {
    pub(crate) clock_format: ClockFormat,
    pub(crate) display: bool,
    // Sets the default value when max_load is not present in the config.toml to
    // the return value of the function `default_max_load`
    #[serde(default = "default_max_load")]
    pub(crate) max_load: u8,
    pub(crate) hardware_acceleration: Option<HardwareAcceleration>,
}

#[must_use]
pub(crate) const fn default_max_load() -> u8 {
    80
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "manufacturer")]
#[serde(rename_all = "lowercase")]
pub(crate) enum HardwareAcceleration {
    Intel(HardwareAccelerationIntel),
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct HardwareAccelerationIntel {
    pub(crate) device: Option<String>,
}
