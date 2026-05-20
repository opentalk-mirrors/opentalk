// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{net::IpAddr, path::PathBuf};

use anyhow::{Context, Result, bail};
use config::{Config, Environment, File, FileFormat, FileSourceFile};
use itertools::Itertools;
use opentalk_client_signaling::BaseUrl;
use opentalk_orchestrator_client::OrchestratorConfig;
use opentalk_service_auth::{ApiKey, service::ApiKeys};
use owo_colors::OwoColorize;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    pub(crate) controller: ControllerSettings,
    pub(crate) http: HttpSettings,
    pub(crate) monitoring: Option<MonitoringSettings>,
    pub(crate) transcription: TranscriptionSettings,
    pub(crate) orchestrator: Option<OrchestratorConfig>,
}

impl Settings {
    pub(crate) fn load(config_arg_path: Option<&String>) -> Result<Self> {
        // FIXME
        Config::builder()
            .add_source(discover_config_file(config_arg_path)?)
            .add_source(Environment::with_prefix("OPENTALK_REC").separator("__"))
            .build()?
            .try_deserialize()
            .context("Failed to parse configuration file")
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
            path: "transcription.toml".into(),
            deprecated: false,
        },
    ];

    if let Some(dirs) = directories::BaseDirs::new() {
        paths.push(ConfigSearchPath {
            path: dirs.config_dir().join("opentalk/transcription.toml"),
            deprecated: false,
        });
    }

    paths.push(ConfigSearchPath {
        path: "/etc/opentalk/transcription.toml".into(),
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

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HttpSettings {
    #[serde(default = "default_http_address")]
    pub(crate) addr: IpAddr,
    #[serde(default = "default_http_port")]
    pub(crate) port: u16,

    pub(crate) api_keys: ApiKeys,
}

fn default_http_address() -> IpAddr {
    [0, 0, 0, 0].into()
}

fn default_http_port() -> u16 {
    11411
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MonitoringSettings {
    #[serde(default = "default_monitoring_http_address")]
    pub(crate) addr: IpAddr,
    #[serde(default = "default_monitoring_http_port")]
    pub(crate) port: u16,
}

fn default_monitoring_http_address() -> IpAddr {
    [0, 0, 0, 0].into()
}

fn default_monitoring_http_port() -> u16 {
    11412
}

impl Default for MonitoringSettings {
    fn default() -> Self {
        Self {
            addr: default_monitoring_http_address(),
            port: default_monitoring_http_port(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ControllerSettings {
    pub(crate) url: BaseUrl,
    pub(crate) api_key: ApiKey,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TranscriptionSettings {
    #[serde(default = "default_model")]
    pub(crate) model: String,
}

fn default_model() -> String {
    "models/ggml-large-v3-turbo.bin".to_string()
}

impl Default for TranscriptionSettings {
    fn default() -> Self {
        Self {
            model: default_model(),
        }
    }
}
