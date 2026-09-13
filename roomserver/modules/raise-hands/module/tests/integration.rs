// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{assert_matches, collections::BTreeSet};

use insta::assert_json_snapshot;
use opentalk_roomserver_module_raise_hands::RaiseHandsModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        command::BreakoutCommand,
        event::BreakoutEvent,
    },
    core::CoreEvent,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_raise_hands::{
    RAISE_HANDS_MODULE_ID,
    command::RaiseHandsCommand,
    event::{RaiseHandsError, RaiseHandsEvent},
    state::{RaisedHandPeerState, RaisedHandState},
};
use pretty_assertions::assert_eq;

#[test_log::test(tokio::test)]
async fn join_events_contain_data() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let state = alice
        .join_success()
        .get_module::<RaisedHandState>()
        .expect("Module data must be present")
        .expect("RaisedHand module state must not be None");
    assert_json_snapshot!(state, @r#"
    {
      "raise_hands_enabled": true
    }
    "#);

    raise_hand(&mut alice, &mut []).await;

    // Bob will receive his own state and Alices state
    let bob = room.join_bob(0).await;
    let own_state = bob
        .join_success()
        .get_module::<RaisedHandState>()
        .expect("Module data must be present")
        .expect("RaisedHand module state must not be None");
    assert_json_snapshot!(own_state,
        @r#"
    {
      "raise_hands_enabled": true
    }
    "#);

    assert_json_snapshot!(&bob.join_success().participants[0].module_data.get(&RAISE_HANDS_MODULE_ID), {
        ".raised_at" => "[timestamp]",
    },
        @r#"
    {
      "raised_at": "[timestamp]"
    }
    "#);

    // Alice receives Bobs state
    let CoreEvent::ParticipantConnected { peer_data, .. } =
        alice.receive::<CoreEvent>().await.unwrap().payload
    else {
        panic!("Expected participant connected event");
    };
    assert!(
        !peer_data.contains_key(&RAISE_HANDS_MODULE_ID),
        "Bob doesn't have state since he just joined"
    );
}

#[test_log::test(tokio::test)]
async fn enable_raise_hands_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;

    // Raised hands is enabled by default
    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    bob.send_command::<RaiseHandsModule>(RaiseHandsCommand::EnableRaiseHands, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::Error(RaiseHandsError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn disable_raise_hands_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::Error(RaiseHandsError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn reset_raise_hands_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<RaiseHandsModule>(
        RaiseHandsCommand::ResetRaisedHands { target: None },
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::Error(RaiseHandsError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn raise_hand_disabled() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Disable raise hands
    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    // Bob tries to raise his hand
    bob.send_command::<RaiseHandsModule>(RaiseHandsCommand::RaiseHand, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::Error(RaiseHandsError::RaiseHandsDisabled)
    );
}

#[test_log::test(tokio::test)]
async fn enable_raise_hands() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::EnableRaiseHands, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsEnabled {
            issued_by: alice.id()
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsEnabled {
            issued_by: alice.id()
        }
    );
}

#[test_log::test(tokio::test)]
async fn disable_raise_hands() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );
}

#[test_log::test(tokio::test)]
async fn raise_and_lower_hand() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Bob raises his hand
    raise_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;
    // Bob lowers his hand
    lower_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;
}

#[test_log::test(tokio::test)]
async fn reset_raised_hands_partial() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Bob and Charlie raise their hands
    raise_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;
    raise_hand(&mut charlie, &mut [&mut alice, &mut bob]).await;

    // Alice resets raised hands for Bob
    alice
        .send_command::<RaiseHandsModule>(
            RaiseHandsCommand::ResetRaisedHands {
                target: Some(BTreeSet::from_iter([bob.id()])),
            },
            None,
        )
        .await
        .unwrap();

    // Everyone receives the reset event
    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );

    let event = charlie
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );
}

#[test_log::test(tokio::test)]
async fn reset_raised_hands_all() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Bob and Charlie raise their hands
    raise_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;
    raise_hand(&mut charlie, &mut [&mut alice, &mut bob]).await;

    // Alice resets raised hands for all
    alice
        .send_command::<RaiseHandsModule>(
            RaiseHandsCommand::ResetRaisedHands { target: None },
            None,
        )
        .await
        .unwrap();

    // Everyone receives the reset event
    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id(), charlie.id()])
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id(), charlie.id()])
        }
    );

    let event = charlie
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id(), charlie.id()])
        }
    );
}

