// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::{modules::ModuleId, users::DisplayName};
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    connection_id::ConnectionId, disconnect_reason::DisconnectReason,
    join::join_success::JoinSuccess, room_parameters_patch::RoomParametersPatch,
    shared_json::SharedJson, signaling::module_error::SignalingModuleError,
};

/// Outgoing websocket messages in the core namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case")]
pub enum CoreEvent {
    /// Message sent to a participant on a successful join
    JoinSuccess(Box<JoinSuccess>),

    /// Broadcast message sent to all participants when a new participant has joined
    ParticipantConnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,

        /// Module specific information about the joined participant.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        peer_data: BTreeMap<ModuleId, SharedJson>,
    },

    /// Joining the room failed
    JoinBlocked { reason: JoinBlockedReason },

    /// Broadcast message sent to all participants when a participant disconnected
    ParticipantDisconnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        reason: DisconnectReason,
    },

    /// Sent to the moderator when a participant joined the waiting room
    JoinedWaitingRoom {
        /// The id of the participant
        participant_id: ParticipantId,

        /// The id of the connection used by the participant
        connection_ids: Vec<ConnectionId>,

        /// The time when the participant joined the waiting room with their first connection
        joined_at: DateTime<Utc>,

        /// The participants display name
        display_name: DisplayName,

        /// The participants avatar URL
        avatar_url: Option<String>,
    },

    /// Sent to the moderators when a participant left the waiting room
    LeftWaitingRoom(LeftWaitingRoom),

    /// Sent to participants who are placed into a waiting room
    InWaitingRoom {
        connection_id: ConnectionId,
        participant_id: ParticipantId,
    },

    /// The room is being closed, further commands will be discarded
    Closing { reason: RoomCloseReason },

    /// The room parameters have been modified
    RoomParametersChanged {
        /// The room parameters that have been modified
        change: RoomParametersPatch,
    },

    /// The storage quota has changed
    StorageQuotaChanged {
        /// The new quota
        quota: Quota,
    },

    /// An error happened when executing a `core` command
    Error(CoreError),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinBlockedReason {
    /// The participant limit for the meeting's tariff has been reached
    ParticipantLimitReached,
}

/// A participant left the waiting room
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeftWaitingRoom {
    /// The participant id for the associated participant
    pub id: ParticipantId,
    /// The connection ids that left the waiting room
    pub connection_ids: HashSet<ConnectionId>,
}

