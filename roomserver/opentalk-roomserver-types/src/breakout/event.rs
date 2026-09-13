// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeMap;

use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::{ModuleData, ParticipantId};
use serde::{Deserialize, Serialize};

use crate::{
    breakout::{BreakoutRoom, breakout_id::BreakoutId},
    room_kind::RoomKind,
    shared_json::SharedJson,
    signaling::module_error::ModuleError,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "message")]
pub enum BreakoutEvent {
    /// Breakout rooms have started
    Started {
        /// The issuing participant
        started_by: ParticipantId,
        /// The configured breakout rooms
        rooms: Vec<BreakoutRoom>,
        /// Optional breakout expiry. When the breakout rooms expire, all participants are moved
        /// back to the main room
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expires_at: Option<Timestamp>,
        /// The optional assignment for the receiving participant. This assignment is not enforced
        /// by the roomserver
        #[serde(default, skip_serializing_if = "Option::is_none")]
        assignment: Option<BreakoutId>,
    },

    /// A participant switched between the main and/or breakout room
    ///
    /// This event is automatically triggered when breakout rooms are closed
    ParticipantSwitchedRoom {
        /// The participant that moved
        participant_id: ParticipantId,
        /// The old room of the participant.
        old_room: RoomKind,
        /// The room that the participant moved to.
        new_room: RoomKind,
        /// Module data that was attached by signaling modules containing
        /// information about the participant that joined the room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        module_data: BTreeMap<ModuleId, SharedJson>,
    },

    /// The receiver has successfully switched between rooms
    SwitchedRoom {
        /// Module data that was attached by signaling modules
        own_data: ModuleData,
        /// The old room of the participant.
        old_room: RoomKind,
        /// The room that the participant moved to.
        new_room: RoomKind,
        /// Module data for other participants in the room.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        peer_data: BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
    },

    /// A notice that the breakout rooms will close soon
    CloseNotice {
        /// The participant that issued the close command
        issued_by: ParticipantId,
        /// The time at which the breakout rooms close
        stops_at: Timestamp,
    },

    /// Breakout rooms are in the process of being closed.
    ///
    /// Is received before all participants are moved back to the main room
    Closing {
        /// The participant that issued the close command. Is `None` when the breakout rooms
        /// expired.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        issued_by: Option<ParticipantId>,
    },

    /// The breakout rooms have been closed
    Closed,

    /// A breakout error
    Error(BreakoutError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
pub enum BreakoutError {
    /// The requesting user has insufficient permission
    InsufficientPermission,
    /// A breakout configuration is already active
    AlreadyActive,
    /// The participant is already in the targeted room
    AlreadyInRoom,
    /// A provided participant id is unknown to the roomserver
    UnknownParticipant { participant_id: ParticipantId },
    /// Invalid selection of assignments when starting the breakout rooms, e.g. a participant is
    /// assigned to multiple rooms
    InvalidSelection,
    /// Too many breakout rooms were requested
    TooManyRooms,
    /// Provided an unknown BreakoutId
    UnknownBreakoutId,
    /// The breakout rooms are inactive
    BreakoutInactive,
}

impl ModuleError for BreakoutError {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use insta::assert_snapshot;
    use opentalk_types_common::time::Timestamp;
    use opentalk_types_signaling::{ModuleData, ParticipantId};

    use crate::{
        breakout::{
            BreakoutRoom,
            breakout_id::BreakoutId,
            event::{BreakoutError, BreakoutEvent},
        },
        room_kind::RoomKind,
    };

    #[test]
    fn started() {
        let val = BreakoutEvent::Started {
            started_by: ParticipantId::nil(),
            rooms: vec![
                BreakoutRoom {
                    id: BreakoutId::from(0),
                    name: "Room 1".into(),
                },
                BreakoutRoom {
                    id: BreakoutId::from(1),
                    name: "Room 2".into(),
                },
            ],
            expires_at: Some(Timestamp::unix_epoch()),
            assignment: None,
        };

        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized,
            @r#"
        {
          "message": "started",
          "started_by": "00000000-0000-0000-0000-000000000000",
          "rooms": [
            {
              "id": 0,
              "name": "Room 1"
            },
            {
              "id": 1,
              "name": "Room 2"
            }
          ],
          "expires_at": "1970-01-01T00:00:00Z"
        }
        "#
        );
    }

    #[test]
    fn participant_switched_room() {
        let val = BreakoutEvent::ParticipantSwitchedRoom {
            participant_id: ParticipantId::nil(),
            old_room: RoomKind::Main,
            new_room: RoomKind::Breakout(BreakoutId::from(1)),
            module_data: BTreeMap::new(),
        };

        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized,
            @r#"
        {
          "message": "participant_switched_room",
          "participant_id": "00000000-0000-0000-0000-000000000000",
          "old_room": {
            "kind": "main"
          },
          "new_room": {
            "kind": "breakout",
            "id": 1
          }
        }
        "#
        );
    }

    #[test]
    fn switched_room() {
        let val = BreakoutEvent::SwitchedRoom {
            own_data: ModuleData::new(),
            old_room: RoomKind::Main,
            new_room: RoomKind::Breakout(BreakoutId::from(1)),
            peer_data: Default::default(),
        };

        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized, @r#"
        {
          "message": "switched_room",
          "own_data": {},
          "old_room": {
            "kind": "main"
          },
          "new_room": {
            "kind": "breakout",
            "id": 1
          }
        }
        "#);
    }

    #[test]
    fn close_notice() {
        let val = BreakoutEvent::CloseNotice {
            issued_by: ParticipantId::nil(),
            stops_at: Timestamp::unix_epoch(),
        };

        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized,
            @r#"
        {
          "message": "close_notice",
          "issued_by": "00000000-0000-0000-0000-000000000000",
          "stops_at": "1970-01-01T00:00:00Z"
        }
        "#
        );
    }

    #[test]
    fn closing() {
        let val = BreakoutEvent::Closing {
            issued_by: Some(ParticipantId::nil()),
        };

        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized,
            @r#"
        {
          "message": "closing",
          "issued_by": "00000000-0000-0000-0000-000000000000"
        }
        "#
        );
    }

    #[test]
    fn closing_issued_by() {
        let val = BreakoutEvent::Closing { issued_by: None };

        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized,
            @r#"
        {
          "message": "closing"
        }
        "#
        );
    }

    #[test]
    fn closed() {
        let val = BreakoutEvent::Closed;

        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized,
            @r#"
        {
          "message": "closed"
        }
        "#
        );
    }

    #[test]
    fn error() {
        let val = BreakoutEvent::Error(BreakoutError::AlreadyActive);

        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized,
            @r#"
        {
          "message": "error",
          "error": "already_active"
        }
        "#
        );
    }

    #[test]
    fn unknown_participant() {
        let val = BreakoutEvent::Error(BreakoutError::UnknownParticipant {
            participant_id: ParticipantId::nil(),
        });
        let serialized = serde_json::to_string_pretty(&val).unwrap();
        assert_snapshot!(serialized,
            @r#"
        {
          "message": "error",
          "error": "unknown_participant",
          "participant_id": "00000000-0000-0000-0000-000000000000"
        }
        "#
        );
    }
}
