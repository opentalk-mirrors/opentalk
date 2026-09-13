// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{token::Token, vote::VoteOption};

/// Represents a record of votes, either by identified users or pseudonymous tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VotingRecord {
    /// A mapping of user identifiers to their respective votes.
    UserVotes(HashMap<ParticipantId, VoteOption>),

    /// A mapping of pseudonymous tokens to their respective votes.
    TokenVotes(HashMap<Token, VoteOption>),
}

impl VotingRecord {
    /// Returns a list of all recorded votes.
    pub fn vote_option_list(&self) -> Vec<VoteOption> {
        match self {
            Self::UserVotes(voters) => voters.values().copied().collect(),
            Self::TokenVotes(tokens) => tokens.values().copied().collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_user_votes_voting_record() {
        let produced = serde_json::to_value(VotingRecord::UserVotes(
            vec![(ParticipantId::from_u128(1), VoteOption::No)]
                .into_iter()
                .collect::<HashMap<ParticipantId, VoteOption>>(),
        ))
        .unwrap();

        let expected = json!({
            "00000000-0000-0000-0000-000000000001": "no",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_user_votes_voting_record() {
        let produced: VotingRecord = serde_json::from_value(json!({
            "00000000-0000-0000-0000-000000000001": "no",
        }))
        .unwrap();

        let expected = VotingRecord::UserVotes(
            vec![(ParticipantId::from_u128(1), VoteOption::No)]
                .into_iter()
                .collect::<HashMap<ParticipantId, VoteOption>>(),
        );

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_token_votes_voting_record() {
        let produced = serde_json::to_value(VotingRecord::TokenVotes(
            vec![(Token::new(1), VoteOption::No)]
                .into_iter()
                .collect::<HashMap<Token, VoteOption>>(),
        ))
        .unwrap();

        let expected = json!({
            "11111111112": "no",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_token_votes_voting_record() {
        let produced: VotingRecord = serde_json::from_value(json!({
            "11111111112": "no",
        }))
        .unwrap();

        let expected = VotingRecord::TokenVotes(
            vec![(Token::new(1), VoteOption::No)]
                .into_iter()
                .collect::<HashMap<Token, VoteOption>>(),
        );

        assert_eq!(produced, expected);
    }
}
