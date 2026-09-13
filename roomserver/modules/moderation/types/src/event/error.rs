// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use opentalk_roomserver_types_livekit::MicrophoneRestrictionErrorKind;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Error from the `moderation` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ModerationError {
    /// Cannot change the display name of registered users
    CannotChangeNameOfRegisteredUsers,
    /// Invalid display name
    InvalidDisplayName,
    /// Insufficient permissions to perform a command
    InsufficientPermissions,
    /// The requested participant is not connected
    UnknownParticipant,
    /// The participant is not known.
    ///
    /// The participant might have disconnected before the command was executed.
    UnknownParticipants {
        /// A list of participants that are currently not part of the meeting.
        participants: BTreeSet<ParticipantId>,
    },
    /// The participant is already banned
    AlreadyBanned,
    /// The participant is already unbanned
    AlreadyUnbanned,
    /// Can't ban the room owner
    CannotBanRoomOwner,
    /// Can't ban guests
    CannotBanGuests,
    /// Cannot ban oneself
    CannotBanSelf,
    /// Cannot change the role of the room owner
    CannotChangeRoomOwnerRole,
    /// The participant already has the role assigned
    RoleAlreadyAssigned,
    /// Guests and call-in users can not become moderators
    UserCannotBeModerator,
    /// The participant is not in the waiting room
    NotWaiting,
    /// The participant cannot enter the room because they were not accepted by a moderator yet.
    NotAccepted,
    /// Cannot send the room owner to the waiting room
    CannotSendRoomOwnerToWaitingRoom,
    /// The room owner cannot be kicked
    CannotKickRoomOwner,
    /// An internal error occurred
    Internal,
    /// The received command cannot be executed since there is already a conflicting ongoing task.
    ConflictingTask,
    /// The livekit server is not available
    LivekitUnavailable,
}

impl From<MicrophoneRestrictionErrorKind> for ModerationError {
    fn from(err: MicrophoneRestrictionErrorKind) -> Self {
        match err {
            MicrophoneRestrictionErrorKind::ConflictingTask => ModerationError::ConflictingTask,
            MicrophoneRestrictionErrorKind::LivekitUnavailable => {
                ModerationError::LivekitUnavailable
            }
        }
    }
}

impl ModuleError for ModerationError {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use serde_json::json;

    use super::ModerationError;

