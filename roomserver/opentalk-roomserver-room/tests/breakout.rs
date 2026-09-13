// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use insta::assert_snapshot;
use opentalk_roomserver_room::mocking::{
    mock_module::MockModule,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};

#[test_log::test(tokio::test)]
async fn start_breakout_rooms() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let breakout = alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![
                    BreakoutRoomConfig {
                        name: "room 1".to_string(),
                        assignments: Vec::new(),
                    },
                    BreakoutRoomConfig {
                        name: "room 2".to_string(),
                        assignments: Vec::new(),
                    },
                ],
                duration: None,
            },
        )
        .await;
    assert_snapshot!(serde_json::to_string_pretty(&breakout).unwrap(), @r#"
    {
      "message": "started",
      "started_by": "00000000-0000-0000-0000-0000000a11ce",
      "rooms": [
        {
          "id": 0,
          "name": "room 1"
        },
        {
          "id": 1,
          "name": "room 2"
        }
      ]
    }
    "#);
}

#[test_log::test(tokio::test)]
async fn switch_breakout_rooms() {
    let mut room = TestRoom::builder().register_module::<MockModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![
                    BreakoutRoomConfig {
                        name: "room 1".to_string(),
                        assignments: Vec::new(),
                    },
                    BreakoutRoomConfig {
                        name: "room 2".to_string(),
                        assignments: Vec::new(),
                    },
                ],
                duration: None,
            },
        )
        .await;

    let switch_event_alice = alice
        .switch_breakout_room(&mut [], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    assert_snapshot!(serde_json::to_string_pretty(&switch_event_alice).unwrap(), @r#"
    {
      "message": "switched_room",
      "own_data": {
        "mock": "Switched room from Main to Breakout(BreakoutId(0))"
      },
      "old_room": {
        "kind": "main"
      },
      "new_room": {
        "kind": "breakout",
        "id": 0
      },
      "peer_data": {
        "00000000-0000-0000-0000-000000000b0b": {
          "mock": "About 00000000-0000-0000-0000-000000000b0b for 00000000-0000-0000-0000-0000000a11ce"
        }
      }
    }
    "#);

    let switch_event_bob = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_snapshot!(serde_json::to_string_pretty(&switch_event_bob).unwrap(), @r#"
    {
      "message": "participant_switched_room",
      "participant_id": "00000000-0000-0000-0000-0000000a11ce",
      "old_room": {
        "kind": "main"
      },
      "new_room": {
        "kind": "breakout",
        "id": 0
      },
      "module_data": {
        "mock": "From 00000000-0000-0000-0000-0000000a11ce for 00000000-0000-0000-0000-000000000b0b"
      }
    }
    "#);
}
