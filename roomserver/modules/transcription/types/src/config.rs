// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutomaticTranscriptionConfig {
    /// If `true` speaker names will be pseudonymized in the automatic transcription.
    pub pseudonymous: bool,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_automatic_transcription_config() {
        let config = AutomaticTranscriptionConfig { pseudonymous: true };

        assert_json_snapshot!(config, @ r#"
        {
          "pseudonymous": true
        }
        "#);
    }

    #[test]
    fn deserialize_automatic_transcription_config() {
        let produced = serde_json::from_value(json!({
           "pseudonymous": false,
        }))
        .unwrap();
        let expected = AutomaticTranscriptionConfig {
            pseudonymous: false,
        };

        assert_eq!(expected, produced);
    }
}
