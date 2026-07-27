// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// The current status of a recording
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum RecordingStatus {
    /// The recording has been requested to be started by the recorder
    Requested,

    /// The recording is inactive
    Inactive,

    /// The recording is active
    Active,

    /// The recording is paused
    Paused,

    /// The recording has returned an error
    Error {
        /// The error reason
        reason: StreamErrorReason,
    },
}

impl RecordingStatus {
    /// Returns true if the status is either `active` or `paused`
    pub fn is_running(&self) -> bool {
        match self {
            RecordingStatus::Active | RecordingStatus::Paused => true,
            RecordingStatus::Requested
            | RecordingStatus::Inactive
            | RecordingStatus::Error { .. } => false,
        }
    }

    /// Returns `true` if the status is [`Requested`]
    ///
    /// [`Requested`]: RecordingStatus::Requested
    pub fn is_requested(&self) -> bool {
        *self == RecordingStatus::Requested
    }

    /// Returns if the recording can be started
    pub fn can_be_started(&self) -> bool {
        match self {
            RecordingStatus::Inactive | RecordingStatus::Error { .. } => true,
            RecordingStatus::Requested | RecordingStatus::Active | RecordingStatus::Paused => false,
        }
    }
}

/// The current status of a stream
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum StreamStatus {
    /// The stream is used by a different (breakout) room
    InUse,

    /// The stream has been requested to be started by the recorder
    Requested,

    /// The stream is inactive
    Inactive,

    /// The stream is active
    Active,

    /// The stream is paused
    Paused,

    /// The stream has returned an error
    Error {
        /// The error reason
        reason: StreamErrorReason,
    },
}

impl StreamStatus {
    /// Returns true if the status shows that the stream is currently active in the current room
    pub fn is_running(&self) -> bool {
        match self {
            StreamStatus::Active | StreamStatus::Paused => true,
            StreamStatus::InUse
            | StreamStatus::Requested
            | StreamStatus::Inactive
            | StreamStatus::Error { .. } => false,
        }
    }

    /// Returns if the stream can be started from the current room
    pub fn can_be_started(&self) -> bool {
        match self {
            StreamStatus::Inactive | StreamStatus::Error { .. } => true,
            StreamStatus::InUse
            | StreamStatus::Requested
            | StreamStatus::Active
            | StreamStatus::Paused => false,
        }
    }
}

/// An error signal from the Recorder
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamErrorReason {
    /// The error that happened
    pub code: String,
    /// The reason as to why that error happened
    pub message: String,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_status_active() {
        let state = StreamStatus::Active;

        assert_json_snapshot!(state, @ r#"
        {
          "status": "active"
        }
        "#);
    }

    #[test]
    fn deserialize_status_active() {
        let json = json!(
            {
                "status": "active"
            }
        );

        let produced: StreamStatus = serde_json::from_value(json).unwrap();
        let expected = StreamStatus::Active;

        assert_eq!(produced, expected)
    }

    #[test]
    fn serialize_status_error() {
        let state = StreamStatus::Error {
            reason: StreamErrorReason {
                code: "error_code".into(),
                message: "error message".into(),
            },
        };

        assert_json_snapshot!(state, @ r#"
        {
          "status": "error",
          "reason": {
            "code": "error_code",
            "message": "error message"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_status_error() {
        let json = json!(
            {
                "status": "error",
                "reason": {
                    "code": "error_code",
                    "message": "error message"
                }
            }
        );

        let produced: StreamStatus = serde_json::from_value(json).unwrap();
        let expected = StreamStatus::Error {
            reason: StreamErrorReason {
                code: "error_code".into(),
                message: "error message".into(),
            },
        };

        assert_eq!(produced, expected)
    }
}
