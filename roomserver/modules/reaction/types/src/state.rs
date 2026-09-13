// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashSet;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::{ParticipantId, SignalingModuleFrontendData};
use serde::{Deserialize, Serialize};

use crate::REACTION_MODULE_ID;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionState {
    pub restrictions: ReactionRestrictions,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReactionRestrictions {
    /// The restrictions are disabled, all participants are allowed to send reactions.
    #[default]
    Disabled,
    /// The restrictions are enabled, only the participants part of `unrestricted_participants` are
    /// allowed to send reactions.
    Enabled {
        /// The list of participants that are still allowed to send reactions.
        unrestricted_participants: HashSet<ParticipantId>,
    },
}

impl SignalingModuleFrontendData for ReactionState {
    const NAMESPACE: Option<ModuleId> = Some(REACTION_MODULE_ID);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::{ReactionRestrictions, ReactionState};

    #[test]
    fn serialize_reaction_state() {
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));

        let state = ReactionState {
            restrictions: ReactionRestrictions::Enabled {
                unrestricted_participants,
            },
        };
        let raw = serde_json::to_string_pretty(&state).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "restrictions": {
            "type": "enabled",
            "unrestricted_participants": [
              "00000000-0000-0000-0000-000000000001"
            ]
          }
        }
        "#);
    }

    #[test]
    fn deserialize_reaction_state() {
        let json = json!({
            "restrictions": {
               "type": "enabled",
               "unrestricted_participants": [
                  "00000000-0000-0000-0000-000000000001",
               ],
            },
        });

        let state: ReactionState = serde_json::from_str(&json.to_string()).unwrap();
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));
        assert_eq!(
            state,
            ReactionState {
                restrictions: ReactionRestrictions::Enabled {
                    unrestricted_participants,
                },
            }
        );
    }

    #[test]
    fn serialize_restrictions_disabled() {
        let restrictions = ReactionRestrictions::Disabled;
        let raw = serde_json::to_string_pretty(&restrictions).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "type": "disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_restrictions_disabled() {
        let json = json!({
           "type": "disabled",
        });

        let restrictions: ReactionRestrictions = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(restrictions, ReactionRestrictions::Disabled);
    }

    #[test]
    fn serialize_restrictions_enabled() {
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));

        let restrictions = ReactionRestrictions::Enabled {
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
    fn deserialize_restrictions_enabled() {
        let json = json!({
           "type": "enabled",
           "unrestricted_participants": [
              "00000000-0000-0000-0000-000000000001",
           ],
        });

        let restrictions: ReactionRestrictions = serde_json::from_str(&json.to_string()).unwrap();
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));
        assert_eq!(
            restrictions,
            ReactionRestrictions::Enabled {
                unrestricted_participants,
            }
        );
    }
}
