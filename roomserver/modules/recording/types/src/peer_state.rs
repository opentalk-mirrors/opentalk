// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// The state of other participants in the `recording` module.
///
/// This struct is sent to the participant in the `join_success` message
/// which will contain this information for each participant in the meeting.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingPeerState {
    /// Flag showing whether the participant consents to recording
    pub consents_recording: bool,
}

impl opentalk_types_signaling::SignalingModulePeerFrontendData for RecordingPeerState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::RECORDING_MODULE_ID);
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize() {
        let state = RecordingPeerState {
            consents_recording: true,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "consents_recording": true
        }
        "#);
    }

    #[test]
    fn deserialize() {
        let json = json!(
            {
                "consents_recording": true
            }
        );

        let produced: RecordingPeerState = serde_json::from_value(json).unwrap();
        let expected = RecordingPeerState {
            consents_recording: true,
        };

        assert_eq!(produced, expected)
    }
}
