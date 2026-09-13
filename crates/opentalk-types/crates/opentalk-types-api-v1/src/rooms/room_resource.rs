// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::{GuestAccess, RoomAlias, RoomId, RoomPassword},
    time::Timestamp,
    utils::ExampleData,
};

use crate::users::PublicUserProfile;

/// A Room
///
/// Contains all room information. Is only be accessible to the owner and users with
/// appropriate permissions.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(
        RoomResource::example_data()
    ))
)]
pub struct RoomResource {
    /// The ID of the room
    pub id: RoomId,

    /// An optional alias for the room that can be used in room links.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub alias: Option<RoomAlias>,

    /// The public user profile of the room's owner
    pub created_by: PublicUserProfile,

    /// The date when the room was created
    pub created_at: Timestamp,

    /// The password of the room, if any
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub password: Option<RoomPassword>,

    /// Guest access mode
    #[cfg_attr(feature = "serde", serde(default))]
    pub guest_access: GuestAccess,

    /// If waiting room is enabled
    pub waiting_room: bool,
}

impl ExampleData for RoomResource {
    fn example_data() -> Self {
        Self {
            id: RoomId::nil(),
            alias: Some(RoomAlias::example_data()),
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::unix_epoch(),
            password: Some(RoomPassword::example_data()),
            waiting_room: false,
            guest_access: GuestAccess::example_data(),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn without_optional_fields() {
        let room = RoomResource {
            id: RoomId::nil(),
            alias: None,
            created_by: PublicUserProfile::example_data(),
            created_at: Timestamp::unix_epoch(),
            password: None,
            waiting_room: false,
            guest_access: GuestAccess::example_data(),
        };
        let json = json!({
            "id": RoomId::nil(),
            "created_by": serde_json::to_value(PublicUserProfile::example_data()).unwrap(),
            "created_at": serde_json::to_value(Timestamp::unix_epoch()).unwrap(),
            "waiting_room": false,
            "guest_access": "waiting_room",
        });

        let serialized = serde_json::to_value(&room).unwrap();
        assert_eq!(serialized, json);

        let deserialized: RoomResource = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, room);
    }

    #[test]
    fn with_optional_fields() {
        let room = RoomResource::example_data();
        let json = json!({
            "id": RoomId::nil(),
            "alias": RoomAlias::example_data(),
            "created_by": serde_json::to_value(PublicUserProfile::example_data()).unwrap(),
            "created_at": serde_json::to_value(Timestamp::unix_epoch()).unwrap(),
            "password": RoomPassword::example_data(),
            "waiting_room": false,
            "guest_access": "waiting_room",
        });

        let serialized = serde_json::to_value(&room).unwrap();
        assert_eq!(serialized, json);

        let deserialized: RoomResource = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized, room);
    }
}
