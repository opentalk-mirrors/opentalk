// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptionPeerState {
    /// Flag showing whether the participant consents to being included in the automatic
    /// transcription
    pub consents_automatic_transcription: bool,
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
            consents_automatic_transcription: true,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "consents_automatic_transcription": true
        }
        "#);
    }

    #[test]
    fn deserialize_transcription_peer_state() {
        let produced = serde_json::from_value(json!({
           "consents_automatic_transcription": false,
        }))
        .unwrap();
        let expected = TranscriptionPeerState {
            consents_automatic_transcription: false,
        };

        assert_eq!(expected, produced);
    }
}
