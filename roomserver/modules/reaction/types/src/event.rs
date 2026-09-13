// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashSet;

use opentalk_roomserver_types::signaling::module_error::ModuleError;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::Reaction;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ReactionEvent {
    Reacted {
        /// The participant that sent the reaction.
        participant_id: ParticipantId,

        /// The reaction that was sent.
        reaction: Reaction,
    },
    /// Reactions have been restricted. Only participants listed in `unrestricted_participants` are
    /// allowed to react.
    RestrictionsEnabled {
        /// The list of participants that are allowed to react.
        unrestricted_participants: HashSet<ParticipantId>,
    },
    /// Restrictions have been disabled. All participants are allowed to react.
    RestrictionsDisabled,
    /// An error happened when executing a `reaction` command
    Error(ReactionError),
}

impl From<ReactionError> for ReactionEvent {
    fn from(value: ReactionError) -> Self {
        Self::Error(value)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ReactionError {
    /// The participant doesn't have the necessary permissions to execute the command.
    InsufficientPermissions,
    /// Restrictions are enabled, preventing the participant from reacting.
    Restricted,
}

impl ModuleError for ReactionError {}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::{ReactionError, ReactionEvent};
    use crate::Reaction;

    #[test]
    fn serialize_reacted() {
        let event = ReactionEvent::Reacted {
            participant_id: ParticipantId::nil(),
            reaction: Reaction::ThumbsUp,
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "reacted",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "reaction": "thumbs_up"
        }
        "#);
    }

    #[test]
    fn deserialize_reacted() {
        let json = json!({
            "message": "reacted",
            "participant_id": "00000000-0000-0000-0000-000000000000",
            "reaction": "thumbs_up",
        });

        let event: ReactionEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ReactionEvent::Reacted {
                participant_id: ParticipantId::nil(),
                reaction: Reaction::ThumbsUp,
            }
        );
    }

    #[test]
    fn serialize_restrictions_enabled() {
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));

        let event = ReactionEvent::RestrictionsEnabled {
            unrestricted_participants,
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "restrictions_enabled",
          "unrestricted_participants": [
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_restrictions_enabled() {
        let json = json!({
            "message": "restrictions_enabled",
            "unrestricted_participants": [
                "00000000-0000-0000-0000-000000000001",
            ],
        });

        let event: ReactionEvent = serde_json::from_str(&json.to_string()).unwrap();
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));
        assert_eq!(
            event,
            ReactionEvent::RestrictionsEnabled {
                unrestricted_participants,
            }
        );
    }

    #[test]
    fn serialize_restrictions_disabled() {
        let event = ReactionEvent::RestrictionsDisabled;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "restrictions_disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_restrictions_disabled() {
        let json = json!({
            "message": "restrictions_disabled",
        });

        let event: ReactionEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(event, ReactionEvent::RestrictionsDisabled);
    }

    #[test]
    fn serialize_error_insufficient_permissions() {
        let event = ReactionEvent::Error(ReactionError::InsufficientPermissions);
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "error",
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_error_insufficient_permissions() {
        let json = json!({
            "message": "error",
            "error": "insufficient_permissions",
        });

        let event: ReactionEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ReactionEvent::Error(ReactionError::InsufficientPermissions)
        );
    }

    #[test]
    fn serialize_error_restricted() {
        let event = ReactionEvent::Error(ReactionError::Restricted);
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "error",
          "error": "restricted"
        }
        "#);
    }

    #[test]
    fn deserialize_error_restricted() {
        let json = json!({
            "message": "error",
            "error": "restricted",
        });

        let event: ReactionEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(event, ReactionEvent::Error(ReactionError::Restricted));
    }
}
