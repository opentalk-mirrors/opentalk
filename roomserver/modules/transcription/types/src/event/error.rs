// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use opentalk_roomserver_types::signaling::module_error::ModuleError;
use serde::{Deserialize, Serialize};

/// Error from the `transcription` module namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum TranscriptionError {
    /// The request to start the transcription failed
    ServiceRequestFailed,

    /// The transcription feature is not available in this room
    FeatureDisabled,

    /// The participant has insufficient permissions to perform a command
    InsufficientPermissions,

    /// The transcription is already active in this room
    AlreadyActive,

    /// Tried to pause or stop a transcription, but it isn't active
    NotActive,

    /// The transcription service disconnected unexpectedly
    ServiceDisconnected,

    /// Internal error while generating the automatic meeting protocol
    GenerateAutomaticMeetingProtocol,
}

impl ModuleError for TranscriptionError {}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use serde_json::json;

    use crate::event::TranscriptionError;

    #[test]
    fn serialize_service_request_failed() {
        let err = TranscriptionError::ServiceRequestFailed;

        assert_json_snapshot!(err, @r#"
        {
          "error": "service_request_failed"
        }
        "#);
    }

    #[test]
    fn deserialize_service_request_failed() {
        let json = json!({"error": "service_request_failed"});
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(TranscriptionError::ServiceRequestFailed, produced);
    }

    #[test]
    fn serialize_feature_disabled() {
        let err = TranscriptionError::FeatureDisabled;

        assert_json_snapshot!(err, @r#"
        {
          "error": "feature_disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_feature_disabled() {
        let json = json!({"error": "feature_disabled"});
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(TranscriptionError::FeatureDisabled, produced);
    }

    #[test]
    fn serialize_insufficient_permissions() {
        let err = TranscriptionError::InsufficientPermissions;

        assert_json_snapshot!(err, @r#"
        {
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_insufficient_permissions() {
        let json = json!({"error": "insufficient_permissions"});
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(TranscriptionError::InsufficientPermissions, produced);
    }

    #[test]
    fn serialize_already_active() {
        let err = TranscriptionError::AlreadyActive;

        assert_json_snapshot!(err, @r#"
        {
          "error": "already_active"
        }
        "#);
    }

    #[test]
    fn deserialize_already_active() {
        let json = json!({"error": "already_active"});
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(TranscriptionError::AlreadyActive, produced);
    }

    #[test]
    fn serialize_not_active() {
        let err = TranscriptionError::NotActive;

        assert_json_snapshot!(err, @r#"
        {
          "error": "not_active"
        }
        "#);
    }

    #[test]
    fn deserialize_not_active() {
        let json = json!({"error": "not_active"});
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(TranscriptionError::NotActive, produced);
    }

    #[test]
    fn serialize_service_disconnected() {
        let err = TranscriptionError::ServiceDisconnected;

        assert_json_snapshot!(err, @r#"
        {
          "error": "service_disconnected"
        }
        "#);
    }

    #[test]
    fn deserialize_service_disconnected() {
        let json = json!({"error": "service_disconnected"});
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(TranscriptionError::ServiceDisconnected, produced);
    }

    #[test]
    fn serialize_generate_automatic_meeting_protocol() {
        let err = TranscriptionError::GenerateAutomaticMeetingProtocol;

        assert_json_snapshot!(err, @r#"
        {
          "error": "generate_automatic_meeting_protocol"
        }
        "#);
    }

    #[test]
    fn deserialize_generate_automatic_meeting_protocol() {
        let json = json!({"error": "generate_automatic_meeting_protocol"});
        let produced = serde_json::from_value(json).unwrap();

        assert_eq!(
            TranscriptionError::GenerateAutomaticMeetingProtocol,
            produced
        );
    }
}
