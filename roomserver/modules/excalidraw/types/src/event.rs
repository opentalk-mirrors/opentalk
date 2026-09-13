// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `excalidraw` namespace.

use std::collections::HashSet;

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{EditRestrictions, ExcalidrawError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum ExcalidrawEvent {
    /// An excalidraw session has been started
    Started {
        /// The initial scene of excalidraw
        initial_scene: serde_json::Value,
        edit_restrictions: EditRestrictions,
    },
    /// The excalidraw session ended
    Stopped,
    /// A volatile update has been broadcasted
    VolatileBroadcast {
        /// The id of the participant that sent the update
        sender: ParticipantId,
        /// Arbitrary excalidraw data
        data: serde_json::Value,
    },
    /// An excalidraw update has been broadcasted
    Broadcast {
        /// The id of the participant that sent the update
        sender: ParticipantId,
        /// Arbitrary excalidraw data
        data: serde_json::Value,
    },
    /// The scene has been updated and persisted server-side.
    SceneStored { scene: serde_json::Value },
    /// Another participant has started following this participant
    FollowerGained {
        /// The id of the participant that started following
        participant_id: ParticipantId,
    },
    /// The participant has started following another participant
    Followed {
        /// The id of the participant that is being followed
        participant_id: ParticipantId,
    },
    /// Another participant has stopped following this participant
    FollowerLost {
        /// The id of the participant that stopped following
        participant_id: ParticipantId,
    },
    /// The participant has stopped following another participant
    Unfollowed {
        /// The id of the participant that is being unfollowed
        participant_id: ParticipantId,
    },
    /// The edit restrictions have been enabled, only a subset of participants are allowed to edit
    /// excalidraw. Moderators are are always allowed to edit excalidraw.
    EditRestrictionsEnabled {
        /// The list of participants that are still allowed to edit
        unrestricted_participants: HashSet<ParticipantId>,
    },
    /// The edit restrictions have been disabled, all participants are allowed to edit excalidraw.
    EditRestrictionsDisabled,
    /// An error happened when executing an `excalidraw` command
    Error(ExcalidrawError),
}

impl From<ExcalidrawError> for ExcalidrawEvent {
    fn from(value: ExcalidrawError) -> Self {
        Self::Error(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use insta::assert_snapshot;
    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::ExcalidrawEvent;
    use crate::{EditRestrictions, ExcalidrawError};

    #[test]
    fn serialize_started() {
        let event = ExcalidrawEvent::Started {
            initial_scene: json!({"some": "scene"}),
            edit_restrictions: EditRestrictions::Disabled,
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "started",
          "initial_scene": {
            "some": "scene"
          },
          "edit_restrictions": {
            "type": "disabled"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_started() {
        let json = json!({
            "message": "started",
            "initial_scene": {
                "some": "scene",
            },
            "edit_restrictions": {
                "type": "disabled",
            },
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::Started {
                initial_scene: json!({"some": "scene"}),
                edit_restrictions: EditRestrictions::Disabled,
            }
        );
    }

    #[test]
    fn serialize_volatile_broadcast() {
        let event = ExcalidrawEvent::VolatileBroadcast {
            sender: ParticipantId::nil(),
            data: json!({"some": "data"}),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "volatile_broadcast",
          "sender": "00000000-0000-0000-0000-000000000000",
          "data": {
            "some": "data"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_volatile_broadcast() {
        let json = json!({
            "message": "volatile_broadcast",
            "sender": "00000000-0000-0000-0000-000000000000",
            "data": {"some": "data"},
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::VolatileBroadcast {
                sender: ParticipantId::nil(),
                data: json!({"some": "data"}),
            }
        );
    }

    #[test]
    fn serialize_broadcast() {
        let event = ExcalidrawEvent::Broadcast {
            sender: ParticipantId::nil(),
            data: json!({"some": "data"}),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "broadcast",
          "sender": "00000000-0000-0000-0000-000000000000",
          "data": {
            "some": "data"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_broadcast() {
        let json = json!({
            "message": "broadcast",
            "sender": "00000000-0000-0000-0000-000000000000",
            "data": {"some": "data"},
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::Broadcast {
                sender: ParticipantId::nil(),
                data: json!({"some": "data"}),
            }
        );
    }

    #[test]
    fn serialize_scene_stored() {
        let event = ExcalidrawEvent::SceneStored {
            scene: json!({"some": "scene"}),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "scene_stored",
          "scene": {
            "some": "scene"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_scene_stored() {
        let json = json!({
            "message": "scene_stored",
            "scene": {"some": "scene"},
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::SceneStored {
                scene: json!({"some": "scene"}),
            }
        );
    }

    #[test]
    fn serialize_follower_gained() {
        let event = ExcalidrawEvent::FollowerGained {
            participant_id: ParticipantId::nil(),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "follower_gained",
          "participant_id": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_follower_gained() {
        let json = json!({
            "message": "follower_gained",
            "participant_id": "00000000-0000-0000-0000-000000000000",
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::FollowerGained {
                participant_id: ParticipantId::nil(),
            }
        );
    }

    #[test]
    fn serialize_followed() {
        let event = ExcalidrawEvent::Followed {
            participant_id: ParticipantId::nil(),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "followed",
          "participant_id": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_followed() {
        let json = json!({
            "message": "followed",
            "participant_id": "00000000-0000-0000-0000-000000000000",
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::Followed {
                participant_id: ParticipantId::nil(),
            }
        );
    }

    #[test]
    fn serialize_follower_lost() {
        let event = ExcalidrawEvent::FollowerLost {
            participant_id: ParticipantId::nil(),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "follower_lost",
          "participant_id": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_follower_lost() {
        let json = json!({
            "message": "follower_lost",
            "participant_id": "00000000-0000-0000-0000-000000000000",
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::FollowerLost {
                participant_id: ParticipantId::nil(),
            }
        );
    }

    #[test]
    fn serialize_unfollowed() {
        let event = ExcalidrawEvent::Unfollowed {
            participant_id: ParticipantId::nil(),
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "unfollowed",
          "participant_id": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_unfollowed() {
        let json = json!({
            "message": "unfollowed",
            "participant_id": "00000000-0000-0000-0000-000000000000",
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::Unfollowed {
                participant_id: ParticipantId::nil(),
            }
        );
    }

    #[test]
    fn serialize_edit_restrictions_enabled() {
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));

        let event = ExcalidrawEvent::EditRestrictionsEnabled {
            unrestricted_participants,
        };
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "edit_restrictions_enabled",
          "unrestricted_participants": [
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_edit_restrictions_enabled() {
        let json = json!({
            "message": "edit_restrictions_enabled",
            "unrestricted_participants": [
                "00000000-0000-0000-0000-000000000001",
            ],
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));
        assert_eq!(
            event,
            ExcalidrawEvent::EditRestrictionsEnabled {
                unrestricted_participants,
            }
        );
    }

    #[test]
    fn serialize_edit_restrictions_disabled() {
        let event = ExcalidrawEvent::EditRestrictionsDisabled;
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "edit_restrictions_disabled"
        }
        "#);
    }

    #[test]
    fn deserialize_edit_restrictions_disabled() {
        let json = json!({
            "message": "edit_restrictions_disabled",
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(event, ExcalidrawEvent::EditRestrictionsDisabled);
    }

    #[test]
    fn serialize_error() {
        let event = ExcalidrawEvent::Error(ExcalidrawError::InsufficientPermissions);
        let raw = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "message": "error",
          "error": "insufficient_permissions"
        }
        "#);
    }

    #[test]
    fn deserialize_error() {
        let json = json!({
            "message": "error",
            "error": "insufficient_permissions",
        });

        let event: ExcalidrawEvent = serde_json::from_str(&json.to_string()).unwrap();
        assert_eq!(
            event,
            ExcalidrawEvent::Error(ExcalidrawError::InsufficientPermissions)
        );
    }
}
