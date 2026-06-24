// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeSet, HashSet};

use opentalk_roomserver_types::{
    client_parameters::Role, kick_reason::KickReason, room_parameters::WaitingRoom,
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    event::{BannedParticipantInfo, ModerationError},
    state::ModeratorJoinInfo,
};

/// Events sent out by the `moderation` module
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ModerationEvent {
    /// Sent to a participant when they are kicked from a meeting
    Kicked {
        /// The action that kicked the participant.
        reason: KickReason,
    },

    /// Sent to a participant when they are banned from a meeting
    Banned,

    /// Sent to all moderators when a participant gets banned
    ParticipantBanned(BannedParticipantInfo),

    /// Sent to all moderators when a participant is unbanned
    ParticipantUnbanned { participant_id: ParticipantId },

    /// A participants role has been updated
    RoleUpdated {
        /// The affected participant
        participant_id: ParticipantId,
        /// The participants new role
        new_role: Role,
        /// The moderator data of the room. Sent only to the participant whose role was updated to
        /// moderator.
        #[serde(skip_serializing_if = "Option::is_none")]
        moderator_data: Option<ModeratorJoinInfo>,
    },

    /// Sent out when debriefing of a session started
    DebriefingStarted {
        /// The moderator who started the debriefing
        issued_by: ParticipantId,
    },

    /// Sent when the waiting room state changed
    WaitingRoomUpdated {
        /// The new state of the waiting room
        new_state: WaitingRoom,
    },

    /// Sent to a participant that is moved to the waiting room
    SentToWaitingRoom,

    /// Sent to a participant when they are accepted by the moderator from the waiting room
    Accepted,

    /// Sent to a participant in the lobby when their permission to enter the conference changed
    EntryPermissionChanged {
        /// Whether the participant may now enter the conference
        can_enter: bool,
    },

    /// Sent to moderators when a participant was accepted
    ParticipantAccepted { participant_id: ParticipantId },

    /// Sent to all participants when a participants display name gets changed
    DisplayNameChanged {
        /// The participant that got their display name changed
        target: ParticipantId,
        /// The issuer of the display name change
        issued_by: ParticipantId,
        /// The old display name
        old_name: DisplayName,
        /// The new display name
        new_name: DisplayName,
    },

    /// Sent to a connection in the lobby when the participants display name changes in the
    /// conference
    DisplayNameAssigned {
        /// The new name. [`None`] when all other connections left the conference and the
        /// participant is now free to choose a display name.
        new_name: Option<DisplayName>,
    },

    /// The moderator enabled the display name restriction state. Only participants listed in
    /// `unrestricted_participants` and moderators are allowed to change their display name.
    DisplayNameChangeRestrictionsEnabled {
        /// Participants that are still allowed to change their display name
        unrestricted_participants: HashSet<ParticipantId>,
    },

    /// The moderator disabled the display name restriction state.
    /// All participants are allowed to change their own display name again.
    DisplayNameChangeRestrictionsDisabled,

    /// The recipient was muted by a moderator
    Muted {
        /// The moderator that muted the participant
        moderator: ParticipantId,
    },

    /// The moderator enabled the microphone-restriction-state. Only participants listed in
    /// `unrestricted_participants` are able to unmute themselves.
    MicrophoneRestrictionsEnabled {
        /// Participants that are still allowed to unmute
        unrestricted_participants: BTreeSet<ParticipantId>,
    },

    /// The moderator disabled the microphone-restriction-state.
    /// Participants are allowed to unmute themselves again.
    MicrophoneRestrictionsDisabled,

    /// An error happened when executing a `moderation` command
    Error(ModerationError),
}

impl From<ModerationError> for ModerationEvent {
    fn from(value: ModerationError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_types::client_parameters::Role;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_kicked() {
        let cmd = ModerationEvent::Kicked {
            reason: KickReason::Kicked,
        };

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "message": "kicked",
          "reason": "kicked"
        }
        "#);
    }

    #[test]
    fn deserialize_kicked() {
        let json = json!({
           "message": "kicked",
           "reason": "kicked"
        });

        let expected = ModerationEvent::Kicked {
            reason: KickReason::Kicked,
        };
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_role_updated_moderator() {
        let cmd = ModerationEvent::RoleUpdated {
            participant_id: ParticipantId::nil(),
            new_role: Role::Moderator,
            moderator_data: None,
        };

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "message": "role_updated",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "new_role": "moderator"
        }
        "#);
    }

