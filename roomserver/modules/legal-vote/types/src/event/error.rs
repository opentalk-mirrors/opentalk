// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_signaling::storage::assets::StorageError;
use opentalk_roomserver_types::signaling::module_error::ModuleError;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// The error kind sent to the user.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum LegalVoteError {
    /// A vote is already active.
    VoteAlreadyActive,

    /// No vote is currently taking place.
    NoVoteActive,

    /// The provided vote id is invalid in the requested context.
    InvalidVoteId,

    /// The user used a token that was either already used or not valid.
    InvalidToken,

    /// The selected vote option is not allowed.
    InvalidOption,

    /// The provided parameters are invalid.
    InvalidParameters,

    /// The provided allow list contains ineligible participants.
    /// Only registered users are allowed to vote.
    IneligibleParticipants {
        /// The identifiers of the ineligible participants.
        participants: Vec<ParticipantId>,
    },

    /// The requesting user has insufficient permissions.
    InsufficientPermissions,

    /// Internal error while generating the report
    GenerateReport,

    /// The requesting user has exceeded their storage.
    StorageExceeded,

    /// An internal error occurred while saving the whiteboard pdf.
    InternalStorage,

    /// A internal server error occurred.
    Internal,
}

impl From<StorageError> for LegalVoteError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::QuotaExceeded => Self::StorageExceeded,
            StorageError::Internal(..) | StorageError::ReadAsset(..) => Self::InternalStorage,
        }
    }
}

impl ModuleError for LegalVoteError {}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_vote_already_active_error() {
        let produced = serde_json::to_value(LegalVoteError::VoteAlreadyActive).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "vote_already_active"
        }
        "#);
    }

    #[test]
    fn deserialize_vote_already_active_error() {
        let produced: LegalVoteError =
            serde_json::from_value(json!({"error": "vote_already_active"})).unwrap();
        assert_eq!(produced, LegalVoteError::VoteAlreadyActive);
    }

    #[test]
    fn serialize_no_vote_active_error() {
        let produced = serde_json::to_value(LegalVoteError::NoVoteActive).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "no_vote_active"
        }
        "#);
    }

    #[test]
    fn deserialize_no_vote_active_error() {
        let produced: LegalVoteError =
            serde_json::from_value(json!({"error": "no_vote_active"})).unwrap();
        assert_eq!(produced, LegalVoteError::NoVoteActive);
    }

    #[test]
    fn serialize_invalid_vote_id_error() {
        let produced = serde_json::to_value(LegalVoteError::InvalidVoteId).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "invalid_vote_id"
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_vote_id_error() {
        let produced: LegalVoteError =
            serde_json::from_value(json!({"error": "invalid_vote_id"})).unwrap();
        assert_eq!(produced, LegalVoteError::InvalidVoteId);
    }

    #[test]
    fn serialize_invalid_token_error() {
        let produced = serde_json::to_value(LegalVoteError::InvalidToken).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "invalid_token"
        }
        "#);
    }

    #[test]
    fn deserialize_invalid_token_error() {
        let produced: LegalVoteError =
            serde_json::from_value(json!({"error": "invalid_token"})).unwrap();
        assert_eq!(produced, LegalVoteError::InvalidToken);
    }

    #[test]
    fn serialize_ineligible_participants_error() {
        let produced = serde_json::to_value(LegalVoteError::IneligibleParticipants {
            participants: vec![ParticipantId::from_u128(1)],
        })
        .unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "ineligible_participants",
          "participants": [
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_ineligible_participants_error() {
        let produced: LegalVoteError = serde_json::from_value(json!({
            "error": "ineligible_participants",
            "participants": ["00000000-0000-0000-0000-000000000001"],
        }))
        .unwrap();

        let expected = LegalVoteError::IneligibleParticipants {
            participants: vec![ParticipantId::from_u128(1)],
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_generate_error() {
        let produced = serde_json::to_value(LegalVoteError::GenerateReport).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "generate_report"
        }
        "#);
    }

    #[test]
    fn deserialize_generate_error() {
        let produced: LegalVoteError =
            serde_json::from_value(json!({"error": "generate_report"})).unwrap();
        assert_eq!(produced, LegalVoteError::GenerateReport);
    }

    #[test]
    fn serialize_internal_error() {
        let produced = serde_json::to_value(LegalVoteError::Internal).unwrap();
        let raw = serde_json::to_string_pretty(&produced).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "error": "internal"
        }
        "#);
    }

    #[test]
    fn deserialize_internal_error() {
        let produced: LegalVoteError =
            serde_json::from_value(json!({"error": "internal"})).unwrap();
        assert_eq!(produced, LegalVoteError::Internal);
    }
}
