// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling invalid message for the `legal-vote` namespace.

use serde::{Deserialize, Serialize};

/// Describes the reason for invalid vote results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum Invalid {
    /// An abstain vote was found when the vote itself has abstain disabled.
    AbstainDisabled,

    /// The protocols vote count is not equal to the votes vote count.
    VoteCountInconsistent,

    /// The protocol entries are inconsistent.
    ProtocolInconsistent,
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_abstain_disabled_invalid() {
        let produced = serde_json::to_value(Invalid::AbstainDisabled).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "reason": "abstain_disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_abstain_disabled_invalid() {
        let produced: Invalid = serde_json::from_value(json!({
            "reason": "abstain_disabled",
        }))
        .unwrap();

        let expected = Invalid::AbstainDisabled;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_vote_count_inconsistent_invalid() {
        let produced = serde_json::to_value(Invalid::VoteCountInconsistent).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "reason": "vote_count_inconsistent"
        }
        "#);
    }

    #[test]
    fn deserialize_vote_count_inconsistent_invalid() {
        let produced: Invalid = serde_json::from_value(json!({
            "reason": "vote_count_inconsistent",
        }))
        .unwrap();

        let expected = Invalid::VoteCountInconsistent;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_protocol_inconsistent_invalid() {
        let produced = serde_json::to_value(Invalid::ProtocolInconsistent).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "reason": "protocol_inconsistent"
        }
        "#);
    }

    #[test]
    fn deserialize_protocol_inconsistent_invalid() {
        let produced: Invalid = serde_json::from_value(json!({
            "reason": "protocol_inconsistent",
        }))
        .unwrap();

        let expected = Invalid::ProtocolInconsistent;

        assert_eq!(produced, expected);
    }
}