    #[test]
    fn deserialize_role_updated_moderator() {
        let json = json!
        ({
          "message": "role_updated",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "new_role": "moderator"
        });

        let expected = ModerationEvent::RoleUpdated {
            participant_id: ParticipantId::nil(),
            new_role: Role::Moderator,
            moderator_data: None,
        };
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_role_updated_user() {
        let cmd = ModerationEvent::RoleUpdated {
            participant_id: ParticipantId::nil(),
            new_role: Role::User,
            moderator_data: None,
        };

        assert_snapshot!(serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "message": "role_updated",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "new_role": "user"
        }
        "#);
    }

    #[test]
    fn deserialize_role_updated_user() {
        let json = json!
        ({
          "message": "role_updated",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "new_role": "user"
        });

        let expected = ModerationEvent::RoleUpdated {
            participant_id: ParticipantId::nil(),
            new_role: Role::User,
            moderator_data: None,
        };
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_debriefing_started() {
        let event = ModerationEvent::DebriefingStarted {
            issued_by: ParticipantId::nil(),
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "debriefing_started",
          "issued_by": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_debriefing_started() {
        let json = json!({
            "message": "debriefing_started",
            "issued_by": "00000000-0000-0000-0000-000000000000"
        });

        let expected = ModerationEvent::DebriefingStarted {
            issued_by: ParticipantId::nil(),
        };
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_waiting_room_updated() {
        let event = ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::Disabled,
        };

        assert_snapshot!(serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "waiting_room_updated",
          "new_state": "disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_waiting_room_updated() {
        let json = json!({
           "message": "waiting_room_updated",
           "new_state": "disabled",
        });

        let expected = ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::Disabled,
        };
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_sent_to_waiting_room() {
        let event = ModerationEvent::SentToWaitingRoom;

        assert_snapshot!(&serde_json::to_string_pretty(&event).unwrap(), @r#"
        {
          "message": "sent_to_waiting_room"
        }
        "#);
    }

    #[test]
    fn deserialize_sent_to_waiting_room() {
        let json = json!({
           "message": "sent_to_waiting_room"
        });

        let expected = ModerationEvent::SentToWaitingRoom;
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_accepted() {
        let produced = serde_json::to_string_pretty(&ModerationEvent::Accepted).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "accepted"
        }
        "#);
    }

    #[test]
    fn deserialize_accepted() {
        let produced = serde_json::from_value(json!({
            "message": "accepted"
        }))
        .unwrap();
        let expected = ModerationEvent::Accepted;

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_muted() {
        let event = ModerationEvent::Muted {
            moderator: ParticipantId::nil(),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "muted",
          "moderator": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_muted() {
        let produced = serde_json::from_value(json!({
            "message": "muted",
            "moderator": "00000000-0000-0000-0000-000000000000"
        }))
        .unwrap();
        let expected = ModerationEvent::Muted {
            moderator: ParticipantId::nil(),
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_display_name_changed() {
        let produced = serde_json::to_string_pretty(&ModerationEvent::DisplayNameChanged {
            target: ParticipantId::nil(),
            issued_by: ParticipantId::nil(),
            old_name: "Alice".parse().expect("valid display name"),
            new_name: "Bob".parse().expect("valid display name"),
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "display_name_changed",
          "target": "00000000-0000-0000-0000-000000000000",
          "issued_by": "00000000-0000-0000-0000-000000000000",
          "old_name": "Alice",
          "new_name": "Bob"
        }
        "#);
    }

    #[test]
    fn deserialize_display_name_changed() {
        let produced = serde_json::from_value(json!({
            "message": "display_name_changed",
            "target": "00000000-0000-0000-0000-000000000000",
            "issued_by": "00000000-0000-0000-0000-000000000000",
            "old_name": "Alice",
            "new_name": "Bob"
        }))
        .unwrap();
        let expected = ModerationEvent::DisplayNameChanged {
            target: ParticipantId::nil(),
            issued_by: ParticipantId::nil(),
            old_name: "Alice".parse().expect("valid display name"),
            new_name: "Bob".parse().expect("valid display name"),
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_display_name_assigned() {
        let produced = serde_json::to_string_pretty(&ModerationEvent::DisplayNameAssigned {
            new_name: Some("Bob".parse().expect("valid display name")),
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "display_name_assigned",
          "new_name": "Bob"
        }
        "#);
    }

    #[test]
    fn deserialize_display_name_assigned() {
        let produced = serde_json::from_value(json!({
            "message": "display_name_assigned",
            "new_name": "Bob"
        }))
        .unwrap();
        let expected = ModerationEvent::DisplayNameAssigned {
            new_name: Some("Bob".parse().expect("valid display name")),
        };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_display_name_assigned_none() {
        let produced =
            serde_json::to_string_pretty(&ModerationEvent::DisplayNameAssigned { new_name: None })
                .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "display_name_assigned",
          "new_name": null
        }
        "#);
    }

    #[test]
    fn deserialize_display_name_assigned_none() {
        let produced = serde_json::from_value(json!({
            "message": "display_name_assigned",
            "new_name": null
        }))
        .unwrap();
        let expected = ModerationEvent::DisplayNameAssigned { new_name: None };

        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_display_name_change_restrictions_enabled() {
        let produced =
            serde_json::to_string_pretty(&ModerationEvent::DisplayNameChangeRestrictionsEnabled {
                unrestricted_participants: HashSet::from([ParticipantId::nil()]),
            })
            .unwrap();
        assert_snapshot!(produced, @r#"
        {
          "message": "display_name_change_restrictions_enabled",
          "unrestricted_participants": [
            "00000000-0000-0000-0000-000000000000"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_display_name_change_restrictions_enabled() {
        let produced = serde_json::from_value(json!({
            "message": "display_name_change_restrictions_enabled",
            "unrestricted_participants": [
                "00000000-0000-0000-0000-000000000000",
            ]
        }))
        .unwrap();
        let expected = ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::from([ParticipantId::nil()]),
        };
        assert_eq!(expected, produced);
    }

    #[test]
    fn serialize_display_name_change_restrictions_disabled() {
        let produced =
            serde_json::to_string_pretty(&ModerationEvent::DisplayNameChangeRestrictionsDisabled)
                .unwrap();
        assert_snapshot!(produced, @r#"
        {
          "message": "display_name_change_restrictions_disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_display_name_change_restrictions_disabled() {
        let produced = serde_json::from_value(json!({
            "message": "display_name_change_restrictions_disabled"
        }))
        .unwrap();
        let expected = ModerationEvent::DisplayNameChangeRestrictionsDisabled;
        assert_eq!(expected, produced);
    }
}