    #[test]
    fn serialize_cannot_change_name_of_registered_users() {
        let error = ModerationError::CannotChangeNameOfRegisteredUsers;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "cannot_change_name_of_registered_users"
        }
        "#);
    }

    #[test]
    fn deserialize_cannot_change_name_of_registered_users() {
        let json = json!({
           "error": "cannot_change_name_of_registered_users",
        });

        let expected = ModerationError::CannotChangeNameOfRegisteredUsers;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_invalid_display_name() {
        let error = ModerationError::InvalidDisplayName;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "invalid_display_name"
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_display_name() {
        let json = json!({
           "error": "invalid_display_name",
        });

        let expected = ModerationError::InvalidDisplayName;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_insufficient_permissions() {
        let error = ModerationError::InsufficientPermissions;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_insufficient_permissions() {
        let json = json!({
           "error": "insufficient_permissions",
        });

        let expected = ModerationError::InsufficientPermissions;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_unknown_participant() {
        let error = ModerationError::UnknownParticipant;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "unknown_participant"
        }
        "#);
    }

    #[test]
    fn deserialize_unknown_participant() {
        let json = json!({
           "error": "unknown_participant",
        });

        let expected = ModerationError::UnknownParticipant;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_unknown_participants() {
        let error = ModerationError::UnknownParticipants {
            participants: BTreeSet::from([ParticipantId::nil()]),
        };

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "unknown_participants",
          "participants": [
            "00000000-0000-0000-0000-000000000000"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_unknown_participants() {
        let json = json!({
           "error": "unknown_participants",
           "participants": ["00000000-0000-0000-0000-000000000000"],
        });

        let expected = ModerationError::UnknownParticipants {
            participants: BTreeSet::from([ParticipantId::nil()]),
        };
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_already_banned() {
        let error = ModerationError::AlreadyBanned;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "already_banned"
        }
        "#);
    }

    #[test]
    fn deserialize_already_banned() {
        let json = json!({
           "error": "already_banned",
        });

        let expected = ModerationError::AlreadyBanned;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_already_unbanned() {
        let error = ModerationError::AlreadyUnbanned;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "already_unbanned"
        }
        "#);
    }

    #[test]
    fn deserialize_already_unbanned() {
        let json = json!({
           "error": "already_unbanned",
        });

        let expected = ModerationError::AlreadyUnbanned;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_cannot_ban_room_owner() {
        let error = ModerationError::CannotBanRoomOwner;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "cannot_ban_room_owner"
        }
        "#);
    }

    #[test]
    fn deserialize_cannot_ban_room_owner() {
        let json = json!({
           "error": "cannot_ban_room_owner",
        });

        let expected = ModerationError::CannotBanRoomOwner;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_cannot_ban_guests() {
        let error = ModerationError::CannotBanGuests;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "cannot_ban_guests"
        }
        "#);
    }

    #[test]
    fn deserialize_cannot_ban_guests() {
        let json = json!({
           "error": "cannot_ban_guests",
        });

        let expected = ModerationError::CannotBanGuests;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_cannot_ban_self() {
        let error = ModerationError::CannotBanSelf;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "cannot_ban_self"
        }
        "#);
    }

    #[test]
    fn deserialize_cannot_ban_self() {
        let json = json!({
           "error": "cannot_ban_self",
        });

        let expected = ModerationError::CannotBanSelf;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_cannot_change_room_owner_role() {
        let error = ModerationError::CannotChangeRoomOwnerRole;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "cannot_change_room_owner_role"
        }
        "#);
    }

    #[test]
    fn deserialize_cannot_change_room_owner_role() {
        let json = json!({
           "error": "cannot_change_room_owner_role",
        });

        let expected = ModerationError::CannotChangeRoomOwnerRole;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_role_already_assigned() {
        let error = ModerationError::RoleAlreadyAssigned;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "role_already_assigned"
        }
        "#);
    }

    #[test]
    fn deserialize_role_already_assigned() {
        let json = json!({
           "error": "role_already_assigned",
        });

        let expected = ModerationError::RoleAlreadyAssigned;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_user_cannot_be_moderator() {
        let error = ModerationError::UserCannotBeModerator;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "user_cannot_be_moderator"
        }
        "#);
    }

    #[test]
    fn deserialize_user_cannot_be_moderator() {
        let json = json!({
           "error": "user_cannot_be_moderator",
        });

        let expected = ModerationError::UserCannotBeModerator;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_not_waiting() {
        let error = ModerationError::NotWaiting;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "not_waiting"
        }
        "#);
    }

    #[test]
    fn deserialize_not_waiting() {
        let json = json!({
           "error": "not_waiting",
        });

        let expected = ModerationError::NotWaiting;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_not_accepted() {
        let error = ModerationError::NotAccepted;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "not_accepted"
        }
        "#);
    }

    #[test]
    fn deserialize_not_accepted() {
        let json = json!({
           "error": "not_accepted",
        });

        let expected = ModerationError::NotAccepted;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_cannot_send_room_owner_to_waiting_room() {
        let error = ModerationError::CannotSendRoomOwnerToWaitingRoom;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "cannot_send_room_owner_to_waiting_room"
        }
        "#);
    }

    #[test]
    fn deserialize_cannot_send_room_owner_to_waiting_room() {
        let json = json!({
           "error": "cannot_send_room_owner_to_waiting_room",
        });

        let expected = ModerationError::CannotSendRoomOwnerToWaitingRoom;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_cannot_kick_room_owner() {
        let error = ModerationError::CannotKickRoomOwner;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "cannot_kick_room_owner"
        }
        "#);
    }

    #[test]
    fn deserialize_cannot_kick_room_owner() {
        let json = json!({
           "error": "cannot_kick_room_owner",
        });

        let expected = ModerationError::CannotKickRoomOwner;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_internal() {
        let error = ModerationError::Internal;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "internal"
        }
        "#);
    }

    #[test]
    fn deserialize_internal() {
        let json = json!({
           "error": "internal",
        });

        let expected = ModerationError::Internal;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_conflicting_task() {
        let error = ModerationError::ConflictingTask;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "conflicting_task"
        }
        "#);
    }

    #[test]
    fn deserialize_conflicting_task() {
        let json = json!({
           "error": "conflicting_task",
        });

        let expected = ModerationError::ConflictingTask;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_livekit_unavailable() {
        let error = ModerationError::LivekitUnavailable;

        assert_snapshot!(serde_json::to_string_pretty(&error).unwrap(), @r#"
        {
          "error": "livekit_unavailable"
        }
        "#);
    }

    #[test]
    fn deserialize_livekit_unavailable() {
        let json = json!({
           "error": "livekit_unavailable",
        });

        let expected = ModerationError::LivekitUnavailable;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }
}
