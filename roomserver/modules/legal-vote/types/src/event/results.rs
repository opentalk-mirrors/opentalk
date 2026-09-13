// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::{event::VotingRecord, tally::Tally};

/// Represents the final tally and voting record of a vote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Results {
    /// The count of votes for each option.
    #[serde(flatten)]
    pub tally: Tally,

    /// The detailed record of how votes were cast.
    pub voting_record: VotingRecord,
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::vote::VoteOption;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(Results {
            tally: Tally {
                yes: 0,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(
                vec![(ParticipantId::from_u128(1), VoteOption::No)]
                    .into_iter()
                    .collect::<HashMap<ParticipantId, VoteOption>>(),
            ),
        })
        .unwrap();

        let expected = json!({
            "yes": 0,
            "no": 1,
            "voting_record": {
                "00000000-0000-0000-0000-000000000001": "no"
            },
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: Results = serde_json::from_value(json!({
            "yes": 0,
            "no": 1,
            "voting_record": {
                "00000000-0000-0000-0000-000000000001": "no"
            },
        }))
        .unwrap();

        let expected = Results {
            tally: Tally {
                yes: 0,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(
                vec![(ParticipantId::from_u128(1), VoteOption::No)]
                    .into_iter()
                    .collect::<HashMap<ParticipantId, VoteOption>>(),
            ),
        };

        assert_eq!(produced, expected);
    }
}
