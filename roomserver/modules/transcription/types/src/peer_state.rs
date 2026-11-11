// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionPeerState {
    /// Flag showing whether the participant consents to being included in the automatic
    /// meeting protocol
    pub consents_automatic_meeting_protocol: bool,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_transcription_peer_state() {
        let state = TranscriptionPeerState {
            consents_automatic_meeting_protocol: true,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "consents_automatic_meeting_protocol": true
        }
        "#);
    }

    #[test]
    fn deserialize_transcription_peer_state() {
        let produced = serde_json::from_value(json!({
           "consents_automatic_meeting_protocol": false,
        }))
        .unwrap();
        let expected = TranscriptionPeerState {
            consents_automatic_meeting_protocol: false,
        };

        assert_eq!(expected, produced);
    }
}
