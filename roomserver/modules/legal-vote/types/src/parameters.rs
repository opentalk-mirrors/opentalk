// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling parameters for the `legal-vote` namespace.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use opentalk_types_common::users::UserId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{token::Token, user_parameters::UserParameters, vote::LegalVoteId};

/// Wraps the [`UserParameters`] with additional server side information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameters {
    /// The participant id of the vote initiator.
    pub initiator_id: ParticipantId,

    /// The unique id of this vote.
    pub legal_vote_id: LegalVoteId,

    /// The time the vote got started.
    pub start_time: DateTime<Utc>,

    /// The maximum amount of votes possible.
    pub max_votes: u32,

    /// List of resolved user ID's.
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub allowed_users: HashSet<UserId>,

    /// Parameters set by the initiator.
    #[serde(flatten)]
    pub inner: UserParameters,

    /// Token for users who are allowed to participate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<Token>,
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use chrono::TimeZone;
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::user_parameters::{AllowedParticipants, Name};

    #[test]
    fn serialize_parameters() {
        let parameters = Parameters {
            initiator_id: ParticipantId::from_u128(1),
            legal_vote_id: LegalVoteId::nil(),
            start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            max_votes: 5,
            allowed_users: HashSet::from_iter([UserId::from_u128(1)]),
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
            token: Some(Token::from_str("1111Cn8eVZg").unwrap()),
        };
        let raw = serde_json::to_string_pretty(&parameters).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "initiator_id": "00000000-0000-0000-0000-000000000001",
          "legal_vote_id": "00000000-0000-0000-0000-000000000000",
          "start_time": "2025-01-01T00:00:00Z",
          "max_votes": 5,
          "allowed_users": [
            "00000000-0000-0000-0000-000000000001"
          ],
          "pseudonymous": false,
          "live": true,
          "name": "Test Name",
          "allowed_participants": [
            "00000000-0000-0000-0000-000000000001"
          ],
          "enable_abstain": false,
          "auto_close": false,
          "create_pdf": false,
          "token": "1111Cn8eVZg"
        }
        "#);

        let parameters = Parameters {
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
        };
        let raw = serde_json::to_string_pretty(&parameters).unwrap();

        assert_snapshot!(raw, @r#"
        {
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
    fn deserialize_parameters() {
        let produced: Parameters = serde_json::from_value(json!({
            "initiator_id": "00000000-0000-0000-0000-000000000001",
            "legal_vote_id": "00000000-0000-0000-0000-000000000000",
            "start_time":"2025-01-01T00:00:00Z",
            "max_votes": 5,
            "allowed_users": [
               "00000000-0000-0000-0000-000000000001",
            ],
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "allowed_participants": [
               "00000000-0000-0000-0000-000000000001",
            ],
            "enable_abstain": false,
            "auto_close": false,
            "create_pdf": false,
            "token": "1111Cn8eVZg",
        }))
        .unwrap();

        let expected = Parameters {
            initiator_id: ParticipantId::from_u128(1),
            legal_vote_id: LegalVoteId::nil(),
            start_time: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            max_votes: 5,
            allowed_users: HashSet::from_iter([UserId::from_u128(1)]),
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
            token: Some(Token::from_str("1111Cn8eVZg").unwrap()),
        };

        assert_eq!(produced, expected);

        let produced: Parameters = serde_json::from_value(json!({
            "initiator_id": "00000000-0000-0000-0000-000000000001",
            "legal_vote_id": "00000000-0000-0000-0000-000000000000",
            "start_time":"2025-01-01T00:00:00Z",
            "max_votes": 5,
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "allowed_participants": [
               "00000000-0000-0000-0000-000000000001",
            ],
            "enable_abstain": false,
            "auto_close": false,
            "create_pdf": false,
        }))
        .unwrap();

        let expected = Parameters {
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
        };

        assert_eq!(produced, expected);
    }
}
