// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum E2eeError {
    /// The targeted participant does not exist
    InvalidParticipantTarget,
    /// The invite is not valid
    InvalidInvite,
}

impl ModuleError for E2eeError {}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn invalid_participant_target() {
        let sample_serialized = json!({ "error": "invalid_participant_target" });
        let sample_err = E2eeError::InvalidParticipantTarget;

        // test serialize
        let json_value = serde_json::to_value(&sample_err).unwrap();
        assert_eq!(json_value, sample_serialized);

        // test deserialize
        let err: E2eeError = serde_json::from_value(sample_serialized).unwrap();
        assert_eq!(err, E2eeError::InvalidParticipantTarget);
    }

    #[test]
    fn serialize_invalid_invite() {
        let sample_serialized = json!({ "error": "invalid_invite" });
        let sample_err = E2eeError::InvalidInvite;

        // test serialize
        let json_value = serde_json::to_value(&sample_err).unwrap();
        assert_eq!(json_value, sample_serialized);

        // test deserialize
        let err: E2eeError = serde_json::from_value(sample_serialized).unwrap();
        assert_eq!(err, E2eeError::InvalidInvite);
    }
}
