// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_service_auth::ApiKey;
use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    app::event_widget::EventWidgetLayout,
    settings::{
        DuiSettings, DuiTheme, LiveKitSettings, MessageHistory,
        room::{alice_client_parameters, default_room_parameters},
    },
};

/// Settings for the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DuiSettingsLegacy {
    /// Theme of the application.
    #[serde(default)]
    pub theme: DuiTheme,

    /// URL of the room server.
    pub roomserver_url: Url,

    /// API token for the room server.
    #[serde(default)]
    pub roomserver_api_token: String,

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
    pub room_parameters: Vec<(String, serde_json::Value)>,
    #[serde(default)]
    pub selected_room_parameters: usize,

    #[serde(default)]
    pub client_parameters: Vec<(String, serde_json::Value)>,
    #[serde(default)]
    pub selected_client_parameters: usize,

    #[serde(default)]
    pub delete_mode: bool,

    #[serde(default)]
    pub livekit: LiveKitSettings,
}

impl Default for DuiSettingsLegacy {
    fn default() -> Self {
        Self {
            theme: DuiTheme::default(),
            roomserver_url: Url::parse("http://localhost:11333").expect("Static URL must be valid"),
            roomserver_api_token: String::new(),
            event_widget_layout: EventWidgetLayout::new(),
            history: MessageHistory::default(),

            room_ids: [
                ("Room-1".to_string(), RoomId::from_u128(1)),
                ("Room-2".to_string(), RoomId::from_u128(2)),
                ("Room-3".to_string(), RoomId::from_u128(3)),
            ]
            .to_vec(),
            selected_room_id: 0,

            room_parameters: [(
                "Default".to_string(),
                serde_json::to_value(default_room_parameters()).expect("msg"),
            )]
            .to_vec(),
            selected_room_parameters: 0,

            client_parameters: [(
                "Alice-1".to_string(),
                serde_json::to_value(alice_client_parameters(1)).expect("msg"),
            )]
            .to_vec(),
            selected_client_parameters: 0,

            delete_mode: false,
            livekit: LiveKitSettings::default(),
        }
    }
}

impl From<DuiSettingsLegacy> for DuiSettings {
    fn from(value: DuiSettingsLegacy) -> Self {
        Self {
            is_default: false,
            theme: value.theme,
            roomserver_url: value.roomserver_url,
            roomserver_api_key: ApiKey::new("roomserver", value.roomserver_api_token),
            event_widget_layout: value.event_widget_layout,
            history: value.history,
            room_ids: value.room_ids,
            selected_room_id: value.selected_room_id,
            room_parameters: value
                .room_parameters
                .into_iter()
                .map(|(name, value)| {
                    (
                        name,
                        serde_json::to_string_pretty(&value).expect("must be serializable"),
                    )
                })
                .collect(),
            selected_room_parameters: value.selected_room_parameters,
            client_parameters: value
                .client_parameters
                .into_iter()
                .map(|(name, value)| {
                    (
                        name,
                        serde_json::to_string_pretty(&value).expect("must be serializable"),
                    )
                })
                .collect(),
            selected_client_parameters: value.selected_client_parameters,
            delete_mode: value.delete_mode,
            livekit: value.livekit,
        }
    }
}
