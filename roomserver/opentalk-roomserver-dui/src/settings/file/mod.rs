// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

pub mod legacy;
pub mod v1;

use opentalk_service_auth::ApiKey;
use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};
use url::Url;

pub use crate::settings::file::v1::DuiSettingsV1 as DuiSettings;
use crate::{
    app::event_widget::EventWidgetLayout,
    settings::{
        DuiTheme, LiveKitSettings, MessageHistory,
        file::legacy::DuiSettingsLegacy,
        room::{
            alice_client_parameters, bob_client_parameters, default_room_parameters,
            gustav_client_parameters,
        },
    },
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum DuiSettingsFile {
    V1(crate::settings::file::v1::DuiSettingsV1),
}

pub fn default() -> DuiSettings {
    DuiSettings {
        theme: DuiTheme::default(),
        roomserver_url: Url::parse("http://localhost:11333").expect("Static URL must be valid"),
        roomserver_api_key: ApiKey::new("roomserver", ""),
        event_widget_layout: EventWidgetLayout::new(),
        is_default: true,
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
            serde_json::to_string_pretty(&default_room_parameters())
                .expect("ClientParameter must be serializable"),
        )]
        .to_vec(),
        selected_room_parameters: 0,

        client_parameters: [
            (
                "Alice-1".to_string(),
                serde_json::to_string_pretty(&alice_client_parameters(1))
                    .expect("ClientParameter must be serializable"),
            ),
            (
                "Alice-2".to_string(),
                serde_json::to_string_pretty(&alice_client_parameters(2))
                    .expect("ClientParameter must be serializable"),
            ),
            (
                "Bob-1".to_string(),
                serde_json::to_string_pretty(&bob_client_parameters(1))
                    .expect("ClientParameter must be serializable"),
            ),
            (
                "Bob-2".to_string(),
                serde_json::to_string_pretty(&bob_client_parameters(2))
                    .expect("ClientParameter must be serializable"),
            ),
            (
                "Gustav-1".to_string(),
                serde_json::to_string_pretty(&gustav_client_parameters(1))
                    .expect("ClientParameter must be serializable"),
            ),
        ]
        .to_vec(),
        selected_client_parameters: 0,

        delete_mode: false,
        livekit: LiveKitSettings::default(),
    }
}

impl DuiSettingsFile {
    pub fn latest(data: &str) -> Result<DuiSettings, anyhow::Error> {
        let settings = serde_json::from_str::<DuiSettingsFile>(data);

        match settings {
            Ok(DuiSettingsFile::V1(v1)) => Ok(v1),
            Err(_) => {
                let settings = serde_json::from_str::<DuiSettingsLegacy>(data)?;
                Ok(settings.into())
            }
        }
    }
}
