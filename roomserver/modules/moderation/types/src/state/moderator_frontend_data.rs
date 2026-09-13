// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use opentalk_roomserver_types::{connection_id::ConnectionId, room_parameters::WaitingRoom};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::event::BannedParticipantInfo;

/// Moderation module state that is visible only to moderators
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeratorJoinInfo {
    /// The state of the waiting room
    pub waiting_room: WaitingRoom,

    /// Whether or not the guest access is enabled
    pub guest_access: bool,

    /// The participants that are currently in the waiting room (if any)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub waiting_room_participants: Vec<WaitingParticipantPeerData>,

    /// The participants that are currently banned from the room
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub banned_participants: Vec<BannedParticipantInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WaitingParticipantPeerData {
    /// The id of the participant
    pub participant_id: ParticipantId,

    /// The connection ids of the participant
    pub connections: Vec<ConnectionId>,

    /// Whether the participant was accepted to enter the meeting
    pub accepted: bool,

    /// The time when the participant joined the waiting room
    pub joined_at: DateTime<Utc>,

    /// The participants display name
    pub display_name: DisplayName,

    /// The participants avatar URL
    pub avatar_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_moderator_join_info() {
        let info = ModeratorJoinInfo {
            waiting_room: WaitingRoom::ForEveryone,
            guest_access: true,
            waiting_room_participants: vec![],
            banned_participants: vec![],
        };

        assert_snapshot!(serde_json::to_string_pretty(&info).unwrap(), @r#"
        {
          "waiting_room": "for_everyone",
          "guest_access": true
        }
        "#);
    }

    #[test]
    fn deserialize_moderator_join_info() {
        let json = json!({
            "waiting_room": "for_everyone",
            "guest_access": true
        });

        let produced: ModeratorJoinInfo = serde_json::from_value(json).unwrap();
        let expected = ModeratorJoinInfo {
            waiting_room: WaitingRoom::ForEveryone,
            guest_access: true,
            waiting_room_participants: vec![],
            banned_participants: vec![],
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_waiting_participant_peer_data() {
        let data = WaitingParticipantPeerData {
            participant_id: ParticipantId::nil(),
            connections: vec![ConnectionId::nil()],
            accepted: true,
            joined_at: DateTime::UNIX_EPOCH,
            display_name: DisplayName::from_str_lossy("Alice"),
            avatar_url: Some("https://example.org/alice.png".to_owned()),
        };

        assert_snapshot!(serde_json::to_string_pretty(&data).unwrap(), @r#"
        {
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "connections": [
            "00000000-0000-0000-0000-000000000000"
          ],
          "accepted": true,
          "joined_at": "1970-01-01T00:00:00Z",
          "display_name": "Alice",
          "avatar_url": "https://example.org/alice.png"
        }
        "#);
    }

    #[test]
    fn deserialize_waiting_participant_peer_data() {
        let json = json!({
            "participant_id": "00000000-0000-0000-0000-000000000000",
            "connections": [
                "00000000-0000-0000-0000-000000000000"
            ],
            "accepted": true,
            "joined_at": "1970-01-01T00:00:00Z",
            "display_name": "Alice",
            "avatar_url": "https://example.org/alice.png"
        });

        let produced: WaitingParticipantPeerData = serde_json::from_value(json).unwrap();
        let expected = WaitingParticipantPeerData {
            participant_id: ParticipantId::nil(),
            connections: vec![ConnectionId::nil()],
            accepted: true,
            joined_at: DateTime::UNIX_EPOCH,
            display_name: DisplayName::from_str_lossy("Alice"),
            avatar_url: Some("https://example.org/alice.png".to_owned()),
        };

        assert_eq!(produced, expected);
    }
}
