// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// Represents the possible choices a voter can make in the voting process.
///
/// The `Abstain` option can be disabled through the vote parameters (See
/// [`UserParameters`](crate::user_parameters::UserParameters)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteOption {
    /// Indicates a vote in favor of the proposal.
    Yes,

    /// Indicates a vote against the proposal.
    No,

    /// Indicates the voter is abstaining from voting.
    ///
    /// This option can be disabled based on vote parameters.
    Abstain,
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_yes_vote_option() {
        let produced = serde_json::to_value(VoteOption::Yes).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#""yes""#);
    }

    #[test]
    fn deserialize_yes_vote_option() {
        let produced: VoteOption = serde_json::from_value(json!("yes")).unwrap();

        let expected = VoteOption::Yes;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_no_vote_option() {
        let produced = serde_json::to_value(VoteOption::No).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#""no""#);
    }

    #[test]
    fn deserialize_no_vote_option() {
        let produced: VoteOption = serde_json::from_value(json!("no")).unwrap();

        let expected = VoteOption::No;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_abstain_vote_option() {
        let produced = serde_json::to_value(VoteOption::Abstain).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#""abstain""#);
    }

    #[test]
    fn deserialize_abstain_vote_option() {
        let produced: VoteOption = serde_json::from_value(json!("abstain")).unwrap();

        let expected = VoteOption::Abstain;

        assert_eq!(produced, expected);
    }
}
