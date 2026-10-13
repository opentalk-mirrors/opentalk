// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingProtocolConfig {
    /// If `true` speaker names will be pseudonymized in the meeting protocol.
    pub pseudonymous: bool,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_meeting_protocol_config() {
        let config = MeetingProtocolConfig { pseudonymous: true };

        assert_json_snapshot!(config, @ r#"
        {
          "pseudonymous": true
        }
        "#);
    }

    #[test]
    fn deserialize_meeting_protocol_config() {
        let produced = serde_json::from_value(json!({
           "pseudonymous": false,
        }))
        .unwrap();
        let expected = MeetingProtocolConfig {
            pseudonymous: false,
        };

        assert_eq!(expected, produced);
    }
}
