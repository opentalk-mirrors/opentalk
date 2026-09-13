// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{parameters::Parameters, vote::VoteState};

/// Represents a summary of the vote, including parameters, state, and end time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteSummary {
    /// Parameters governing the vote, such as quorum and thresholds.
    #[serde(flatten)]
    pub parameters: Parameters,

    /// The current state of the vote, tracking progress and outcomes.
    #[serde(flatten)]
    pub state: VoteState,

    /// The optional timestamp indicating when the vote concluded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::TimeZone;
    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{
        user_parameters::{AllowedParticipants, Name, UserParameters},
        vote::LegalVoteId,
    };

    #[test]
    fn serialize_summary() {
        let summary = VoteSummary {
            parameters: Parameters {
                initiator_id: ParticipantId::from_u128(2),
                legal_vote_id: LegalVoteId::from_u128(3),
                start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                max_votes: 5,
                allowed_users: HashSet::new(),
                inner: UserParameters {
                    pseudonymous: false,
                    live: true,
                    name: Name::try_from("Test Name").unwrap(),
                    subtitle: None,
                    topic: None,
                    allowed_participants: AllowedParticipants::try_from([
                        ParticipantId::from_u128(1),
                    ])
                    .unwrap(),
                    enable_abstain: false,
                    auto_close: false,
                    duration: None,
                    create_pdf: false,
                    timezone: None,
                },
                token: None,
            },
            state: VoteState::Started,
            end_time: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
        };
        let raw = serde_json::to_string_pretty(&summary).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "initiator_id": "00000000-0000-0000-0000-000000000002",
          "legal_vote_id": "00000000-0000-0000-0000-000000000003",
          "start_time": "2025-01-01T00:00:00Z",
          "max_votes": 5,
          "pseudonymous": false,
          "live": true,
          "name": "Test Name",
          "allowed_participants": [
            "00000000-0000-0000-0000-000000000001"
          ],
          "enable_abstain": false,
          "auto_close": false,
          "create_pdf": false,
          "state": "started",
          "end_time": "2025-01-01T00:00:00Z"
        }
        "#);

        let summary = VoteSummary {
            parameters: Parameters {
                initiator_id: ParticipantId::from_u128(2),
                legal_vote_id: LegalVoteId::from_u128(3),
                start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                max_votes: 5,
                allowed_users: HashSet::new(),
                inner: UserParameters {
                    pseudonymous: false,
                    live: true,
                    name: Name::try_from("Test Name").unwrap(),
                    subtitle: None,
                    topic: None,
                    allowed_participants: AllowedParticipants::try_from([
                        ParticipantId::from_u128(1),
                    ])
                    .unwrap(),
                    enable_abstain: false,
                    auto_close: false,
                    duration: None,
                    create_pdf: false,
                    timezone: None,
                },
                token: None,
            },
            state: VoteState::Started,
            end_time: None,
        };
        let raw = serde_json::to_string_pretty(&summary).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "initiator_id": "00000000-0000-0000-0000-000000000002",
          "legal_vote_id": "00000000-0000-0000-0000-000000000003",
          "start_time": "2025-01-01T00:00:00Z",
          "max_votes": 5,
          "pseudonymous": false,
          "live": true,
          "name": "Test Name",
          "allowed_participants": [
            "00000000-0000-0000-0000-000000000001"
          ],
          "enable_abstain": false,
          "auto_close": false,
          "create_pdf": false,
          "state": "started"
        }
        "#);
    }

    #[test]
    fn deserialize_summary() {
        let produced: VoteSummary = serde_json::from_value(json!({
            "state": "started",
            "initiator_id": "00000000-0000-0000-0000-000000000002",
            "legal_vote_id": "00000000-0000-0000-0000-000000000003",
            "start_time": "2025-01-01T00:00:00Z",
            "max_votes": 5,
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "allowed_participants": ["00000000-0000-0000-0000-000000000001"],
            "enable_abstain": false,
            "auto_close": false,
            "create_pdf": false,
            "end_time": "2025-01-01T00:00:00Z",
        }))
        .unwrap();

        let expected = VoteSummary {
            parameters: Parameters {
                initiator_id: ParticipantId::from_u128(2),
                legal_vote_id: LegalVoteId::from_u128(3),
                start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                max_votes: 5,
                allowed_users: HashSet::new(),
                inner: UserParameters {
                    pseudonymous: false,
                    live: true,
                    name: Name::try_from("Test Name").unwrap(),
                    subtitle: None,
                    topic: None,
                    allowed_participants: AllowedParticipants::try_from([
                        ParticipantId::from_u128(1),
                    ])
                    .unwrap(),
                    enable_abstain: false,
                    auto_close: false,
                    duration: None,
                    create_pdf: false,
                    timezone: None,
                },
                token: None,
            },
            state: VoteState::Started,
            end_time: Some(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
        };

        assert_eq!(produced, expected);

        let produced: VoteSummary = serde_json::from_value(json!({
            "state": "started",
            "initiator_id": "00000000-0000-0000-0000-000000000002",
            "legal_vote_id": "00000000-0000-0000-0000-000000000003",
            "start_time": "2025-01-01T00:00:00Z",
            "max_votes": 5,
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "allowed_participants": ["00000000-0000-0000-0000-000000000001"],
            "enable_abstain": false,
            "auto_close": false,
            "create_pdf": false,
        }))
        .unwrap();

        let expected = VoteSummary {
            parameters: Parameters {
                initiator_id: ParticipantId::from_u128(2),
                legal_vote_id: LegalVoteId::from_u128(3),
                start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                max_votes: 5,
                allowed_users: HashSet::new(),
                inner: UserParameters {
                    pseudonymous: false,
                    live: true,
                    name: Name::try_from("Test Name").unwrap(),
                    subtitle: None,
                    topic: None,
                    allowed_participants: AllowedParticipants::try_from([
                        ParticipantId::from_u128(1),
                    ])
                    .unwrap(),
                    enable_abstain: false,
                    auto_close: false,
                    duration: None,
                    create_pdf: false,
                    timezone: None,
                },
                token: None,
            },
            state: VoteState::Started,
            end_time: None,
        };

        assert_eq!(produced, expected);
    }
}
