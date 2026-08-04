// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

/// Commands sent by the roomserver to the transcription service
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TranscriptionServiceCommand {
    ChangeLanguage {
        language: Option<String>,
    },

    /// Stop the transcription service
    Stop,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn serialize_stop_command() {
        let command = TranscriptionServiceCommand::Stop;

        assert_json_snapshot!(command, @ r#"
        {
          "kind": "stop"
        }
        "#);
    }
}
