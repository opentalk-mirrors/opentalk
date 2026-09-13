// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChangeDisplayNameRestrictionState {
    /// Participants are allowed to change their own display names
    Disabled,
    /// Only the participants in `unrestricted_participants` are allowed to change their display
    /// names
    Enabled {
        /// The list of participants that are still allowed to change their display names
        unrestricted_participants: HashSet<ParticipantId>,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use serde_json::json;

    use crate::state::ChangeDisplayNameRestrictionState;

    #[test]
    fn serialize_disabled() {
        let produced = serde_json::to_string(&ChangeDisplayNameRestrictionState::Disabled).unwrap();

        assert_snapshot!(produced, @r#"{"type":"disabled"}"#);
    }

    #[test]
    fn deserialize_disabled() {
        let produced: ChangeDisplayNameRestrictionState = serde_json::from_value(json!({
            "type": "disabled",
        }))
        .unwrap();
        let expected = ChangeDisplayNameRestrictionState::Disabled;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_enabled() {
        let produced = serde_json::to_string_pretty(&ChangeDisplayNameRestrictionState::Enabled {
            unrestricted_participants: HashSet::from_iter([ParticipantId::nil()]),
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "type": "enabled",
          "unrestricted_participants": [
            "00000000-0000-0000-0000-000000000000"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_enabled() {
        let produced: ChangeDisplayNameRestrictionState = serde_json::from_value(json!({
            "type": "enabled",
            "unrestricted_participants": ["00000000-0000-0000-0000-000000000000"]
        }))
        .unwrap();
        let expected = ChangeDisplayNameRestrictionState::Enabled {
            unrestricted_participants: HashSet::from_iter([ParticipantId::nil()]),
        };

        assert_eq!(produced, expected);
    }
}
