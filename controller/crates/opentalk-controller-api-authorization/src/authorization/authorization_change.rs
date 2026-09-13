// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_common::{
    events::{EventId, invites::InviteRole},
    rooms::{GuestAccess, RoomId},
    users::{GroupId, UserId},
};

/// A change that can be applied to an authorization dataset
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(tag = "change", rename_all = "snake_case")
)]
pub enum AuthorizationChange {
    /// Add a user to a set of groups.
    AddUserToGroups {
        /// The id of the user that should be added to the groups.
        user: UserId,

        /// The ids of the groups to which the user should be added.
        groups: BTreeSet<GroupId>,
    },

    /// Remove a user from a set of groups.
    RemoveUserFromGroups {
        /// The id of the user that should be removed from the groups.
        user: UserId,

        /// The ids of the groups from which the user should be removed.
        groups: BTreeSet<GroupId>,
    },

    /// Create an event
    CreateEvent {
        /// The id of the event that should be crated.
        event: EventId,

        /// The id of the user that created the event.
        creator: UserId,
    },

    /// Delete an event
    DeleteEvent {
        /// The id of the event that should be deleted.
        event: EventId,
    },

    /// Create a room
    CreateRoom {
        /// The id of the room that should be crated.
        room: RoomId,

        /// The id of the user that created the room.
        creator: UserId,

        /// Whether the guest feature is enabled in the tariff of the creator.
        is_guest_feature_enabled: bool,

        /// The guest access of the room.
        guest_access: GuestAccess,

        /// Whether the room is end-to-end encrypted.
        e2e_encryption: bool,
    },

    /// Delete a room
    DeleteRoom {
        /// The id of the room that should be deleted.
        room: RoomId,
    },

    /// Create a user
    CreateUser {
        /// The id of the user that should be crated.
        user: UserId,
    },

    /// Delete a user
    DeleteUser {
        /// The id of the user that should be deleted.
        user: UserId,
    },

    /// Add a user to a set of rooms.
    AddUserToRooms {
        /// The id of the user that should be added to the rooms.
        user: UserId,

        /// The role of the user in the room.
        role: InviteRole,

        /// The ids of the rooms to which the user should be added.
        rooms: BTreeSet<RoomId>,
    },

    /// Remove a user to a set of rooms.
    RemoveUserFromRooms {
        /// The id of the user that should be added to the rooms.
        user: UserId,

        /// The ids of the rooms from which the user should be removed.
        rooms: BTreeSet<RoomId>,
    },

    /// Add a user to a set of events.
    AddUserToEvents {
        /// The id of the user that should be added to the events.
        user: UserId,

        /// The role of the user in the room.
        role: InviteRole,

        /// The ids of the events to which the user should be added.
        events: BTreeSet<EventId>,
    },

    /// Remove a user to a set of events.
    RemoveUserFromEvents {
        /// The id of the user that should be added to the events.
        user: UserId,

        /// The ids of the events from which the user should be removed.
        events: BTreeSet<EventId>,
    },

    /// Update the configuration of a room.
    UpdateRoomConfiguration {
        /// The room for which the guest access should be patched.
        room: RoomId,

        /// The new guest access for the room.
        guest_access: Option<GuestAccess>,

        /// The new end-to-end encryption setting for the room.
        e2e_encryption: Option<bool>,
    },

    /// Update rooms on changes to the owner's tariff assignment.
    UpdateUserTariffAssignment {
        /// The user who's tariff assignment has changed.
        user: UserId,

        /// `true` when the `core::guests_allowed` feature is enabled in the new tariff.
        is_guest_feature_enabled: bool,
    },
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use std::collections::BTreeSet;

    use opentalk_types_common::{
        events::{EventId, invites::InviteRole},
        rooms::{GuestAccess, RoomId},
        users::{GroupId, UserId},
    };
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::authorization::AuthorizationChange;

