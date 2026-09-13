// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_service_auth::ApiKey;
use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    app::event_widget::EventWidgetLayout,
    settings::{DuiTheme, LiveKitSettings, MessageHistory},
};

/// Settings for the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DuiSettingsV1 {
    /// True if no settings where stored before.
    ///
    /// In case no config file or persistent settings where found, the default settings will be
    /// used.
    #[serde(skip)]
    pub is_default: bool,

    /// Theme of the application.
    #[serde(default)]
    pub theme: DuiTheme,

    /// URL of the room server.
    pub roomserver_url: Url,

    /// API token for the room server.
    #[serde(default = "empty_api_key")]
    pub roomserver_api_key: ApiKey,

    /// Layout of the event widget.
    #[serde(default)]
    pub event_widget_layout: EventWidgetLayout,

    /// Message history
    ///
    /// Every new message that is sent will be recorded here.
    #[serde(default)]
    pub history: MessageHistory,

    #[serde(default)]
    pub room_ids: Vec<(String, RoomId)>,
    #[serde(default)]
    pub selected_room_id: usize,

    #[serde(default)]
    pub room_parameters: Vec<(String, String)>,
    #[serde(default)]
    pub selected_room_parameters: usize,

    #[serde(default)]
    pub client_parameters: Vec<(String, String)>,
    #[serde(default)]
    pub selected_client_parameters: usize,

    #[serde(default)]
    pub delete_mode: bool,

    #[serde(default)]
    pub livekit: LiveKitSettings,
}

fn empty_api_key() -> ApiKey {
    ApiKey::new("roomserver", "")
}

impl DuiSettingsV1 {
    /// True if no settings where stored before.
    ///
    /// In case no config file or persistent settings where found, the default settings will be
    /// used.
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn mark_custom(&mut self) {
        self.is_default = false;
    }
}