#[test_log::test(tokio::test)]
async fn reset_raise_hands_ignores_unraised() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Bob raises his hands
    raise_hand(&mut bob, &mut [&mut alice, &mut charlie]).await;

    // Alice resets raised hands all
    alice
        .send_command::<RaiseHandsModule>(
            RaiseHandsCommand::ResetRaisedHands { target: None },
            None,
        )
        .await
        .unwrap();

    // Everyone receives the reset event
    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );

    let event = charlie
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaisedHandResetByModerator {
            issued_by: alice.id(),
            participants: BTreeSet::from_iter([bob.id()])
        }
    );
}

#[test_log::test(tokio::test)]
async fn reset_raised_hands_noop() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<RaiseHandsModule>(
            RaiseHandsCommand::ResetRaisedHands { target: None },
            None,
        )
        .await
        .unwrap();

    // Resetting raised hands when no hands were raised does not produce any events
    assert!(alice.received_nothing());
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn enable_raise_hands_does_not_overwrite() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice raises her hand
    raise_hand(&mut alice, &mut []).await;

    // Alice sends the enable raise hands command
    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::EnableRaiseHands, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsEnabled {
            issued_by: alice.id()
        }
    );

    let bob = room.join_bob(0).await;
    let join_success = bob.join_success();
    let bob_state = join_success
        .get_module::<RaisedHandState>()
        .expect("Module data must be present")
        .expect("RaisedHand module state must not be None");
    // The raised hands were not affected by the enable command
    assert_eq!(
        bob_state,
        RaisedHandState {
            raise_hands_enabled: true,
            state: None,
        }
    );
    let peer_state_alice = join_success
        .participants
        .iter()
        .find(|&p| p.id == alice.id())
        .unwrap();
    let peer_state_alice = peer_state_alice
        .get_module::<RaisedHandPeerState>()
        .unwrap();
    assert_matches!(peer_state_alice, Some(RaisedHandPeerState { .. }));
}

#[test_log::test(tokio::test)]
async fn breakout_switch() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Breakout Room".into(),
                    assignments: Vec::new(),
                }],
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::Started { .. });

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::Started { .. });

    alice
        .send_breakout_command(
            BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(0))),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Received unexpected event: {:?}", event);
    };
    let state = own_data
        .get::<RaisedHandState>()
        .expect("Module data must be present")
        .expect("RaisedHand module state must not be None");
    assert_eq!(
        state,
        RaisedHandState {
            raise_hands_enabled: true,
            state: None,
        }
    );

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_json_snapshot!(event, @r#"
    {
      "message": "participant_switched_room",
      "participant_id": "00000000-0000-0000-0000-0000000a11ce",
      "old_room": {
        "kind": "main"
      },
      "new_room": {
        "kind": "breakout",
        "id": 0
      }
    }
    "#);

    // Alice raises her hand in the breakout room
    raise_hand(&mut alice, &mut []).await;

    assert!(bob.received_nothing());

    bob.send_breakout_command(
        BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(0))),
        None,
    )
    .await
    .unwrap();

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    let BreakoutEvent::SwitchedRoom {
        own_data,
        peer_data,
        ..
    } = event
    else {
        panic!("Received unexpected event: {:?}", event);
    };

    let state = own_data
        .get::<RaisedHandState>()
        .expect("Module data must be present")
        .expect("RaisedHand module state must not be None");
    assert_eq!(
        state,
        RaisedHandState {
            raise_hands_enabled: true,
            state: None,
        }
    );
    let peer_state_alice = peer_data.get(&alice.id()).unwrap();
    let peer_state_alice = peer_state_alice.get(&RAISE_HANDS_MODULE_ID).unwrap();
    assert_json_snapshot!(peer_state_alice, {
        ".raised_at" => "[timestamp]"
    }, @r#"
    {
      "raised_at": "[timestamp]"
    }
    "#);
}

#[test_log::test(tokio::test)]
async fn raise_hand_is_breakout_local() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Breakout Room".into(),
                    assignments: Vec::new(),
                }],
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::Started { .. });

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::Started { .. });

    let event = charlie.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::Started { .. });

    // Charlie switches to the breakout room
    charlie
        .send_breakout_command(
            BreakoutCommand::SwitchRoom(RoomKind::Breakout(BreakoutId::from(0))),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::ParticipantSwitchedRoom { .. });

    let event = bob.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::ParticipantSwitchedRoom { .. });

    let event = charlie.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::SwitchedRoom { .. });

    // Bob raises his hand in the main room
    raise_hand(&mut bob, &mut [&mut alice]).await;

    // Charlie does not receive the raised hand event
    assert!(charlie.received_nothing());

    // Bob lowers his hand in the main room
    lower_hand(&mut bob, &mut [&mut alice]).await;

    // Charlie does not receive the lowered hand event
    assert!(charlie.received_nothing());

    alice
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::DisableRaiseHands, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    let event = bob
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::RaiseHandsDisabled {
            issued_by: alice.id()
        }
    );

    assert!(charlie.received_nothing());

    // Charlie can still raise his hand
    raise_hand(&mut charlie, &mut []).await;
}

