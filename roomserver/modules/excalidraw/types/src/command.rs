// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling commands for the `excalidraw` namespace.

use std::collections::HashSet;

use opentalk_roomserver_signaling::signaling_module::CreateReplica;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{EditRestrictions, ExcalidrawEvent};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum ExcalidrawCommand {
    /// Start a excalidraw session
    Start {
        /// The initial scene of excalidraw
        initial_scene: serde_json::Value,
        edit_restrictions: EditRestrictions,
    },
    /// Stop the excalidraw session and discard the current state
    Stop,
    /// Broadcast arbitrary excalidraw data to all participants in the room. This data is volatile,
    /// i.e. it must not change the excalidraw state. E.g., update the mouse position of the sender.
    BroadcastVolatile {
        /// Arbitrary excalidraw data
        data: serde_json::Value,
    },
    /// Broadcast arbitrary excalidraw data to all participants in the room. This data can modify
    /// the excalidraw state for all participants, however it is not persisted server-side. To
    /// persist the excalidraw state server side, use [`ExcalidrawCommand::StoreScene`].
    Broadcast {
        /// Arbitrary excalidraw data
        data: serde_json::Value,
    },
    /// Persists the scene server-side.
    StoreScene {
        /// The scene to be stored
        scene: serde_json::Value,
    },
    Follow {
        /// The id of the participant to follow
        participant_id: ParticipantId,
    },
    Unfollow {
        /// The id of the participant to unfollow
        participant_id: ParticipantId,
    },
    /// Enables the edit restriction state, where only a subset of participants are allowed to edit
    /// excalidraw. Moderators are always allowed to edit excalidraw.
    EnableEditRestrictions {
        /// The list of participants that are still allowed to edit
        unrestricted_participants: HashSet<ParticipantId>,
    },
    /// Disables the edit restriction state, all participants are allowed to edit excalidraw.
    DisableEditRestrictions,
}

impl CreateReplica<ExcalidrawEvent> for ExcalidrawCommand {
    fn replicate(&self) -> Option<ExcalidrawEvent> {
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

    use super::ExcalidrawCommand;
    use crate::EditRestrictions;

    #[test]
    fn serialize_start() {
        let command = ExcalidrawCommand::Start {
            initial_scene: json!({"some": "scene"}),
            edit_restrictions: EditRestrictions::Disabled,
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "start",
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
    fn deserialize_start() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "start",
            "initial_scene": {
                "some": "scene",
            },
            "edit_restrictions": {
                "type": "disabled",
            },
        }))
        .unwrap();

        assert_eq!(
            command,
            ExcalidrawCommand::Start {
                initial_scene: json!({"some": "scene"}),
                edit_restrictions: EditRestrictions::Disabled,
            }
        );
    }

    #[test]
    fn serialize_stop() {
        let command = ExcalidrawCommand::Stop;
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "stop"
        }
        "#);
    }

    #[test]
    fn deserialize_stop() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "stop",
        }))
        .expect("Failed to deserialize");

        assert_eq!(command, ExcalidrawCommand::Stop);
    }

    #[test]
    fn serialize_broadcast_volatile() {
        let command = ExcalidrawCommand::BroadcastVolatile {
            data: json!({"some": "data"}),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "broadcast_volatile",
          "data": {
            "some": "data"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_broadcast_volatile() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "broadcast_volatile",
            "data": {"some": "data"},
        }))
        .unwrap();

        assert_eq!(
            command,
            ExcalidrawCommand::BroadcastVolatile {
                data: json!({"some": "data"}),
            }
        );
    }

    #[test]
    fn serialize_broadcast() {
        let command = ExcalidrawCommand::Broadcast {
            data: json!({"some": "data"}),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "broadcast",
          "data": {
            "some": "data"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_broadcast() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "broadcast",
            "data": {"some": "data"},
        }))
        .unwrap();

        assert_eq!(
            command,
            ExcalidrawCommand::Broadcast {
                data: json!({"some": "data"}),
            }
        );
    }

    #[test]
    fn serialize_store_scene() {
        let command = ExcalidrawCommand::StoreScene {
            scene: json!({"some": "scene"}),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "store_scene",
          "scene": {
            "some": "scene"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_store_scene() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "store_scene",
            "scene": {"some": "scene"},
        }))
        .unwrap();

        assert_eq!(
            command,
            ExcalidrawCommand::StoreScene {
                scene: json!({"some": "scene"}),
            }
        );
    }

    #[test]
    fn serialize_follow() {
        let command = ExcalidrawCommand::Follow {
            participant_id: ParticipantId::nil(),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "follow",
          "participant_id": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_follow() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "follow",
            "participant_id": "00000000-0000-0000-0000-000000000000",
        }))
        .unwrap();

        assert_eq!(
            command,
            ExcalidrawCommand::Follow {
                participant_id: ParticipantId::nil(),
            }
        );
    }

    #[test]
    fn serialize_unfollow() {
        let command = ExcalidrawCommand::Unfollow {
            participant_id: ParticipantId::nil(),
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "unfollow",
          "participant_id": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }

    #[test]
    fn deserialize_unfollow() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "unfollow",
            "participant_id": "00000000-0000-0000-0000-000000000000",
        }))
        .unwrap();

        assert_eq!(
            command,
            ExcalidrawCommand::Unfollow {
                participant_id: ParticipantId::nil(),
            }
        );
    }

    #[test]
    fn serialize_enable_edit_restrictions() {
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));

        let command = ExcalidrawCommand::EnableEditRestrictions {
            unrestricted_participants,
        };
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "enable_edit_restrictions",
          "unrestricted_participants": [
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_enable_edit_restrictions() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "enable_edit_restrictions",
            "unrestricted_participants": [
                "00000000-0000-0000-0000-000000000001",
            ],
        }))
        .unwrap();
        let mut unrestricted_participants = HashSet::new();
        unrestricted_participants.insert(ParticipantId::from_u128(1));

        assert_eq!(
            command,
            ExcalidrawCommand::EnableEditRestrictions {
                unrestricted_participants,
            }
        );
    }

    #[test]
    fn serialize_disable_edit_restrictions() {
        let command = ExcalidrawCommand::DisableEditRestrictions;
        let raw = serde_json::to_string_pretty(&command).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "action": "disable_edit_restrictions"
        }
        "#);
    }

    #[test]
    fn deserialize_disable_edit_restrictions() {
        let command: ExcalidrawCommand = serde_json::from_value(json!({
            "action": "disable_edit_restrictions",
        }))
        .unwrap();

        assert_eq!(command, ExcalidrawCommand::DisableEditRestrictions);
    }
}
