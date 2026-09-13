// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `recording` namespace and used by recording services

use opentalk_types_common::streaming::StreamingTargetId;
use serde::{Deserialize, Serialize};

use crate::{RecordingStatus, StreamStatus};

/// Event sent by the recording service to the roomserver
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecordingServiceEvent {
    /// The recording service notifies that the status of the recording for the current room has
    /// changed
    RecordingUpdated(RecordingStatus),

    /// The recording service notifies that the status of the specified stream has changed
    StreamUpdated {
        /// The stream id which has been updated
        target_id: StreamingTargetId,
        /// The new status
        #[serde(flatten)]
        status: StreamStatus,
    },
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_recording_updated() {
        let event = RecordingServiceEvent::RecordingUpdated(RecordingStatus::Active);

        assert_json_snapshot!(event, @ r#"
        {
          "kind": "recording_updated",
          "status": "active"
        }
        "#);
    }

    #[test]
    fn deserialize_recording_updated() {
        let json = json!(
            {
                "kind": "recording_updated",
                "status": "active"
            }
        );

        let produced: RecordingServiceEvent = serde_json::from_value(json).unwrap();
        let expected = RecordingServiceEvent::RecordingUpdated(RecordingStatus::Active);

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_stream_updated() {
        let event = RecordingServiceEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::Active,
        };

        assert_json_snapshot!(event, @ r#"
        {
          "kind": "stream_updated",
          "target_id": "00000000-0000-0000-0000-000000000000",
          "status": "active"
        }
        "#);
    }

    #[test]
    fn deserialize_stream_updated() {
        let json = json!(
            {
                "kind": "stream_updated",
                "target_id": "00000000-0000-0000-0000-000000000000",
                "status": "active"
            }
        );

        let produced: RecordingServiceEvent = serde_json::from_value(json).unwrap();
        let expected = RecordingServiceEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::Active,
        };

        assert_eq!(produced, expected)
    }
}