impl From<CoreError> for CoreEvent {
    fn from(value: CoreError) -> Self {
        Self::Error(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomCloseReason {
    GracefulShutdown,
    ImmediateShutdown,
    FatalError,
    TimeLimitReached,
    IdleTimeoutReached,
    RoomDeleted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum CoreError {
    /// The requested participant is not connected
    UnknownParticipant,
    /// The participant cannot enter the room because they were not accepted by a moderator yet.
    NotAccepted,
    /// The participant sent a [`CoreCommand::EnterRoom`](super::command::CoreCommand::EnterRoom),
    /// but is already in the room.
    AlreadyInRoom,
}

impl From<CoreError> for SignalingModuleError<CoreError> {
    fn from(value: CoreError) -> Self {
        Self::Module(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashSet};

    use chrono::DateTime;
    use insta::assert_snapshot;
    use opentalk_types_common::{
        events::MeetingDetails,
        features::feature_id,
        modules::module_id,
        rooms::RoomId,
        users::{DisplayName, UserInfo},
        utils::ExampleData as _,
    };
    use opentalk_types_signaling::{ModuleData, ParticipantId, Role};
    use serde_json::json;

    use super::{CoreEvent, LeftWaitingRoom};
    use crate::{
        connection_id::ConnectionId, core::CoreError, device_id::DeviceId,
        disconnect_reason::DisconnectReason, join::join_success::JoinSuccess, room_info::RoomInfo,
        shared_json::SharedJson, tariff_details::TariffDetails,
    };

    #[test]
    fn serialize_join_success() {
        let join_success = JoinSuccess {
            id: ParticipantId::nil(),
            connection_id: ConnectionId::nil(),
            device_id: DeviceId::nil(),
            connections: vec![],
            display_name: DisplayName::example_data(),
            avatar_url: None,
            role: Role::Guest,
            closes_at: None,
            tariff: Box::new(TariffDetails::example_data()),
            enabled_modules: BTreeMap::from_iter([(
                module_id!("test_module"),
                BTreeSet::from_iter([feature_id!("test_feature")]),
            )]),
            module_data: ModuleData::new(),
            participants: vec![],
            event_info: None,
            room_info: RoomInfo {
                id: RoomId::nil(),
                password: None,
                created_by: UserInfo::example_data(),
            },
            meeting_details: MeetingDetails {
                invite_code_id: None,
                call_in: None,
                streaming_links: vec![],
            },
            is_room_owner: false,
        };
        let event = CoreEvent::JoinSuccess(Box::new(join_success));
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "join_success",
          "id": "00000000-0000-0000-0000-000000000000",
          "connection_id": "00000000-0000-0000-0000-000000000000",
          "device_id": "00000000-0000-0000-0000-000000000000",
          "connections": [],
          "display_name": "Alice Adams",
          "role": "guest",
          "tariff": {
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "Starter tariff",
            "quotas": {
              "max_storage": 50000
            },
            "used_quota": {
              "max_storage": 20000
            },
            "disabled_features": [
              "recording::record"
            ]
          },
          "enabled_modules": {
            "test_module": [
              "test_feature"
            ]
          },
          "module_data": {},
          "participants": [],
          "event_info": null,
          "meeting_details": {
            "streaming_links": []
          },
          "room_info": {
            "id": "00000000-0000-0000-0000-000000000000",
            "created_by": {
              "title": "",
              "firstname": "Alice",
              "lastname": "Adams",
              "display_name": "Alice Adams",
              "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060"
            }
          },
          "is_room_owner": false
        }
        "#);
    }

    #[test]
    fn serialize_participant_connected() {
        let mut peer_join_data = BTreeMap::new();
        peer_join_data.insert(
            module_id!("test"),
            SharedJson::from(json!({
                "key": "value"
            })),
        );

        let event = CoreEvent::ParticipantConnected {
            participant_id: ParticipantId::nil(),
            connection_id: ConnectionId::nil(),
            peer_data: peer_join_data,
        };
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "participant_connected",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "connection_id": "00000000-0000-0000-0000-000000000000",
          "peer_data": {
            "test": {
              "key": "value"
            }
          }
        }
        "#);
    }

    #[test]
    fn serialize_join_blocked() {
        let produced = serde_json::to_string_pretty(&CoreEvent::JoinBlocked {
            reason: super::JoinBlockedReason::ParticipantLimitReached,
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "join_blocked",
          "reason": "participant_limit_reached"
        }
        "#);
    }

    #[test]
    fn serialize_participant_disconnected() {
        let event = CoreEvent::ParticipantDisconnected {
            participant_id: ParticipantId::nil(),
            connection_id: ConnectionId::nil(),
            reason: DisconnectReason::ConnectionLost,
        };
        let produced = serde_json::to_string_pretty(&event).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "participant_disconnected",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "connection_id": "00000000-0000-0000-0000-000000000000",
          "reason": "connection_lost"
        }
        "#);
    }

    #[test]
    fn serialize_joined_waiting_room() {
        let produced = serde_json::to_string_pretty(&CoreEvent::JoinedWaitingRoom {
            participant_id: ParticipantId::from_u128(123),
            joined_at: DateTime::UNIX_EPOCH,
            display_name: "Waiting Walter".parse().unwrap(),
            avatar_url: Some("https://example.com/avatar_url/waiting-walter".to_string()),
            connection_ids: vec![ConnectionId::from_u128(456)],
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "joined_waiting_room",
          "participant_id": "00000000-0000-0000-0000-00000000007b",
          "connection_ids": [
            "00000000-0000-0000-0000-0000000001c8"
          ],
          "joined_at": "1970-01-01T00:00:00Z",
          "display_name": "Waiting Walter",
          "avatar_url": "https://example.com/avatar_url/waiting-walter"
        }
        "#);
    }

    #[test]
    fn serialize_left_waiting_room() {
        let left_waiting_room = LeftWaitingRoom {
            id: opentalk_types_signaling::ParticipantId::from_u128(456),
            connection_ids: HashSet::from_iter([ConnectionId::from_u128(567)]),
        };

        let produced =
            serde_json::to_string_pretty(&CoreEvent::LeftWaitingRoom(left_waiting_room.clone()))
                .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "left_waiting_room",
          "id": "00000000-0000-0000-0000-0000000001c8",
          "connection_ids": [
            "00000000-0000-0000-0000-000000000237"
          ]
        }
        "#);
    }

    #[test]
    fn serialize_in_waiting_room() {
        let produced = serde_json::to_string_pretty(&CoreEvent::InWaitingRoom {
            connection_id: ConnectionId::from_u128(1),
            participant_id: ParticipantId::from_u128(2),
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "in_waiting_room",
          "connection_id": "00000000-0000-0000-0000-000000000001",
          "participant_id": "00000000-0000-0000-0000-000000000002"
        }
        "#);
    }

    #[test]
    fn serialize_closing() {
        let produced = serde_json::to_string_pretty(&CoreEvent::Closing {
            reason: super::RoomCloseReason::GracefulShutdown,
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "closing",
          "reason": "graceful_shutdown"
        }
        "#);
    }

    #[test]
    fn serialize_error() {
        let produced =
            serde_json::to_string_pretty(&CoreEvent::Error(CoreError::NotAccepted)).unwrap();

        assert_snapshot!(produced, @r#"
        {
          "message": "error",
          "error": "not_accepted"
        }
        "#);
    }
}
