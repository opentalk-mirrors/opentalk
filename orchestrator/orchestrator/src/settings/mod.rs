// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use config::{Config, Environment, File, FileFormat};
use http::Http;
use serde::Deserialize;
use services::Services;

use crate::settings::{logging::Logging, monitoring::Monitoring, storage::Storage};

pub mod http;
pub mod logging;
pub mod monitoring;
pub mod services;
pub mod storage;

#[derive(Debug, thiserror::Error)]
#[error("Settings error")]
pub struct Error {
    #[from]
    source: anyhow::Error,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// Configuration for the orchestrators HTTP server
    pub http: Http,

    pub logging: Option<Logging>,

    /// Configuration for the orchestrators monitoring endpoints (ready, startup, liveness).
    pub monitoring: Option<Monitoring>,

    /// Configuration related to services that the orchestrator connects to
    pub services: Services,

    pub storage: Storage,
}

impl Settings {
    /// Creates a new Settings instance from the provided TOML file.
    /// Specific fields can be set or overwritten with environment variables (See struct level docs
    /// for more details).
    pub fn load_from_path(file_path: &Path) -> Result<Self, Error> {
        let config = Config::builder()
            .add_source(File::from(file_path).format(FileFormat::Toml))
            .add_source(
                Environment::with_prefix("OT_ORCHESTRATOR")
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true)
                    .list_separator(",")
                    .with_list_parse_key("http.api_keys")
                    .with_list_parse_key("services.keys"),
            )
            .build()
            .context("failed to build settings loader")?;

        let mut warn_unknown_key = Self::warn_unused_key;
        let ignored_deserializer = serde_ignored::Deserializer::new(config, &mut warn_unknown_key);
        let settings = serde_path_to_error::deserialize(ignored_deserializer)
            .context("invalid configuration")?;

        Ok(settings)
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
    pub fn load(file_path: Option<&Path>) -> Result<Settings, Error> {
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
        let mut paths = vec!["orchestrator.toml".into()];

        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("opentalk/orchestrator.toml"));
        }

        paths.push("/etc/opentalk/orchestrator.toml".into());

        paths
    }
}
