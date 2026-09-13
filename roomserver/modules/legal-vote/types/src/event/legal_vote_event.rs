// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use opentalk_types_common::assets::AssetId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    cancel::CancelReason,
    event::{FinalResults, LegalVoteError, Results, StopKind},
    issue::Issue,
    parameters::Parameters,
    token::Token,
    vote::{LegalVoteId, VoteOption},
};

/// A message sent to a participant via a WebSocket connection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum LegalVoteEvent {
    /// The vote has started.
    Started(Parameters),

    /// Response to a previous vote request.
    Voted {
        /// The vote id of the requested vote
        legal_vote_id: LegalVoteId,

        /// The response to the vote request.
        vote_option: VoteOption,

        /// The participant who issued the vote.
        issuer: ParticipantId,

        /// The token that was consumed during the vote.
        consumed_token: Token,
    },

    /// The results of a specific vote have changed.
    Updated {
        /// The vote id.
        legal_vote_id: LegalVoteId,

        /// The vote results.
        #[serde(flatten)]
        results: Results,
    },

    /// The vote has been stopped.
    Stopped {
        /// The unique identifier of the vote.
        legal_vote_id: LegalVoteId,

        /// The final results of the vote.
        #[serde(flatten)]
        results: FinalResults,

        /// The reason for stopping the vote.
        #[serde(flatten)]
        kind: StopKind,

        /// The timestamp when the voting process ended.
        end_time: DateTime<Utc>,
    },

    /// The vote has been canceled.
    Canceled {
        /// The identifier of the canceled vote.
        legal_vote_id: LegalVoteId,

        /// The reason for canceling the vote.
        #[serde(flatten)]
        reason: CancelReason,

        /// The timestamp when the vote was canceled.
        end_time: DateTime<Utc>,
    },

    /// A participant has reported an issue.
    ReportedIssue {
        /// The identifier of the affected vote.
        legal_vote_id: LegalVoteId,

        /// The participant who reported the issue, if applicable.
        #[serde(skip_serializing_if = "Option::is_none")]
        participant_id: Option<ParticipantId>,

        /// Details of the reported issue.
        #[serde(flatten)]
        issue: Issue,
    },

    /// The PDF report has been generated and is available as an asset.
    ReportGenerated {
        /// The filename of the PDF.
        filename: String,

        /// The identifier of the related vote.
        legal_vote_id: LegalVoteId,

        /// The unique identifier of the asset.
        asset_id: AssetId,
    },

    /// An error message caused by invalid requests or internal errors.
    Error(LegalVoteError),
}

impl From<LegalVoteError> for LegalVoteEvent {
    fn from(err: LegalVoteError) -> Self {
        Self::Error(err)
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::{HashMap, HashSet},
        str::FromStr,
    };

    use chrono::{TimeZone, Utc};
    use insta::assert_snapshot;
    use opentalk_types_common::assets::AssetId;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{
        cancel::CancelReason,
        event::{FinalResults, Results, StopKind, VotingRecord},
        invalid::Invalid,
        issue::Issue,
        tally::Tally,
        token::Token,
        user_parameters::{AllowedParticipants, Name, UserParameters},
        vote::{LegalVoteId, VoteOption},
    };

