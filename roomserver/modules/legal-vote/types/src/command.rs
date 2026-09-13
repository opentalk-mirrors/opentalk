// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_common::time::TimeZone;
use serde::{Deserialize, Serialize};

use crate::{
    cancel::CustomCancelReason,
    event::LegalVoteEvent,
    issue::Issue,
    token::Token,
    user_parameters::UserParameters,
    vote::{LegalVoteId, VoteOption},
};

/// An incoming message issued by an participant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum LegalVoteCommand {
    /// Start a new vote.
    Start(UserParameters),

    /// Stop a vote and show results to the participants.
    Stop {
        /// The vote id of the targeted vote.
        legal_vote_id: LegalVoteId,
    },

    /// Cancel a vote.
    Cancel {
        /// The vote id of the targeted vote.
        legal_vote_id: LegalVoteId,

        /// The reason for the cancel.
        reason: CustomCancelReason,
    },

    /// Vote for an item on a vote.
    Vote {
        /// The vote id of the targeted vote.
        legal_vote_id: LegalVoteId,

        /// The chosen vote option.
        option: VoteOption,

        /// The user's vote token.
        token: Token,
    },

    /// Report an issue to the vote creator.
    ReportIssue {
        /// The identifier of the affected vote.
        legal_vote_id: LegalVoteId,

        /// The details of the reported issue.
        #[serde(flatten)]
        issue: Issue,
    },

    /// Generate a PDF from a passed vote.
    GeneratePdf {
        /// The identifier of the targeted vote.
        legal_vote_id: LegalVoteId,

        /// An optional timezone for the PDF generation. Defaults to UTC.
        /// The timezone should be in a format standardized by IANA (e.g., "CET" or
        /// "Europe/Vienna"). For more details, visit: <https://www.iana.org/time-zones>.
        #[serde(skip_serializing_if = "Option::is_none")]
        timezone: Option<TimeZone>,
    },
}

impl CreateReplica<LegalVoteEvent> for LegalVoteCommand {
    fn replicate(&self) -> Option<LegalVoteEvent> {
        // The vote command is not replicated, instead `LegalVoteEvent::Voted` is sent to all
        // connections of the voting participant. This event also contains information about whether
        // the vote was successful.
        None
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{
        cancel::CustomCancelReason,
        issue::{Issue, TechnicalIssueKind},
        token::Token,
        user_parameters::{AllowedParticipants, Name},
        vote::{LegalVoteId, VoteOption},
    };

    #[test]
    fn serialize_start_command() {
        let produced = serde_json::to_value(LegalVoteCommand::Start(UserParameters {
            pseudonymous: false,
            live: true,
            name: Name::try_from("Vote Test").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)])
                .unwrap(),
            enable_abstain: false,
            auto_close: false,
            duration: None,
            create_pdf: false,
            timezone: None,
        }))
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "start",
          "allowed_participants": [
            "00000000-0000-0000-0000-000000000001"
          ],
          "auto_close": false,
          "create_pdf": false,
          "enable_abstain": false,
          "live": true,
          "name": "Vote Test",
          "pseudonymous": false
        }
        "#);
    }

    #[test]
    fn deserializiation_start_command() {
        let produced: LegalVoteCommand = serde_json::from_value(json!({
            "action": "start",
            "pseudonymous": false,
            "live": true,
            "name": "Vote Test",
            "allowed_participants": ["00000000-0000-0000-0000-000000000001"],
            "enable_abstain": false,
            "auto_close": false,
            "create_pdf": false
        }))
        .unwrap();

        let expected = LegalVoteCommand::Start(UserParameters {
            pseudonymous: false,
            live: true,
            name: Name::try_from("Vote Test").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)])
                .unwrap(),
            enable_abstain: false,
            auto_close: false,
            duration: None,
            create_pdf: false,
            timezone: None,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_stop_command() {
        let produced = serde_json::to_value(LegalVoteCommand::Stop {
            legal_vote_id: LegalVoteId::from_u128(1),
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "stop",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001"
        }
        "#);
    }

    #[test]
    fn deserializiation_stop_command() {
        let produced: LegalVoteCommand = serde_json::from_value(json!({
            "action": "stop",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
        }))
        .unwrap();

        let expected = LegalVoteCommand::Stop {
            legal_vote_id: LegalVoteId::from_u128(1),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_cancel_command() {
        let produced = serde_json::to_value(LegalVoteCommand::Cancel {
            legal_vote_id: LegalVoteId::from_u128(1),
            reason: CustomCancelReason::try_from("Test Reason").unwrap(),
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "cancel",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001",
          "reason": "Test Reason"
        }
        "#);
    }

    #[test]
    fn deserializiation_cancel_command() {
        let produced: LegalVoteCommand = serde_json::from_value(json!({
            "action": "cancel",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "reason": "Test Reason",
        }))
        .unwrap();

        let expected = LegalVoteCommand::Cancel {
            legal_vote_id: LegalVoteId::from_u128(1),
            reason: CustomCancelReason::try_from("Test Reason").unwrap(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_vote_command() {
        let produced = serde_json::to_value(LegalVoteCommand::Vote {
            legal_vote_id: LegalVoteId::from_u128(1),
            option: VoteOption::Yes,
            token: Token::from_str("1111Cn8eVZg").unwrap(),
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "vote",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001",
          "option": "yes",
          "token": "1111Cn8eVZg"
        }
        "#);
    }

    #[test]
    fn deserializiation_vote_command() {
        let produced: LegalVoteCommand = serde_json::from_value(json!({
            "action": "vote",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "option": "yes",
            "token": "1111Cn8eVZg",
        }))
        .unwrap();

        let expected = LegalVoteCommand::Vote {
            legal_vote_id: LegalVoteId::from_u128(1),
            option: VoteOption::Yes,
            token: Token::from_str("1111Cn8eVZg").unwrap(),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_report_issue_command() {
        let produced = serde_json::to_value(LegalVoteCommand::ReportIssue {
            legal_vote_id: LegalVoteId::from_u128(1),
            issue: Issue::Technical {
                kind: TechnicalIssueKind::Audio,
                description: None,
            },
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "report_issue",
          "kind": "audio",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001"
        }
        "#);
    }

    #[test]
    fn deserializiation_report_issue_command() {
        let produced: LegalVoteCommand = serde_json::from_value(json!({
            "action": "report_issue",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
            "kind": "audio",
        }))
        .unwrap();

        let expected = LegalVoteCommand::ReportIssue {
            legal_vote_id: LegalVoteId::from_u128(1),
            issue: Issue::Technical {
                kind: TechnicalIssueKind::Audio,
                description: None,
            },
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_generate_pdf_command() {
        let produced = serde_json::to_value(LegalVoteCommand::GeneratePdf {
            legal_vote_id: LegalVoteId::from_u128(1),
            timezone: None,
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "generate_pdf",
          "legal_vote_id": "00000000-0000-0000-0000-000000000001"
        }
        "#);
    }

    #[test]
    fn deserializiation_generate_pdf_command() {
        let produced: LegalVoteCommand = serde_json::from_value(json!({
            "action": "generate_pdf",
            "legal_vote_id": "00000000-0000-0000-0000-000000000001",
        }))
        .unwrap();

        let expected = LegalVoteCommand::GeneratePdf {
            legal_vote_id: LegalVoteId::from_u128(1),
            timezone: None,
        };

        assert_eq!(produced, expected);
    }
}
