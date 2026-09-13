// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::UserId;
use opentalk_types_signaling::ParticipantId;

/// Information about a voting user.
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    /// The user ID of the voting participant.
    pub issuer: UserId,

    /// The participant ID.
    pub participant_id: ParticipantId,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(UserInfo {
            issuer: UserId::from_u128(1),
            participant_id: ParticipantId::from_u128(2),
        })
        .unwrap();

        let expected = json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: UserInfo = serde_json::from_value(json!({
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
        }))
        .unwrap();

        let expected = UserInfo {
            issuer: UserId::from_u128(1),
            participant_id: ParticipantId::from_u128(2),
        };

        assert_eq!(produced, expected);
    }
}
