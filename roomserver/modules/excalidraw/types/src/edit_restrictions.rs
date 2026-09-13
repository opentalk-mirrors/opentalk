// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditRestrictions {
    /// The edit restrictions are disabled, all participants are allowed to edit excalidraw.
    #[default]
    Disabled,
    /// The edit restrictions are enabled, only the participants part of `unrestricted_participants`
    /// are allowed to edit excalidraw.
    Enabled {
        /// The list of participants that are still allowed to edit
        unrestricted_participants: HashSet<ParticipantId>,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::EditRestrictions;

    #[test]
    fn serialize_disabled() {
        let restrictions = EditRestrictions::Disabled;
        let raw = serde_json::to_string_pretty(&restrictions).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "type": "disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_disabled() {
        let raw = json!({
            "type": "disabled",
        });

        let restrictions: EditRestrictions = serde_json::from_str(&raw.to_string()).unwrap();
        assert_eq!(restrictions, EditRestrictions::Disabled);
    }

    #[test]
    fn serialize_enabled() {
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));

        let restrictions = EditRestrictions::Enabled {
            unrestricted_participants,
        };
        let raw = serde_json::to_string_pretty(&restrictions).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "type": "enabled",
          "unrestricted_participants": [
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_enabled() {
        let raw = json!({
            "type": "enabled",
            "unrestricted_participants": [
                "00000000-0000-0000-0000-000000000001",
            ],
        });

        let restrictions: EditRestrictions = serde_json::from_str(&raw.to_string()).unwrap();
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));
        assert_eq!(
            restrictions,
            EditRestrictions::Enabled {
                unrestricted_participants,
            }
        );
    }
}
