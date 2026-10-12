// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::transcription::RequestId;
use serde::{Deserialize, Serialize};

use crate::segment::TranscriptionSegment;

/// Signaling events from the transcription service to the roomserver
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TranscriptionServiceEvent {
    /// Transcription service has been started
    Started {
        request_id: RequestId,
        language: Option<String>,
    },

    /// Transcription service state has been updated
    Updated { language: Option<String> },

    /// Transcription service has stopped
    Stopped,

    /// Transcription segment to be sent
    Segment(TranscriptionSegment),
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::ParticipantId;

    use super::*;

    #[test]
    fn serialize_started_event() {
        let event = TranscriptionServiceEvent::Started {
            request_id: RequestId::nil(),
            language: Some("de".into()),
        };

        assert_json_snapshot!(event, @ r#"
        {
          "kind": "started",
          "request_id": "00000000-0000-0000-0000-000000000000",
          "language": "de"
        }
        "#);
    }

    #[test]
    fn serialize_segment_event() {
        let event = TranscriptionServiceEvent::Segment(TranscriptionSegment {
            participant_id: ParticipantId::nil(),
            track_id: "track1".into(),
            starts_at: Timestamp::unix_epoch(),
            ends_at: Timestamp::unix_epoch(),
            text: "Hello world".into(),
        });

        assert_json_snapshot!(event, @ r#"
        {
          "kind": "segment",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "track_id": "track1",
          "starts_at": "1970-01-01T00:00:00Z",
          "ends_at": "1970-01-01T00:00:00Z",
          "text": "Hello world"
        }
        "#);
    }
}
