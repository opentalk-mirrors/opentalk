// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use config::{Config, ConfigBuilder, Environment, File, FileFormat, builder::DefaultState};
use http::Http;
use opentalk_orchestrator_client::OrchestratorConfig;
use serde::Deserialize;
use task::Task;
use telemetry::{Metrics, Monitoring, Tracing};
use thiserror::Error;

use super::controller_settings::ControllerConfig;
use crate::settings::settings_file::internal::Internal;

pub mod conference;
pub mod defaults;
pub mod http;
pub mod internal;
pub mod reports;
pub mod reports_typst;
pub mod task;
pub mod telemetry;

#[derive(Debug, Error)]
#[error("Settings error")]
pub struct Error {
    #[from]
    source: anyhow::Error,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsFile {
    /// HTTP web server settings
    pub(crate) http: Http,

    #[serde(default)]
    pub(crate) controller: Option<ControllerConfig>,

    pub(crate) orchestrator: Option<OrchestratorConfig>,

    #[serde(default)]
    pub(crate) monitoring: Option<Monitoring>,

    #[serde(default)]
    pub(crate) metrics: Option<Metrics>,

    #[serde(default)]
    pub(crate) tracing: Option<Tracing>,

    #[serde(default)]
    pub(crate) internal: Option<Internal>,

    #[serde(default, flatten)]
    pub(crate) task: Task,
}

impl SettingsFile {
    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs
    /// for more details).
    pub fn load_from_path(file_path: &Path) -> Result<Self, Error> {
        let builder = Config::builder().add_source(File::from(file_path).format(FileFormat::Toml));
        let config = add_environment_source(builder)
            .build()
            .context("failed to build settings loader")?;

        let mut warn_unknown_key = Self::warn_unused_key;
        let ignored_deserializer = serde_ignored::Deserializer::new(config, &mut warn_unknown_key);
        let settings_file = serde_path_to_error::deserialize(ignored_deserializer)
            .context("invalid configuration")?;

        Ok(settings_file)
    }

    fn warn_unused_key(path: serde_ignored::Path) {
        // Be aware that this might get called before the logger is initialized. Don't use
        // tracing/log crates.
        use owo_colors::OwoColorize as _;
        anstream::eprintln!(
            "{}: Unknown configuration key {}",
            "WARNING".yellow().bold(),
            path.bold(),
        );
    }

    /// Creates a new Settings instance from the provided TOML file if provided
    /// or from the first available standard path otherwise.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs
    /// for more details).
    pub fn load(file_path: Option<&Path>) -> Result<SettingsFile, Error> {
        if let Some(path) = file_path {
            return Self::load_from_path(path);
        }

        let paths = Self::build_standard_search_paths();
        for path in &paths {
            if path.exists() {
                return Self::load_from_path(path);
            }
        }

        Err(anyhow!(
            "Couldn't find a configuration file. Searched: {}.",
            paths
                .iter()
                .map(|path| format!("\"{}\"", path.to_string_lossy()))
                .collect::<Vec<String>>()
                .join(", ")
        )
        .into())
    }

    fn build_standard_search_paths() -> Vec<PathBuf> {
        let mut paths = vec!["roomserver.toml".into()];

        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("opentalk/roomserver.toml"));
        }

        paths.push("/etc/opentalk/roomserver.toml".into());

        paths
    }
}

fn add_environment_source(builder: ConfigBuilder<DefaultState>) -> ConfigBuilder<DefaultState> {
    builder
        // legacy environment prefix (before v0.1)
        .add_source(
            Environment::with_prefix("OT_ROOMSERVER")
                .prefix_separator("_")
                .separator("__")
                // try_parsing, list_separator and with_list_parse_key are required
                // to parse sequences from the environment
                .try_parsing(true)
                .list_separator(",")
                .with_list_parse_key("http.api_keys")
                .with_list_parse_key("metrics.allowlist"),
        )
        // currently supported prefix (starting with v0.1)
        .add_source(
            Environment::with_prefix("OPENTALK_ROOM")
                .prefix_separator("_")
                .separator("__")
                // try_parsing, list_separator and with_list_parse_key are required
                // to parse sequences from the environment
                .try_parsing(true)
                .list_separator(",")
                .with_list_parse_key("http.api_keys")
                .with_list_parse_key("metrics.allowlist"),
        )
}

#[cfg(test)]
mod tests {
    use config::Config;

    use super::{SettingsFile, add_environment_source};

    #[test]
    fn should_parse_array_from_env() {
        temp_env::with_vars(
            [
                (
                    "OPENTALK_ROOM_METRICS__ALLOWLIST",
                    Some("172.0.0.0/9,172.128.0.0/9"),
                ),
                (
                    "OPENTALK_ROOM_HTTP__API_KEYS",
                    Some("roomserver:secret1,recorder:secret2"),
                ),
            ],
            || {
                let config = add_environment_source(
                    Config::builder()
                        .set_default("http.public_url", "http://localhost")
                        .unwrap(),
                )
                .build()
                .unwrap();

                config.try_deserialize::<SettingsFile>().unwrap();
            },
        );
    }

    #[test]
    fn should_parse_array_from_env_single() {
        temp_env::with_vars(
            [
                ("OPENTALK_ROOM_METRICS__ALLOWLIST", Some("172.0.0.0/9")),
                ("OPENTALK_ROOM_HTTP__API_KEYS", Some("roomserver:secret1")),
            ],
            || {
                let config = add_environment_source(
                    Config::builder()
                        .set_default("http.public_url", "http://localhost")
                        .unwrap(),
                )
                .build()
                .unwrap();

                config.try_deserialize::<SettingsFile>().unwrap();
            },
        );
    }
}
