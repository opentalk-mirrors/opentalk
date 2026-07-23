// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

use opentalk_roomserver_module_whiteboard::{WhiteboardModule, state::WhiteboardState};
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};
use opentalk_roomserver_types_whiteboard::{
    EditRestrictions, WhiteboardCommand, WhiteboardError, WhiteboardEvent,
};
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;
use serde_json::json;

async fn start_whiteboard(
    sender: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    let initial_scene = json!({
        "some": "scene",
    });
    let edit_restrictions = EditRestrictions::Disabled;

    sender
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Start {
                initial_scene: initial_scene.clone(),
                edit_restrictions: edit_restrictions.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let event = sender
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Started {
            initial_scene: initial_scene.clone(),
            edit_restrictions: edit_restrictions.clone(),
        }
    );

    for participant in others {
        let event = participant
            .receive_event::<WhiteboardModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(
            event,
            WhiteboardEvent::Started {
                initial_scene: initial_scene.clone(),
                edit_restrictions: edit_restrictions.clone(),
            }
        );
    }
}

#[test_log::test(tokio::test)]
async fn already_started() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut []).await;

    // Alice tries to start the whiteboard again
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Start {
                initial_scene: json!({
                    "some": "scene",
                }),
                edit_restrictions: EditRestrictions::Disabled,
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the AlreadyStarted error
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::AlreadyStarted)
    );
}

#[test_log::test(tokio::test)]
async fn start_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to start the whiteboard
    bob.send_command::<WhiteboardModule>(
        WhiteboardCommand::Start {
            initial_scene: json!({
                "some": "scene",
            }),
            edit_restrictions: EditRestrictions::Disabled,
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives the InsufficientPermissions error
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn stop_whiteboard() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut []).await;

    // Alice stops the whiteboard
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Stop, None)
        .await
        .unwrap();

    // Alice receives the Stopped event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::Stopped);
}

#[test_log::test(tokio::test)]
async fn stop_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to start the whiteboard
    bob.send_command::<WhiteboardModule>(WhiteboardCommand::Stop, None)
        .await
        .unwrap();

    // Bob receives the InsufficientPermissions error
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn join_success() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut []).await;

    // Bob joins the room. His JoinSuccess message contains the current whiteboard state
    let state = room
        .join_bob(0)
        .await
        .join_success()
        .get_module::<WhiteboardState>()
        .expect("Whiteboard state must be deserializable")
        .expect("Whiteboard state must be present");
    assert_eq!(
        state,
        WhiteboardState {
            scene: json!({"some":"scene"}),
            edit_restrictions: EditRestrictions::Disabled,
        }
    );
}

#[test_log::test(tokio::test)]
async fn broadcast() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut [&mut bob]).await;

    // Alice sends a broadcast command
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Broadcast {
                data: json!({
                    "some": "data",
                }),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the broadcast event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Broadcast {
            sender: alice.id(),
            data: json!({"some": "data"})
        }
    );

    // Bob receives the broadcast event
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Broadcast {
            sender: alice.id(),
            data: json!({"some": "data"})
        }
    );
}

#[test_log::test(tokio::test)]
async fn broadcast_not_started() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice sends a broadcast command without starting the whiteboard first
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Broadcast {
                data: json!({
                    "some": "data",
                }),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the NotStarted error
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::Error(WhiteboardError::NotStarted));
}

#[test_log::test(tokio::test)]
async fn broadcast_volatile() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut [&mut bob]).await;

    // Alice sends a volatile broadcast command
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::BroadcastVolatile {
                data: json!({
                    "some": "data",
                }),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the volatile broadcast event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::VolatileBroadcast {
            sender: alice.id(),
            data: json!({"some": "data"})
        }
    );

    // Bob receives the volatile broadcast event
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::VolatileBroadcast {
            sender: alice.id(),
            data: json!({"some": "data"})
        }
    );
}

#[test_log::test(tokio::test)]
async fn broadcast_volatile_not_started() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice sends a volatile broadcast command without starting the whiteboard first
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::BroadcastVolatile {
                data: json!({
                    "some": "data",
                }),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the NotStarted error
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::Error(WhiteboardError::NotStarted));
}

#[test_log::test(tokio::test)]
async fn follow_not_started() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to follow Bob without starting the whiteboard first
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Follow {
                participant_id: ParticipantId::from_u128(0x12345678),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the NotStarted error
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::Error(WhiteboardError::NotStarted));
}

#[test_log::test(tokio::test)]
async fn follow_unknown() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut []).await;

    // Alice tries to follow an unknown participant
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Follow {
                participant_id: ParticipantId::from_u128(0xdeadbeef),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the UnknownParticipant error
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::UnknownParticipant)
    );
}

