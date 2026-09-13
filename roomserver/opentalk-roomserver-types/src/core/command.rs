// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::users::DisplayName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum CoreCommand {
    EnterWaitingRoom {
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<DisplayName>,
    },
    EnterRoom {
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<DisplayName>,
    },
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use insta::assert_snapshot;
    use opentalk_types_common::users::DisplayName;
    use serde_json::json;

    use crate::core::CoreCommand;

    #[test]
    fn serialize_enter_waiting_room() {
        let cmd = CoreCommand::EnterWaitingRoom {
            display_name: Some(DisplayName::from_str_lossy("Alice")),
        };

        assert_snapshot!(&serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "enter_waiting_room",
          "display_name": "Alice"
        }
        "#);
    }

    #[test]
    fn deserialize_enter_waiting_room() {
        let produced = serde_json::from_value(json!({
           "action": "enter_waiting_room",
           "display_name": "Alice",
        }))
        .unwrap();

        assert_matches!(
            produced,
            CoreCommand::EnterWaitingRoom { display_name }
                if display_name == Some(DisplayName::from_str_lossy("Alice"))
        );
    }

    #[test]
    fn serialize_enter_room() {
        let cmd = CoreCommand::EnterRoom {
            display_name: Some(DisplayName::from_str_lossy("Alice")),
        };

        assert_snapshot!(&serde_json::to_string_pretty(&cmd).unwrap(), @r#"
        {
          "action": "enter_room",
          "display_name": "Alice"
        }
        "#);
    }

    #[test]
    fn deserialize_enter_room() {
        let produced = serde_json::from_value(json!({
           "action": "enter_room",
           "display_name": "Alice",
        }))
        .unwrap();

        assert_matches!(
            produced,
            CoreCommand::EnterRoom { display_name }
                if display_name == Some(DisplayName::from_str_lossy("Alice"))
        );
    }
}
