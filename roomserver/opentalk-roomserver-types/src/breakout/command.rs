// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::breakout_config::BreakoutConfig;
use crate::room_kind::RoomKind;

/// Incoming websocket commands to the `breakout` namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "action")]
pub enum BreakoutCommand {
    /// Start the specified amount of breakout rooms
    Start(BreakoutConfig),

    /// Move to the targeted room. Providing no BreakoutId will move the participant to the main
    /// room.
    ///
    /// Switching the room moves all active connections to that room, a participant can only be in
    /// one room at a time.
    SwitchRoom(RoomKind),

    /// Stop all breakout rooms, moving participants back to the main room.
    Stop {
        /// Delay the stop of the breakout rooms to give participants time to leave by themselves.
        ///
        /// Providing `None` or a 0 second delay will immediately stop all breakout rooms and force
        /// participants back to the main room.
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            with = "opentalk_types_common::utils::duration_seconds_option"
        )]
        delay: Option<Duration>,
    },
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use opentalk_types_signaling::ParticipantId;
    use serde_json::json;

    use crate::{
        breakout::{
            breakout_config::{BreakoutConfig, BreakoutRoomConfig},
            breakout_id::BreakoutId,
            command::BreakoutCommand,
        },
        room_kind::RoomKind,
    };

    #[test]
    fn start() {
        let val = BreakoutCommand::Start(BreakoutConfig {
            rooms: vec![
                BreakoutRoomConfig {
                    name: "Room 1".into(),
                    assignments: vec![],
                },
                BreakoutRoomConfig {
                    name: "Room 2".into(),
                    assignments: vec![],
                },
            ],
            duration: None,
        });

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "action": "start",
                "rooms": [
                    {"name": "Room 1"},
                    {"name": "Room 2"}
                ]
            }
            )
        );
    }

    #[test]
    fn start_with_assignment() {
        let val = BreakoutCommand::Start(BreakoutConfig {
            rooms: vec![
                BreakoutRoomConfig {
                    name: "Room 1".into(),
                    assignments: vec![ParticipantId::from_u128(0)],
                },
                BreakoutRoomConfig {
                    name: "Room 2".into(),
                    assignments: vec![ParticipantId::from_u128(1), ParticipantId::from_u128(2)],
                },
            ],
            duration: None,
        });

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "action": "start",
                "rooms": [
                    {"name": "Room 1", "assignments": ["00000000-0000-0000-0000-000000000000"]},
                    {"name": "Room 2", "assignments": ["00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000002"]}
                ]
            }
            )
        );
    }

    #[test]
    fn stop() {
        let val = BreakoutCommand::Stop { delay: None };

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "action": "stop"
                }
            )
        );
    }

    #[test]
    fn stop_with_delay() {
        let val = BreakoutCommand::Stop {
            delay: Some(Duration::from_secs(10)),
        };

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "action": "stop",
                "delay": 10
                }
            )
        );
    }

    #[test]
    fn switch_room() {
        let val = BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(1)));

        let json = serde_json::to_value(val).unwrap();

        assert_eq!(
            json,
            json!({
                "action": "switch_room",
                "kind": "breakout",
                "id": 1,
            })
        );
    }
}
