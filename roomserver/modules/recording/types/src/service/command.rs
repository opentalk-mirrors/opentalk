// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `recording_service` namespace

use std::collections::BTreeSet;

use opentalk_types_common::streaming::StreamingTargetId;
use serde::{Deserialize, Serialize};

/// Commands sent by the roomserver to the recording service
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecordingServiceCommand {
    StartRecording,
    PauseRecording,
    StopRecording,

    /// Start Streams
    StartStreams {
        /// The ids of the streams that should be started.
        target_ids: BTreeSet<StreamingTargetId>,
    },
    /// Pause Streams
    PauseStreams {
        /// The ids of the streams that should be paused.
        target_ids: BTreeSet<StreamingTargetId>,
    },
    /// Stop Streams
    StopStreams {
        /// The ids of the streams that should be stopped.
        target_ids: BTreeSet<StreamingTargetId>,
    },
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_start_recording() {
        let command = RecordingServiceCommand::StartRecording;

        assert_json_snapshot!(command, @ r#"
        {
          "kind": "start_recording"
        }
        "#);
    }

    #[test]
    fn deserialize_start_recording() {
        let json = json!(
            {
                "kind": "start_recording"
            }
        );

        let produced: RecordingServiceCommand = serde_json::from_value(json).unwrap();
        let expected = RecordingServiceCommand::StartRecording;

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_start_streams() {
        let command = RecordingServiceCommand::StartStreams {
            target_ids: [
                StreamingTargetId::from_u128(1),
                StreamingTargetId::from_u128(2),
                StreamingTargetId::from_u128(3),
            ]
            .into(),
        };

        assert_json_snapshot!(command, @ r#"
        {
          "kind": "start_streams",
          "target_ids": [
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-000000000002",
            "00000000-0000-0000-0000-000000000003"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_start_streams() {
        let json = json!(
            {
                "kind": "start_streams",
                "target_ids": [
                    "00000000-0000-0000-0000-000000000001",
                    "00000000-0000-0000-0000-000000000002",
                    "00000000-0000-0000-0000-000000000003"
                ]
            }
        );

        let produced: RecordingServiceCommand = serde_json::from_value(json).unwrap();
        let expected = RecordingServiceCommand::StartStreams {
            target_ids: [
                StreamingTargetId::from_u128(1),
                StreamingTargetId::from_u128(2),
                StreamingTargetId::from_u128(3),
            ]
            .into(),
        };

        assert_eq!(produced, expected)
    }
}
