// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::{GuestAccess, RoomAlias, RoomId, RoomPassword},
    utils::ExampleData,
};

use super::CallInInfo;

/// All information about a room in which an event takes place
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventRoomInfo::example_data()))
)]
pub struct EventRoomInfo {
    /// ID of the room
    pub id: RoomId,

    /// An alias that can be used in the URL to access the room instead of the [`RoomId`].
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub alias: Option<RoomAlias>,

    /// Password of the room
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,

    /// Flag to check if the room has a waiting room enabled
    pub waiting_room: bool,

    /// Guest access mode
    #[cfg_attr(feature = "serde", serde(default))]
    pub guest_access: GuestAccess,

    /// Flag to check if the room has e2e encryption enabled
    #[cfg_attr(feature = "serde", serde(default))]
    pub e2e_encryption: bool,

    /// Call-In information
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub call_in: Option<CallInInfo>,
}

impl ExampleData for EventRoomInfo {
    fn example_data() -> Self {
        Self {
            id: RoomId::example_data(),
            alias: Some(RoomAlias::example_data()),
            password: Some(RoomPassword::example_data()),
            waiting_room: false,
            e2e_encryption: false,
            call_in: Some(CallInInfo::example_data()),
            guest_access: GuestAccess::example_data(),
        }
    }
}
