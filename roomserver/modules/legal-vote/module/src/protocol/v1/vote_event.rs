// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_legal_vote::{
    cancel::CancelReason, issue::Issue, parameters::Parameters, token::Token, vote::VoteOption,
};
use opentalk_types_common::users::UserId;

use crate::protocol::v1::{FinalResults, StopKind, UserInfo};

/// An event related to a legal vote.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum VoteEvent {
    /// The vote has started.
    Start {
        /// The user ID of the initiator.
        issuer: UserId,

        /// The parameters for the vote.
        parameters: Box<Parameters>,
    },

    /// A vote has been cast.
    Vote {
        /// User information of the voting participant.
        ///
        /// `None` if the vote is hidden.
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        user_info: Option<UserInfo>,

        /// The token used for voting.
        token: Token,

        /// The chosen vote option.
        option: VoteOption,
    },

    /// The vote has been stopped.
    Stop(StopKind),

    /// The final results of the vote.
    FinalResults(FinalResults),

    /// An issue has been reported.
    Issue {
        /// User information of the participant that reported the issue.
        ///
        /// `None` if the vote is hidden.
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        user_info: Option<UserInfo>,

        /// The kind of issue that the user encountered.
        #[serde(flatten)]
        issue: Issue,
    },

    /// A user has left the room.
    UserLeft {
        /// User information of the participant.
        ///
        /// [`None`] if the vote is hidden.
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        user_info: Option<UserInfo>,
    },

    /// A user has joined the room.
    UserJoined {
        /// User information of the participant.
        ///
        /// [`None`] if the vote is hidden.
        #[serde(flatten, skip_serializing_if = "Option::is_none")]
        user_info: Option<UserInfo>,
    },

    /// The vote has been canceled.
    Cancel {
        /// The user ID of the issuer of the cancellation.
        issuer: UserId,

        /// The reason for the cancellation.
        #[serde(flatten)]
        reason: CancelReason,
    },
}

#[cfg(test)]
mod tests {

    use std::{collections::HashSet, str::FromStr};

    use chrono::{TimeZone, Utc};
    use insta::assert_snapshot;
    use opentalk_roomserver_types_legal_vote::{
        cancel::CancelReason,
        issue::{Issue, TechnicalIssueKind},
        parameters::Parameters,
        tally::Tally,
        token::Token,
        user_parameters::{AllowedParticipants, Name, UserParameters},
        vote::{LegalVoteId, VoteOption},
    };
    use opentalk_types_common::users::UserId;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::protocol::v1::UserInfo;

