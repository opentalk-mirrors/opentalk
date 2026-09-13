// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `transcription` namespace

use serde::{Deserialize, Serialize};

mod error;

pub use error::TranscriptionError;

use crate::{
    segment::TranscriptionSegment, service::command::TranscriptionServiceCommand,
    state::TranscriptionStatus,
};

/// Events for the `transcription` namespace
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum TranscriptionEvent {
    Segment(TranscriptionSegment),

    StateUpdated {
        #[serde(flatten)]
        status: TranscriptionStatus,
    },

    ServiceCommand {
        command: TranscriptionServiceCommand,
    },

    Error(TranscriptionError),
}

impl From<TranscriptionError> for TranscriptionEvent {
    fn from(value: TranscriptionError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::ParticipantId;

    use super::*;

    #[test]
    fn serialize_segment() {
        let event = TranscriptionEvent::Segment(TranscriptionSegment {
            participant_id: ParticipantId::nil(),
            track_id: "track1".into(),
            starts_at: Timestamp::unix_epoch(),
            ends_at: Timestamp::unix_epoch(),
            text: "Hello world".into(),
        });

        assert_json_snapshot!(event, @ r#"
        {
          "message": "segment",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "track_id": "track1",
          "starts_at": "1970-01-01T00:00:00Z",
          "ends_at": "1970-01-01T00:00:00Z",
          "text": "Hello world"
        }
        "#);
    }

    #[test]
    fn serialize_requested() {
        let event = TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Requested {
                language: Some("de".into()),
            },
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "state_updated",
          "status": "requested",
          "language": "de"
        }
        "#);
    }

    #[test]
    fn serialize_running() {
        let event = TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running {
                language: Some("de".into()),
            },
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "state_updated",
          "status": "running",
          "language": "de"
        }
        "#);
    }

    #[test]
    fn serialize_inactive() {
        let event = TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "state_updated",
          "status": "inactive"
        }
        "#);
    }

    #[test]
    fn serialize_service_command() {
        let event = TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "message": "service_command",
          "command": {
            "kind": "stop"
          }
        }
        "#);
    }

    #[test]
    fn serialize_error() {
        let event = TranscriptionEvent::Error(TranscriptionError::ServiceRequestFailed);

        assert_json_snapshot!(event, @ r#"
        {
          "message": "error",
          "error": "service_request_failed"
        }
        "#);
    }
}
