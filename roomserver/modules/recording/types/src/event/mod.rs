// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `recording` namespace

use opentalk_types_common::streaming::StreamingTargetId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{RecordingStatus, StreamStatus, service::command::RecordingServiceCommand};

mod error;

pub use error::RecordingError;

/// Events sent out by the `recording` module
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum RecordingEvent {
    /// Stream has an update
    RecordingUpdated(RecordingStatus),

    /// The specified streaming target has received a status update
    StreamUpdated {
        /// The stream id which has been updated
        target_id: StreamingTargetId,
        /// The new status
        #[serde(flatten)]
        status: StreamStatus,
    },

    /// A participant updated its recording consent
    ConsentUpdated {
        participant: ParticipantId,
        consents: bool,
    },

    /// Event for recorder services, will not be received by non-recorders
    Service { event: RecordingServiceCommand },

    /// An error happened when executing a `recording` command
    Error(RecordingError),
}

impl From<RecordingError> for RecordingEvent {
    fn from(value: RecordingError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_recording_updated() {
        let command = RecordingEvent::RecordingUpdated(RecordingStatus::Active);

        assert_json_snapshot!(command, @ r#"
        {
          "message": "recording_updated",
          "status": "active"
        }
        "#);
    }

    #[test]
    fn deserialize_recording_updated() {
        let json = json!(
            {
                "message": "recording_updated",
                "status": "active"
            }
        );

        let produced: RecordingEvent = serde_json::from_value(json).unwrap();
        let expected = RecordingEvent::RecordingUpdated(RecordingStatus::Active);

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_stream_updated() {
        let event = RecordingEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::Active,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "stream_updated",
          "target_id": "00000000-0000-0000-0000-000000000000",
          "status": "active"
        }
        "#);
    }

    #[test]
    fn deserialize_stream_updated() {
        let json = json!(
            {
                "message": "stream_updated",
                "target_id": "00000000-0000-0000-0000-000000000000",
                "status": "active"
            }
        );

        let produced: RecordingEvent = serde_json::from_value(json).unwrap();
        let expected = RecordingEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::Active,
        };

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_consent_updated() {
        let event = RecordingEvent::ConsentUpdated {
            participant: ParticipantId::from_u128(1),
            consents: true,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "consent_updated",
          "participant": "00000000-0000-0000-0000-000000000001",
          "consents": true
        }
        "#);
    }

    #[test]
    fn deserialize_consent_updated() {
        let json = json!(
            {
                "message": "consent_updated",
                "participant": "00000000-0000-0000-0000-000000000001",
                "consents": true
            }
        );

        let produced: RecordingEvent = serde_json::from_value(json).unwrap();
        let expected = RecordingEvent::ConsentUpdated {
            participant: ParticipantId::from_u128(1),
            consents: true,
        };

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_service_command() {
        let event = RecordingEvent::Service {
            event: RecordingServiceCommand::StopRecording,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "service",
          "event": {
            "kind": "stop_recording"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_service_command() {
        let json = json!(
            {
                "message": "service",
                "event": {
                    "kind": "stop_recording"
                }
            }
        );

        let produced: RecordingEvent = serde_json::from_value(json).unwrap();
        let expected = RecordingEvent::Service {
            event: RecordingServiceCommand::StopRecording,
        };

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_error() {
        let event = RecordingEvent::Error(RecordingError::InsufficientPermissions);

        assert_json_snapshot!(event, @ r#"
        {
          "message": "error",
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_error() {
        let json = json!(
            {
                "message": "error",
                "error": "insufficient_permissions"
            }
        );

        let produced: RecordingEvent = serde_json::from_value(json).unwrap();
        let expected = RecordingEvent::Error(RecordingError::InsufficientPermissions);

        assert_eq!(produced, expected)
    }
}
