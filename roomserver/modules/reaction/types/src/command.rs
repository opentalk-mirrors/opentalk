// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Signaling commands for the `reactions` module

use std::collections::HashSet;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{Reaction, ReactionEvent};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ReactionCommand {
    /// Send a reaction.
    React {
        /// The emoji reaction to send.
        reaction: Reaction,
    },
    /// Enables the restrictions for reactions, allowing only the specified participants to react.
    EnableRestrictions {
        unrestricted_participants: HashSet<ParticipantId>,
    },
    /// Disables the restrictions for reactions, allowing all participants to react.
    DisableRestrictions,
}

impl CreateReplica<ReactionEvent> for ReactionCommand {
    fn replicate(&self) -> Option<ReactionEvent> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::ReactionCommand;
    use crate::Reaction;

    #[test]
    fn serialize_react() {
        let command = ReactionCommand::React {
            reaction: Reaction::ThumbsUp,
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "react",
          "reaction": "thumbs_up"
        }
        "#);
    }

    #[test]
    fn deserialize_react() {
        let json = json!({
            "action": "react",
            "reaction": "thumbs_up",
        });

        let command: ReactionCommand = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            command,
            ReactionCommand::React {
                reaction: Reaction::ThumbsUp,
            }
        );
    }

    #[test]
    fn serialize_enable_restrictions() {
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));

        let command = ReactionCommand::EnableRestrictions {
            unrestricted_participants,
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "enable_restrictions",
          "unrestricted_participants": [
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_enable_restrictions() {
        let json = json!({
            "action": "enable_restrictions",
            "unrestricted_participants": [
                "00000000-0000-0000-0000-000000000001",
            ],
        });

        let command: ReactionCommand = serde_json::from_str(&json.to_string()).unwrap();
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));
        assert_eq!(
            command,
            ReactionCommand::EnableRestrictions {
                unrestricted_participants,
            }
        );
    }

    #[test]
    fn serialize_disable_restrictions() {
        let command = ReactionCommand::DisableRestrictions;
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "disable_restrictions"
        }
        "#);
    }

    #[test]
    fn deserialize_disable_restrictions() {
        let json = json!({
            "action": "disable_restrictions",
        });

        let command: ReactionCommand = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(command, ReactionCommand::DisableRestrictions);
    }
}