    #[test]
    fn add_user_to_groups() {
        let c = AuthorizationChange::AddUserToGroups {
            user: UserId::from_u128(0x11335577),
            groups: BTreeSet::from_iter([
                GroupId::from_u128(0x987654),
                GroupId::from_u128(0x997744),
            ]),
        };

        let json = json!({
            "change": "add_user_to_groups",
            "user": "00000000-0000-0000-0000-000011335577",
            "groups": [
                "00000000-0000-0000-0000-000000987654",
                "00000000-0000-0000-0000-000000997744"
            ]
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn remove_user_from_groups() {
        let c = AuthorizationChange::RemoveUserFromGroups {
            user: UserId::from_u128(0x11335577),
            groups: BTreeSet::from_iter([
                GroupId::from_u128(0x987654),
                GroupId::from_u128(0x997744),
            ]),
        };

        let json = json!({
            "change": "remove_user_from_groups",
            "user": "00000000-0000-0000-0000-000011335577",
            "groups": [
                "00000000-0000-0000-0000-000000987654",
                "00000000-0000-0000-0000-000000997744"
            ]
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn create_event() {
        let c = AuthorizationChange::CreateEvent {
            event: EventId::from_u128(0x3c9_2f84_79a4_3875),
            creator: UserId::from_u128(0x11335577),
        };

        let json = json!({
            "change": "create_event",
            "event": "00000000-0000-0000-03c9-2f8479a43875",
            "creator": "00000000-0000-0000-0000-000011335577",
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn delete_event() {
        let c = AuthorizationChange::DeleteEvent {
            event: EventId::from_u128(0x3c9_2f84_79a4_3875),
        };

        let json = json!({
            "change": "delete_event",
            "event": "00000000-0000-0000-03c9-2f8479a43875",
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn create_room() {
        let c = AuthorizationChange::CreateRoom {
            room: RoomId::from_u128(0x3c9_2f84_79a4_3875),
            creator: UserId::from_u128(0x11335577),
            is_guest_feature_enabled: true,
            guest_access: GuestAccess::DirectAccess,
            e2e_encryption: false,
        };

        let json = json!({
            "change": "create_room",
            "room": "00000000-0000-0000-03c9-2f8479a43875",
            "creator": "00000000-0000-0000-0000-000011335577",
            "is_guest_feature_enabled": true,
            "e2e_encryption": false,
            "guest_access": "direct_access",
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn delete_room() {
        let c = AuthorizationChange::DeleteRoom {
            room: RoomId::from_u128(0x3c9_2f84_79a4_3875),
        };

        let json = json!({
            "change": "delete_room",
            "room": "00000000-0000-0000-03c9-2f8479a43875",
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn create_user() {
        let c = AuthorizationChange::CreateUser {
            user: UserId::from_u128(0x11335577),
        };

        let json = json!({
            "change": "create_user",
            "user": "00000000-0000-0000-0000-000011335577",
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn delete_user() {
        let c = AuthorizationChange::DeleteUser {
            user: UserId::from_u128(0x11335577),
        };

        let json = json!({
            "change": "delete_user",
            "user": "00000000-0000-0000-0000-000011335577",
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn add_user_to_rooms() {
        let c = AuthorizationChange::AddUserToRooms {
            user: UserId::from_u128(0x11335577),
            role: InviteRole::User,
            rooms: BTreeSet::from_iter([RoomId::from_u128(0x987654), RoomId::from_u128(0x997744)]),
        };

        let json = json!({
            "change": "add_user_to_rooms",
            "user": "00000000-0000-0000-0000-000011335577",
            "role": "user",
            "rooms": [
                "00000000-0000-0000-0000-000000987654",
                "00000000-0000-0000-0000-000000997744"
            ]
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn remove_user_from_rooms() {
        let c = AuthorizationChange::RemoveUserFromRooms {
            user: UserId::from_u128(0x11335577),
            rooms: BTreeSet::from_iter([RoomId::from_u128(0x987654), RoomId::from_u128(0x997744)]),
        };

        let json = json!({
            "change": "remove_user_from_rooms",
            "user": "00000000-0000-0000-0000-000011335577",
            "rooms": [
                "00000000-0000-0000-0000-000000987654",
                "00000000-0000-0000-0000-000000997744"
            ]
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn add_user_to_events() {
        let c = AuthorizationChange::AddUserToEvents {
            user: UserId::from_u128(0x11335577),
            role: InviteRole::User,
            events: BTreeSet::from_iter([
                EventId::from_u128(0x987654),
                EventId::from_u128(0x997744),
            ]),
        };

        let json = json!({
            "change": "add_user_to_events",
            "user": "00000000-0000-0000-0000-000011335577",
            "role": "user",
            "events": [
                "00000000-0000-0000-0000-000000987654",
                "00000000-0000-0000-0000-000000997744"
            ]
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn remove_user_from_events() {
        let c = AuthorizationChange::RemoveUserFromEvents {
            user: UserId::from_u128(0x11335577),
            events: BTreeSet::from_iter([
                EventId::from_u128(0x987654),
                EventId::from_u128(0x997744),
            ]),
        };

        let json = json!({
            "change": "remove_user_from_events",
            "user": "00000000-0000-0000-0000-000011335577",
            "events": [
                "00000000-0000-0000-0000-000000987654",
                "00000000-0000-0000-0000-000000997744"
            ]
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn update_room_configuration() {
        let c = AuthorizationChange::UpdateRoomConfiguration {
            room: RoomId::from_u128(0x987654),
            guest_access: Some(GuestAccess::WaitingRoom),
            e2e_encryption: Some(false),
        };

        let json = json!({
           "change": "update_room_configuration",
           "room": "00000000-0000-0000-0000-000000987654",
           "guest_access": "waiting_room",
           "e2e_encryption": false,
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }

    #[test]
    fn update_user_tariff_assignment() {
        let c = AuthorizationChange::UpdateUserTariffAssignment {
            user: UserId::from_u128(0x11335577),
            is_guest_feature_enabled: true,
        };

        let json = json!({
           "change": "update_user_tariff_assignment",
           "user": "00000000-0000-0000-0000-000011335577",
           "is_guest_feature_enabled": true,
        });

        let serialized = serde_json::to_value(c.clone()).expect("Must be serializable");
        assert_eq!(serialized, json);

        let deserialized: AuthorizationChange =
            serde_json::from_value(json).expect("Must be deserializable");
        assert_eq!(deserialized, c);
    }
}