#[test_log::test(tokio::test)]
async fn raise_hand_resets_when_switching_rooms() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice raises her hand in the main room
    raise_hand(&mut alice, &mut []).await;

    alice
        .start_breakout_rooms(
            &mut [],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Breakout Room".into(),
                    assignments: Vec::new(),
                }],
                duration: None,
            },
        )
        .await;
    alice
        .switch_breakout_room(&mut [], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let peer_state_alice = bob
        .join_success()
        .participants
        .iter()
        .find(|&p| p.id == alice.id())
        .unwrap();
    let peer_state_alice = peer_state_alice
        .get_module::<RaisedHandPeerState>()
        .unwrap();
    assert!(
        peer_state_alice.is_none(),
        "Alice state must be none since she left the room, but was: {:?}",
        peer_state_alice,
    );

    let event = bob
        .switch_breakout_room(&mut [&mut alice], RoomKind::Breakout(BreakoutId::from(0)))
        .await;
    let BreakoutEvent::SwitchedRoom { peer_data, .. } = event else {
        panic!("unexpected event");
    };

    // there should be no state for alice in the breakout room
    assert!(!peer_data.contains_key(&alice.id()));
}

/// The raised hand state is not reset when the participant disconnects but is
/// still connected with another connection
#[test_log::test(tokio::test)]
async fn raise_hand_not_reset_when_connected() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice_0 = room.join_alice_moderator(0).await;

    // Alice raises her hand in the main room
    raise_hand(&mut alice_0, &mut []).await;

    // Second connection should see raised hand
    let mut alice_1 = room.join_alice_moderator(1).await;
    flush_connected_events(&mut [&mut alice_0]).await;

    let alice_1_state = alice_1
        .join_success()
        .get_module::<RaisedHandState>()
        .unwrap();
    assert_matches!(
        alice_1_state,
        Some(RaisedHandState {
            raise_hands_enabled: true,
            state: Some(_),
        })
    );

    // alice_0 disconnects
    alice_0.disconnect().await.unwrap();
    alice_1.receive::<CoreEvent>().await.unwrap();

    // raised hand state should still persist
    let alice_0 = room.join_alice_moderator(0).await;
    flush_connected_events(&mut [&mut alice_1]).await;

    let alice_0_state = alice_0
        .join_success()
        .get_module::<RaisedHandState>()
        .unwrap();
    assert_matches!(
        alice_0_state,
        Some(RaisedHandState {
            raise_hands_enabled: true,
            state: Some(_),
        })
    );
}

#[test_log::test(tokio::test)]
async fn raise_hand_resets_last_connection_is_closed() {
    let mut room = TestRoom::builder()
        .register_module::<RaiseHandsModule>()
        .spawn();
    let mut alice_0 = room.join_alice_moderator(0).await;

    // Alice raises her hand in the main room
    raise_hand(&mut alice_0, &mut []).await;

    // alice_0 disconnects
    alice_0.disconnect().await.unwrap();

    // Second connection should not see raised hand since alice disconnected
    let alice_1 = room.join_alice_moderator(1).await;
    let alice_1_state = alice_1
        .join_success()
        .get_module::<RaisedHandState>()
        .unwrap();
    assert!(
        matches!(
            alice_1_state,
            Some(RaisedHandState {
                raise_hands_enabled: true,
                state: None,
            })
        ),
        "Wrong state: {alice_1_state:?}"
    );
}

async fn raise_hand(
    participant: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    participant
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::RaiseHand, None)
        .await
        .unwrap();

    let event = participant
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::HandRaised {
            participant: participant.id()
        }
    );

    for other in others {
        let event = other
            .receive_event::<RaiseHandsModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(
            event,
            RaiseHandsEvent::HandRaised {
                participant: participant.id()
            }
        );
    }
}

async fn lower_hand(
    participant: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    participant
        .send_command::<RaiseHandsModule>(RaiseHandsCommand::LowerHand, None)
        .await
        .unwrap();

    let event = participant
        .receive_event::<RaiseHandsModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        RaiseHandsEvent::HandLowered {
            participant: participant.id()
        }
    );

    for other in others {
        let event = other
            .receive_event::<RaiseHandsModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(
            event,
            RaiseHandsEvent::HandLowered {
                participant: participant.id()
            }
        );
    }
}
