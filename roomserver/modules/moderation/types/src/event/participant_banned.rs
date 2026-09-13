// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::banned_participant::BannedParticipant;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Received by moderators on join or when a participant gets banned
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BannedParticipantInfo {
    /// The participant that got banned
    pub participant_id: ParticipantId,

    #[serde(flatten)]
    pub banned_participant: BannedParticipant,
}

impl From<(&ParticipantId, &BannedParticipant)> for BannedParticipantInfo {
    fn from((participant_id, banned_participant): (&ParticipantId, &BannedParticipant)) -> Self {
        Self {
            participant_id: *participant_id,
            banned_participant: banned_participant.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_common::{time::Timestamp, users::DisplayName};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_banned_participant_info() {
        let info = BannedParticipantInfo {
            participant_id: ParticipantId::nil(),
            banned_participant: BannedParticipant {
                display_name: DisplayName::from_str_lossy("Alice"),
                avatar_url: "https://example.org/alice.png".to_owned(),
                banned_by: ParticipantId::nil(),
                banned_at: Timestamp::unix_epoch(),
            },
        };

        assert_snapshot!(serde_json::to_string_pretty(&info).unwrap(), @r#"
        {
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "display_name": "Alice",
          "avatar_url": "https://example.org/alice.png",
          "banned_by": "00000000-0000-0000-0000-000000000000",
          "banned_at": "1970-01-01T00:00:00Z"
        }
        "#);
    }

    #[test]
    fn deserialize_banned_participant_info() {
        let json = json!({
            "participant_id": "00000000-0000-0000-0000-000000000000",
            "display_name": "Alice",
            "avatar_url": "https://example.org/alice.png",
            "banned_by": "00000000-0000-0000-0000-000000000000",
            "banned_at": "1970-01-01T00:00:00Z"
        });

        let produced: BannedParticipantInfo = serde_json::from_value(json).unwrap();
        let expected = BannedParticipantInfo {
            participant_id: ParticipantId::nil(),
            banned_participant: BannedParticipant {
                display_name: DisplayName::from_str_lossy("Alice"),
                avatar_url: "https://example.org/alice.png".to_owned(),
                banned_by: ParticipantId::nil(),
                banned_at: Timestamp::unix_epoch(),
            },
        };

        assert_eq!(produced, expected);
    }
}