#[test_log::test(tokio::test)]
async fn follow_unfollow_across_room_boundaries() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts breakout room
    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_string(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;

    // Alice switches to the breakout room
    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice starts the whiteboard in the breakout room
    start_whiteboard(&mut alice, &mut []).await;

    // Bob stays in the main room, so Alice must not be allowed to follow him
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Follow {
                participant_id: bob.id(),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::UnknownParticipant)
    );

    // Alice also must not be allowed to unfollow a participant in a different room
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Unfollow {
                participant_id: bob.id(),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::UnknownParticipant)
    );

    // Bob should not receive any follow related events from another room
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn autostart_in_breakout_room() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the whiteboard in the main room
    start_whiteboard(&mut alice, &mut []).await;

    // Alice starts a breakout room
    alice
        .start_breakout_rooms(
            &mut [],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_string(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;

    // Alice switches to the breakout room without starting the whiteboard there herself
    let event = alice
        .switch_breakout_room(&mut [], RoomKind::Breakout(0.into()))
        .await;

    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Expected BreakoutEvent::SwitchedRoom, got: {event:?}");
    };

    // The whiteboard was autostarted in the breakout room with a fresh, empty board
    let state = own_data
        .get::<WhiteboardState>()
        .expect("Module data must be present")
        .expect("Whiteboard must have been autostarted in the breakout room");
    assert_eq!(
        state,
        WhiteboardState {
            scene: serde_json::json!({"elements": []}),
            edit_restrictions: EditRestrictions::Disabled,
        }
    );
}

#[test_log::test(tokio::test)]
async fn unfollow_not_started() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to unfollow a participant without starting the whiteboard first
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Unfollow {
                participant_id: ParticipantId::from_u128(0x12345678),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the NotStarted error
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::Error(WhiteboardError::NotStarted));
}

#[test_log::test(tokio::test)]
async fn unfollow_unknown() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut []).await;

    // Alice tries to unfollow an unknown participant
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Unfollow {
                participant_id: ParticipantId::from_u128(0xdeadbeef),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the UnknownParticipant error
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::UnknownParticipant)
    );
}

#[test_log::test(tokio::test)]
async fn follow() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut [&mut bob]).await;

    // Alice follows Bob
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Follow {
                participant_id: bob.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the Followed event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Followed {
            participant_id: bob.id()
        }
    );

    // Bob receives the FollowerGained event
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::FollowerGained {
            participant_id: alice.id()
        }
    );

    // Alice unfollows Bob
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Unfollow {
                participant_id: bob.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the Unfollowed event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Unfollowed {
            participant_id: bob.id()
        }
    );

    // Bob receives the FollowerLost event
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::FollowerLost {
            participant_id: alice.id()
        }
    );
}

#[test_log::test(tokio::test)]
async fn edit_restrictions() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts the whiteboard
    start_whiteboard(&mut alice, &mut [&mut bob]).await;

    // Alice enables edit restrictions
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::EnableEditRestrictions {
                unrestricted_participants: HashSet::new(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the EditRestrictionsEnabled event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::EditRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    // Bob receives the EditRestrictionsEnabled event
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::EditRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    // Bob tries to send a broadcast command, but is not allowed to edit
    bob.send_command::<WhiteboardModule>(
        WhiteboardCommand::Broadcast {
            data: json!({
                "some": "data",
            }),
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives the InsufficientPermissions error
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::InsufficientPermissions)
    );

    // Alice receives nothing
    assert!(alice.received_nothing());

    // Alice is still allowed to send a broadcast command because she is a moderator
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Broadcast {
                data: json!({
                    "some": "data",
                }),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the broadcast event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Broadcast {
            sender: alice.id(),
            data: json!({"some": "data"})
        }
    );

    // Bob receives the broadcast event
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Broadcast {
            sender: alice.id(),
            data: json!({"some": "data"})
        }
    );

    // Alice disables edit restrictions
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::DisableEditRestrictions, None)
        .await
        .unwrap();

    // Alice receives the EditRestrictionsDisabled event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::EditRestrictionsDisabled);

    // Bob receives the EditRestrictionsDisabled event
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::EditRestrictionsDisabled);

    // Bob is allowed to send a broadcast command again
    bob.send_command::<WhiteboardModule>(
        WhiteboardCommand::Broadcast {
            data: json!({
                "some": "data",
            }),
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives the broadcast event
    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Broadcast {
            sender: bob.id(),
            data: json!({"some": "data"})
        }
    );

    // Alice receives the broadcast event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Broadcast {
            sender: bob.id(),
            data: json!({"some": "data"})
        }
    );
}

#[test_log::test(tokio::test)]
async fn alice_in_breakout_bob_in_main() {
    let mut room = TestRoom::builder()
        .register_module::<WhiteboardModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts breakout room
    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_string(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;

    // Alice switches to the breakout room
    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice starts the whiteboard in the breakout room
    start_whiteboard(&mut alice, &mut []).await;

    // Bob receives nothing
    assert!(bob.received_nothing());

    // Alice sends a broadcast command
    alice
        .send_command::<WhiteboardModule>(
            WhiteboardCommand::Broadcast {
                data: json!({
                    "some": "data",
                }),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the broadcast event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Broadcast {
            sender: alice.id(),
            data: json!({"some": "data"})
        }
    );

    // Bob receives nothing
    assert!(bob.received_nothing());
}