    #[test]
    fn serialize_started_legal_vote_event() {
        let event = LegalVoteEvent::Started(Parameters {
            initiator_id: ParticipantId::from_u128(1),
            legal_vote_id: LegalVoteId::nil(),
            start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            max_votes: 5,
            allowed_users: HashSet::new(),
            inner: UserParameters {
                pseudonymous: false,
                live: true,
                name: Name::try_from("Test Name").unwrap(),
                subtitle: None,
                topic: None,
                allowed_participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)])
                    .unwrap(),
                enable_abstain: false,
                auto_close: false,
                duration: None,
                create_pdf: false,
                timezone: None,
            },
            token: None,
        });
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "started",
          "initiator_id": "00000000-0000-0000-0000-000000000001",
          "legal_vote_id": "00000000-0000-0000-0000-000000000000",
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
          "create_pdf": false
        }
        "#);
    }

    #[test]
    fn deserialize_started_legal_vote_event() {
        let produced: LegalVoteEvent = serde_json::from_value(json!({
            "message": "started",
            "initiator_id": "00000000-0000-0000-0000-000000000001",
            "legal_vote_id": "00000000-0000-0000-0000-000000000000",
            "start_time":"2025-01-01T00:00:00Z",
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

        let expected = LegalVoteEvent::Started(Parameters {
            initiator_id: ParticipantId::from_u128(1),
            legal_vote_id: LegalVoteId::nil(),
            start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            max_votes: 5,
            allowed_users: HashSet::new(),
            inner: UserParameters {
                pseudonymous: false,
                live: true,
                name: Name::try_from("Test Name").unwrap(),
                subtitle: None,
                topic: None,
                allowed_participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)])
                    .unwrap(),
                enable_abstain: false,
                auto_close: false,
                duration: None,
                create_pdf: false,
                timezone: None,
            },
            token: None,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_voted_legal_vote_event() {
        let produced = LegalVoteEvent::Voted {
            legal_vote_id: LegalVoteId::from_u128(1),
            vote_option: VoteOption::Yes,
            issuer: ParticipantId::from_u128(1),
            consumed_token: Token::from_str("1111Cn8eVZg").unwrap(),
        };
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "voted",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001",
          "vote_option": "yes",
          "issuer": "00000000-0000-0000-0000-000000000001",
          "consumed_token": "1111Cn8eVZg"
        }
        "#);
    }

    #[test]
    fn deserialize_voted_legal_vote_event() {
        let produced: LegalVoteEvent = serde_json::from_value(json!({
            "message": "voted",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "vote_option": "yes",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "consumed_token": "1111Cn8eVZg",
        }))
        .unwrap();

        let expected = LegalVoteEvent::Voted {
            legal_vote_id: LegalVoteId::from_u128(1),
            vote_option: VoteOption::Yes,
            issuer: ParticipantId::from_u128(1),
            consumed_token: Token::from_str("1111Cn8eVZg").unwrap(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_updated_legal_vote_event() {
        let event = LegalVoteEvent::Updated {
            legal_vote_id: LegalVoteId::from_u128(1),
            results: Results {
                tally: Tally {
                    yes: 1,
                    no: 0,
                    abstain: None,
                },
                voting_record: VotingRecord::UserVotes(
                    vec![(ParticipantId::from_u128(2), VoteOption::Yes)]
                        .into_iter()
                        .collect::<HashMap<ParticipantId, VoteOption>>(),
                ),
            },
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "updated",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001",
          "yes": 1,
          "no": 0,
          "voting_record": {
            "00000000-0000-0000-0000-000000000002": "yes"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_updated_legal_vote_event() {
        let produced: LegalVoteEvent = serde_json::from_value(json!({
            "message": "updated",
               "legal_vote_id": "00000000-0000-0000-0000-000000000001",
               "yes": 1,
               "no": 0,
               "voting_record": {
                   "00000000-0000-0000-0000-000000000002": "yes",
               },
        }))
        .unwrap();

        let expected = LegalVoteEvent::Updated {
            legal_vote_id: LegalVoteId::from_u128(1),
            results: Results {
                tally: Tally {
                    yes: 1,
                    no: 0,
                    abstain: None,
                },
                voting_record: VotingRecord::UserVotes(
                    vec![(ParticipantId::from_u128(2), VoteOption::Yes)]
                        .into_iter()
                        .collect::<HashMap<ParticipantId, VoteOption>>(),
                ),
            },
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_stopped_legal_vote_event() {
        let event = LegalVoteEvent::Stopped {
            legal_vote_id: LegalVoteId::from_u128(1),
            results: FinalResults::Invalid(Invalid::AbstainDisabled),
            kind: StopKind::Auto,
            end_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "stopped",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001",
          "results": "invalid",
          "reason": "abstain_disabled",
          "kind": "auto",
          "end_time": "2025-01-01T00:00:00Z"
        }
        "#);
    }

    #[test]
    fn deserialize_stopped_legal_vote_event() {
        let produced: LegalVoteEvent = serde_json::from_value(json!({
            "message": "stopped",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "results": "invalid",
            "reason": "abstain_disabled",
            "kind": "auto",
            "end_time":"2025-01-01T00:00:00Z",
        }))
        .unwrap();

        let expected = LegalVoteEvent::Stopped {
            legal_vote_id: LegalVoteId::from_u128(1),
            results: FinalResults::Invalid(Invalid::AbstainDisabled),
            kind: StopKind::Auto,
            end_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_canceled_legal_vote_event() {
        let event = LegalVoteEvent::Canceled {
            legal_vote_id: LegalVoteId::from_u128(1),
            reason: CancelReason::RoomDestroyed,
            end_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "canceled",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001",
          "reason": "room_destroyed",
          "end_time": "2025-01-01T00:00:00Z"
        }
        "#);
    }

    #[test]
    fn deserialize_canceled_legal_vote_event() {
        let produced: LegalVoteEvent = serde_json::from_value(json!({
            "message": "canceled",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "reason": "room_destroyed",
            "end_time":"2025-01-01T00:00:00Z",
        }))
        .unwrap();

        let expected = LegalVoteEvent::Canceled {
            legal_vote_id: LegalVoteId::from_u128(1),
            reason: CancelReason::RoomDestroyed,
            end_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_reported_issue_legal_vote_event() {
        let event = LegalVoteEvent::ReportedIssue {
            legal_vote_id: LegalVoteId::from_u128(1),
            participant_id: None,
            issue: Issue::Other {
                description: "Test Description".to_string(),
            },
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "reported_issue",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001",
          "description": "Test Description"
        }
        "#);
    }

    #[test]
    fn deserialize_reported_issue_legal_vote_event() {
        let produced: LegalVoteEvent = serde_json::from_value(json!({
            "message": "reported_issue",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "description": "Test Description",
        }))
        .unwrap();

        let expected = LegalVoteEvent::ReportedIssue {
            legal_vote_id: LegalVoteId::from_u128(1),
            participant_id: None,
            issue: Issue::Other {
                description: "Test Description".to_string(),
            },
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_error_legal_vote_event() {
        let event = LegalVoteEvent::Error(LegalVoteError::Internal);
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "error",
          "error": "internal"
        }
        "#);
    }

    #[test]
    fn deserialize_error_issue_legal_vote_event() {
        let produced: LegalVoteEvent = serde_json::from_value(json!({
            "message": "error",
            "error": "internal",
        }))
        .unwrap();

        let expected = LegalVoteEvent::Error(LegalVoteError::Internal);

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_report_generated_legal_vote_event() {
        let event = LegalVoteEvent::ReportGenerated {
            filename: "test_filename".to_string(),
            legal_vote_id: LegalVoteId::from_u128(1),
            asset_id: AssetId::from_u128(2),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "report_generated",
          "filename": "test_filename",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001",
          "asset_id": "00000000-0000-0000-0000-000000000002"
        }
        "#);
    }

    #[test]
    fn deserialize_report_generated_issue_legal_vote_event() {
        let produced: LegalVoteEvent = serde_json::from_value(json!({
            "message": "report_generated",
            "filename": "test_filename",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "asset_id": "00000000-0000-0000-0000-000000000002",
        }))
        .unwrap();

        let expected = LegalVoteEvent::ReportGenerated {
            filename: "test_filename".to_string(),
            legal_vote_id: LegalVoteId::from_u128(1),
            asset_id: AssetId::from_u128(2),
        };

        assert_eq!(produced, expected);
    }
}
