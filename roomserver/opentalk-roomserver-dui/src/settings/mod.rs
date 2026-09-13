// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::path::Path;

use anyhow::Context;
use eframe::CreationContext;
use egui::ThemePreference;
use opentalk_roomserver_common::settings::{Settings, SettingsFile};
use serde::{Deserialize, Serialize};

use crate::settings::file::{DuiSettingsFile, default};

mod file;
mod livekit;
mod message_history;
pub mod room;

pub use file::v1::DuiSettingsV1 as DuiSettings;
pub use livekit::LiveKitSettings;
pub use message_history::{HistoryEntry, MessageHistory};

const SETTINGS_KEY: &str = "settings";

pub fn save(settings: &DuiSettings, storage: &mut dyn eframe::Storage) {
    storage.set_string(
        SETTINGS_KEY,
        serde_json::to_string(&DuiSettingsFile::V1(settings.clone()))
            .expect("Settings are serializable"),
    );
}

pub fn load(
    cc: &CreationContext<'_>,
    roomserver_config: Option<&Path>,
) -> Result<DuiSettings, anyhow::Error> {
    let mut settings = if let Some(raw_settings) = cc
        .storage
        .and_then(|storage| storage.get_string(SETTINGS_KEY))
    {
        log::info!("Load settings from persistent storage");

        let mut settings = DuiSettingsFile::latest(&raw_settings)?;
        settings.mark_custom();

        log::trace!("loaded settings: {settings:#?}");
        settings
    } else {
        log::info!("No persistent storage");
        default()
    };
    cc.egui_ctx.set_theme(settings.theme);

    if let Some(config) = roomserver_config {
        log::debug!("Loading Roomserver Configuration");
        let roomserver_settings: Settings = SettingsFile::load_from_path(config)?.try_into()?;

        let host = roomserver_settings.http.address;
        let port = roomserver_settings.http.port;

        let roomserver_url = format!("http://{host}:{port}").parse()?;

        let roomserver_api_token = roomserver_settings.http.api_keys;

        log::info!("Overwrite roomserver URL and API token");
        settings.roomserver_api_key = roomserver_api_token
            .into_inner()
            .pop()
            .context("Missing api key in roomserver config")?;
        settings.roomserver_url = roomserver_url;

        settings.mark_custom();
    }
    Ok(settings)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DuiTheme {
    #[default]
    System,
    Dark,
    Light,
}

impl From<ThemePreference> for DuiTheme {
    fn from(value: ThemePreference) -> Self {
        match value {
            egui::ThemePreference::Dark => DuiTheme::Dark,
            egui::ThemePreference::Light => DuiTheme::Light,
            egui::ThemePreference::System => DuiTheme::System,
        }
    }
}

impl From<DuiTheme> for ThemePreference {
    fn from(value: DuiTheme) -> Self {
        match value {
            DuiTheme::Dark => egui::ThemePreference::Dark,
            DuiTheme::Light => egui::ThemePreference::Light,
            DuiTheme::System => egui::ThemePreference::System,
        }
    }
}

#[cfg(test)]
mod tests {
    use opentalk_service_auth::ApiKey;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize_with_missing_optional_fields() {
        let json_data = json!({
            "roomserver_url": "http://example.com"
        });

        let settings: DuiSettings =
            serde_json::from_value(json_data).expect("Deserialization failed");
        assert_eq!(settings.theme, DuiTheme::System);
        assert_eq!(settings.roomserver_url.as_str(), "http://example.com/");
        assert_eq!(settings.roomserver_api_key, ApiKey::new("roomserver", ""));
    }

    #[test]
    fn ensure_backwards_compatibility() {
        let json_data = json!({
            "roomserver_url": "http://example.com"
        });

        let settings: DuiSettings =
            serde_json::from_value(json_data).expect("Deserialization failed");
        assert_eq!(settings.theme, DuiTheme::System);
        assert_eq!(settings.roomserver_url.as_str(), "http://example.com/");
        assert_eq!(settings.roomserver_api_key, ApiKey::new("roomserver", ""));
    }
}
