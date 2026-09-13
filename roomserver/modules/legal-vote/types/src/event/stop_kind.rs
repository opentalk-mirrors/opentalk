// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Describes the type of a vote stop
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "issuer")]
pub enum StopKind {
    /// A normal vote stop issued by a participant. Contains the [`ParticipantId`] of the issuer.
    ByParticipant(ParticipantId),
    /// The vote has been stopped automatically because all allowed users have voted.
    Auto,
    /// The vote expired due to a set duration.
    Expired,
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_by_participant_stop_kind() {
        let produced =
            serde_json::to_value(StopKind::ByParticipant(ParticipantId::from_u128(0))).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "issuer": "00000000-0000-0000-0000-000000000000",
          "kind": "by_participant"
        }
        "#);
    }

    #[test]
    fn deserialize_by_participant_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!({
            "kind": "by_participant",
            "issuer": "00000000-0000-0000-0000-000000000000",
        }))
        .unwrap();

        let expected = StopKind::ByParticipant(ParticipantId::from_u128(0));

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_auto_stop_kind() {
        let produced = serde_json::to_value(StopKind::Auto).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "auto"
        }
        "#);
    }

    #[test]
    fn deserialize_auto_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!({
            "kind": "auto",
        }))
        .unwrap();

        let expected = StopKind::Auto;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_expired_stop_kind() {
        let produced = serde_json::to_value(StopKind::Expired).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "kind": "expired"
        }
        "#);
    }

    #[test]
    fn deserialize_expired_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!({
            "kind": "expired",
        }))
        .unwrap();

        let expected = StopKind::Expired;

        assert_eq!(produced, expected);
    }
}
