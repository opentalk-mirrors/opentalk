// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{rooms::RoomPassword, roomserver::DeviceSecret, utils::ExampleData};

/// The JSON body expected when making a *POST /rooms/{room_id}/start* request
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema (example = json!(PostRoomsRoomserverStartRequestBody::example_data()))
)]
pub struct PostRoomsRoomserverStartRequestBody {
    /// The clients device secret. Used to identify the client across sessions
    pub device_secret: DeviceSecret,

    /// The invited user's password to the room
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,
}

impl ExampleData for PostRoomsRoomserverStartRequestBody {
    fn example_data() -> Self {
        Self {
            device_secret: DeviceSecret::example_data(),
            password: None,
        }
    }
}
