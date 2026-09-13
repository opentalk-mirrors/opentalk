// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use opentalk_types_common::streaming::StreamingTargetId;
use serde::{Deserialize, Serialize};

use crate::{RecordingStatus, StreamingTarget, service::state::RecordingServiceState};

/// The state of the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingState {
    pub consents_recording: bool,
    pub recording_state: RecordingStatus,
    pub stream_states: BTreeMap<StreamingTargetId, StreamingTarget>,
    pub service: Option<RecordingServiceState>,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for RecordingState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::RECORDING_MODULE_ID);
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{StreamErrorReason, StreamStatus, service::state::ServiceStreamingTarget};

    #[test]
    fn serialize_null_service() {
        let state = RecordingState {
            consents_recording: false,
            recording_state: RecordingStatus::Error {
                reason: StreamErrorReason {
                    code: "error_code".into(),
                    message: "error message".into(),
                },
            },
            stream_states: [(
                StreamingTargetId::from_u128(123),
                StreamingTarget {
                    name: "My Test Stream".into(),
                    public_url: "http://localhost/mystream".parse().unwrap(),
                    status: StreamStatus::Active,
                },
            )]
            .into(),
            service: None,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "consents_recording": false,
          "recording_state": {
            "status": "error",
            "reason": {
              "code": "error_code",
              "message": "error message"
            }
          },
          "stream_states": {
            "00000000-0000-0000-0000-00000000007b": {
              "name": "My Test Stream",
              "public_url": "http://localhost/mystream",
              "status": "active"
            }
          },
          "service": null
        }
        "#);
    }

    #[test]
    fn deserialize_null_service() {
        let json = json!(
            {
                "consents_recording": false,
                "recording_state": {
                    "status": "error",
                    "reason": {
                        "code": "error_code",
                        "message": "error message"
                    }
                },
                "stream_states": {
                    "00000000-0000-0000-0000-00000000007b": {
                        "name": "My Test Stream",
                        "public_url": "http://localhost/mystream",
                        "status": "active"
                    }
                },
                "service": null
            }
        );

        let produced: RecordingState = serde_json::from_value(json).unwrap();
        let expected = RecordingState {
            consents_recording: false,
            recording_state: RecordingStatus::Error {
                reason: StreamErrorReason {
                    code: "error_code".into(),
                    message: "error message".into(),
                },
            },
            stream_states: [(
                StreamingTargetId::from_u128(123),
                StreamingTarget {
                    name: "My Test Stream".into(),
                    public_url: "http://localhost/mystream".parse().unwrap(),
                    status: StreamStatus::Active,
                },
            )]
            .into(),
            service: None,
        };

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_some_service() {
        let state = RecordingState {
            consents_recording: false,
            recording_state: RecordingStatus::Active,
            stream_states: [(
                StreamingTargetId::from_u128(123),
                StreamingTarget {
                    name: "My Test Stream".into(),
                    public_url: "http://localhost/mystream".parse().unwrap(),
                    status: StreamStatus::Active,
                },
            )]
            .into(),
            service: Some(RecordingServiceState {
                streaming_targets: [(
                    StreamingTargetId::from_u128(123),
                    ServiceStreamingTarget {
                        location: "http://localhost/mystream/stream".parse().unwrap(),
                    },
                )]
                .into(),
            }),
        };

        assert_json_snapshot!(state, @ r#"
        {
          "consents_recording": false,
          "recording_state": {
            "status": "active"
          },
          "stream_states": {
            "00000000-0000-0000-0000-00000000007b": {
              "name": "My Test Stream",
              "public_url": "http://localhost/mystream",
              "status": "active"
            }
          },
          "service": {
            "streaming_targets": {
              "00000000-0000-0000-0000-00000000007b": {
                "location": "http://localhost/mystream/stream"
              }
            }
          }
        }
        "#);
    }

    #[test]
    fn deserialize_some_service() {
        let json = json!({
          "consents_recording": false,
          "recording_state": {
            "status": "active"
          },
          "stream_states": {
            "00000000-0000-0000-0000-00000000007b": {
              "name": "My Test Stream",
              "public_url": "http://localhost/mystream",
              "status": "active"
            }
          },
          "service": {
            "streaming_targets": {
              "00000000-0000-0000-0000-00000000007b": {
                "location": "http://localhost/mystream/stream"
              }
            }
          }
        });

        let produced: RecordingState = serde_json::from_value(json).unwrap();
        let expected = RecordingState {
            consents_recording: false,
            recording_state: RecordingStatus::Active,
            stream_states: [(
                StreamingTargetId::from_u128(123),
                StreamingTarget {
                    name: "My Test Stream".into(),
                    public_url: "http://localhost/mystream".parse().unwrap(),
                    status: StreamStatus::Active,
                },
            )]
            .into(),
            service: Some(RecordingServiceState {
                streaming_targets: [(
                    StreamingTargetId::from_u128(123),
                    ServiceStreamingTarget {
                        location: "http://localhost/mystream/stream".parse().unwrap(),
                    },
                )]
                .into(),
            }),
        };

        assert_eq!(produced, expected)
    }
}