    #[test]
    fn serialize_start_vote_event() {
        let event = VoteEvent::Start {
            issuer: UserId::from_u128(1),
            parameters: Box::new(Parameters {
                initiator_id: ParticipantId::from_u128(1),
                legal_vote_id: LegalVoteId::from_u128(2),
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
                        ParticipantId::from_u128(3),
                    ])
                    .unwrap(),
                    enable_abstain: false,
                    auto_close: false,
                    duration: None,
                    create_pdf: false,
                    timezone: None,
                },
                token: None,
            }),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "event": "start",
          "issuer": "00000000-0000-0000-0000-000000000001",
          "parameters": {
            "initiator_id": "00000000-0000-0000-0000-000000000001",
            "legal_vote_id": "00000000-0000-0000-0000-000000000002",
            "start_time": "2025-01-01T00:00:00Z",
            "max_votes": 5,
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "allowed_participants": [
              "00000000-0000-0000-0000-000000000003"
            ],
            "enable_abstain": false,
            "auto_close": false,
            "create_pdf": false
          }
        }
        "#);
    }

    #[test]
    fn deserialize_start_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "start",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "parameters": {
                "initiator_id": "00000000-0000-0000-0000-000000000001",
                "legal_vote_id": "00000000-0000-0000-0000-000000000002",
                "start_time":"2025-01-01T00:00:00Z",
                "max_votes": 5,
                "pseudonymous": false,
                "live": true,
                "name": "Test Name",
                "allowed_participants": [
                   "00000000-0000-0000-0000-000000000003",
                ],
                "enable_abstain": false,
                "auto_close": false,
                "create_pdf": false,
            }
        }))
        .unwrap();

        let expected = VoteEvent::Start {
            issuer: UserId::from_u128(1),
            parameters: Box::new(Parameters {
                initiator_id: ParticipantId::from_u128(1),
                legal_vote_id: LegalVoteId::from_u128(2),
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
                        ParticipantId::from_u128(3),
                    ])
                    .unwrap(),
                    enable_abstain: false,
                    auto_close: false,
                    duration: None,
                    create_pdf: false,
                    timezone: None,
                },
                token: None,
            }),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_vote_vote_event() {
        let produced = serde_json::to_value(VoteEvent::Vote {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
            token: Token::from_str("1111Cn8eVZg").unwrap(),
            option: VoteOption::Yes,
        })
        .unwrap();

        let expected = json!({
            "event": "vote",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
            "token": "1111Cn8eVZg",
            "option": "yes",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_vote_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "vote",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002",
            "token": "1111Cn8eVZg",
            "option": "yes",
        }))
        .unwrap();

        let expected = VoteEvent::Vote {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
            token: Token::from_str("1111Cn8eVZg").unwrap(),
            option: VoteOption::Yes,
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_stop_vote_event() {
        let produced =
            serde_json::to_value(VoteEvent::Stop(StopKind::ByUser(UserId::from_u128(1)))).unwrap();

        let expected = json!({
            "event": "stop",
            "by_user": "00000000-0000-0000-0000-000000000001",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_stop_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "stop",
            "by_user": "00000000-0000-0000-0000-000000000001",
        }))
        .unwrap();

        let expected = VoteEvent::Stop(StopKind::ByUser(UserId::from_u128(1)));

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_final_results_vote_event() {
        let produced = serde_json::to_value(VoteEvent::FinalResults(FinalResults::Valid(Tally {
            yes: 1,
            no: 0,
            abstain: None,
        })))
        .unwrap();

        let expected = json!({
            "event": "final_results",
            "results": "valid",
            "yes": 1,
            "no": 0,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_final_results_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "final_results",
            "results": "valid",
            "yes": 1,
            "no": 0,
        }))
        .unwrap();

        let expected = VoteEvent::FinalResults(FinalResults::Valid(Tally {
            yes: 1,
            no: 0,
            abstain: None,
        }));

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_issue_vote_event() {
        let produced = serde_json::to_value(VoteEvent::Issue {
            user_info: None,
            issue: Issue::Technical {
                kind: TechnicalIssueKind::Audio,
                description: None,
            },
        })
        .unwrap();

        let expected = json!({
            "event": "issue",
            "kind": "audio",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_issue_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "issue",
            "kind": "audio",
        }))
        .unwrap();

        let expected = VoteEvent::Issue {
            user_info: None,
            issue: Issue::Technical {
                kind: TechnicalIssueKind::Audio,
                description: None,
            },
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_user_left_vote_event() {
        let produced = serde_json::to_string_pretty(&VoteEvent::UserLeft {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "event": "user_left",
          "issuer": "00000000-0000-0000-0000-000000000001",
          "participant_id": "00000000-0000-0000-0000-000000000002"
        }
        "#);
    }

    #[test]
    fn deserialize_user_left_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "user_left",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "participant_id": "00000000-0000-0000-0000-000000000002"
        }))
        .unwrap();

        let expected = VoteEvent::UserLeft {
            user_info: Some(UserInfo {
                issuer: UserId::from_u128(1),
                participant_id: ParticipantId::from_u128(2),
            }),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_user_joined_vote_event() {
        let produced = serde_json::to_value(VoteEvent::UserJoined { user_info: None }).unwrap();

        let expected = json!({
            "event": "user_joined",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_user_joined_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "user_joined",
        }))
        .unwrap();

        let expected = VoteEvent::UserJoined { user_info: None };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_cancel_vote_event() {
        let produced = serde_json::to_value(VoteEvent::Cancel {
            issuer: UserId::from_u128(1),
            reason: CancelReason::InitiatorLeft,
        })
        .unwrap();

        let expected = json!({
            "event": "cancel",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "reason": "initiator_left",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_cancel_vote_event() {
        let produced: VoteEvent = serde_json::from_value(json!({
            "event": "cancel",
            "issuer": "00000000-0000-0000-0000-000000000001",
            "reason": "initiator_left",
        }))
        .unwrap();

        let expected = VoteEvent::Cancel {
            issuer: UserId::from_u128(1),
            reason: CancelReason::InitiatorLeft,
        };

        assert_eq!(produced, expected);
    }
}
