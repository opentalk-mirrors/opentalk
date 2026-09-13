// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use bytes::Bytes;
use opentalk_roomserver_module_e2ee::E2eeModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    connection_id::ConnectionId,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_e2ee::{
    E2eeCommand, E2eeError, E2eeEvent, MlsMessages, WelcomeMessage,
};
use pretty_assertions::assert_eq;

fn sample_welcome_message() -> WelcomeMessage {
    WelcomeMessage {
        welcome: Bytes::from_static(b"welcome-bytes"),
        ratchet_tree: Bytes::from_static(b"ratchet-tree-bytes"),
    }
}

fn sample_mls_messages() -> MlsMessages {
    MlsMessages {
        payload: vec![Bytes::from_static(b"mls1"), Bytes::from_static(b"mls2")],
    }
}

/// 1. Alice joins twice (alice1 and alice2)
/// 2. Bob and Charlie join
/// 3. Dave joins and enters a breakout room
/// 4. Alice (alice1) sends Invite to Bob
/// 5. Bob should receive WelcomeMessage
/// 6. alice2 and Charlie should receive MlsMessages
/// 7. Dave received nothing
#[test_log::test(tokio::test)]
async fn invite() {
    let mut room = TestRoom::builder().register_module::<E2eeModule>().spawn();

    let mut alice1 = room.join_alice_moderator(1).await;
    let mut alice2 = room.join_alice_moderator(2).await;
    flush_connected_events(&mut [&mut alice1]).await;

    alice1
        .start_breakout_rooms(
            &mut [&mut alice2],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 1".to_string(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice1, &mut alice2]).await;

    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice1, &mut alice2, &mut bob]).await;

    let mut dave = room.join_dave(0).await;
    flush_connected_events(&mut [&mut alice1, &mut alice2, &mut bob, &mut charlie]).await;
    dave.switch_breakout_room(
        &mut [&mut alice1, &mut alice2, &mut bob, &mut charlie],
        RoomKind::Breakout(0.into()),
    )
    .await;

    // Alice sends an invite to Bob
    let invitee = bob.connection_id();
    let welcome_message = sample_welcome_message();
    let mls_messages = sample_mls_messages();
    alice1
        .send_command::<E2eeModule>(
            E2eeCommand::Invite {
                invitee,
                welcome_message: welcome_message.clone(),
                mls_messages: mls_messages.clone(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice2 should receive the replicated invite event
    let replica_event = alice2.receive_event::<E2eeModule>().await.unwrap().payload;
    assert_eq!(replica_event, E2eeEvent::MlsMessages(mls_messages.clone()));

    // Bob should receive the welcome message
    let bob_event = bob.receive_event::<E2eeModule>().await.unwrap().payload;
    assert_eq!(bob_event, E2eeEvent::Welcome(welcome_message.clone()));

    // Charlie should receive the MLS message
    let charlie_event = charlie.receive_event::<E2eeModule>().await.unwrap().payload;
    assert_eq!(charlie_event, E2eeEvent::MlsMessages(mls_messages.clone()));

    // No additional messages should be sent
    assert!(alice1.received_nothing());
    assert!(alice2.received_nothing());
    assert!(bob.received_nothing());
    assert!(charlie.received_nothing());
    assert!(dave.received_nothing());
}

/// 1. Alice joins twice (alice1 and alice2)
/// 2. Bob joins
/// 3. Alice (alice1) sends Invite to herself (alice2)
/// 4. alice2 should receive WelcomeMessage
/// 5. Bob and alice1 should receive MlsMessage
#[test_log::test(tokio::test)]
async fn invite_self() {
    let mut room = TestRoom::builder().register_module::<E2eeModule>().spawn();

    let mut alice1 = room.join_alice_moderator(1).await;
    let mut alice2 = room.join_alice_moderator(2).await;
    flush_connected_events(&mut [&mut alice1]).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice1, &mut alice2]).await;

    // alice-1 sends invite to alice-2
    let invitee = alice2.connection_id();
    let welcome_message = sample_welcome_message();
    let mls_messages = sample_mls_messages();
    alice1
        .send_command::<E2eeModule>(
            E2eeCommand::Invite {
                invitee,
                welcome_message: welcome_message.clone(),
                mls_messages: mls_messages.clone(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice2 should receive the invite event
    let invite_event = alice2.receive_event::<E2eeModule>().await.unwrap().payload;
    assert_eq!(invite_event, E2eeEvent::Welcome(welcome_message.clone()));

    // Bob should receive a MLS message
    let bob_event = bob.receive_event::<E2eeModule>().await.unwrap().payload;
    assert_eq!(bob_event, E2eeEvent::MlsMessages(mls_messages.clone()));

    // No additional messages should be sent
    assert!(alice1.received_nothing());
    assert!(alice2.received_nothing());
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn invitee_unknown() {
    let mut room = TestRoom::builder().register_module::<E2eeModule>().spawn();

    // Alice and Bob join
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice sends an invalid invite (e.g., to a non-existent participant)
    alice
        .send_command::<E2eeModule>(
            E2eeCommand::Invite {
                invitee: ConnectionId::from_u128(0xdeadbeef),
                welcome_message: sample_welcome_message(),
                mls_messages: sample_mls_messages(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice should receive an error event
    let event = alice.receive_event::<E2eeModule>().await.unwrap().payload;
    assert_eq!(event, E2eeEvent::Error(E2eeError::InvalidParticipantTarget));

    // Bob should receive nothing
    assert!(bob.received_nothing());
}

/// 1. Alice joins twice (alice1 and alice2)
/// 2. Bob joins
/// 3. Dave joins and enters a breakout room
/// 4. Alice (alice1) forwards a message
/// 5. Bob, alice2 receive the forwarded message
/// 6. Dave received nothing
#[test_log::test(tokio::test)]
async fn forward_message() {
    let mut room = TestRoom::builder().register_module::<E2eeModule>().spawn();

    let mut alice1 = room.join_alice_moderator(1).await;
    let mut alice2 = room.join_alice_moderator(2).await;
    flush_connected_events(&mut [&mut alice1]).await;

    alice1
        .start_breakout_rooms(
            &mut [&mut alice2],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 1".to_string(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice1, &mut alice2]).await;

    let mut dave = room.join_dave(0).await;
    flush_connected_events(&mut [&mut alice1, &mut alice2, &mut bob]).await;
    dave.switch_breakout_room(
        &mut [&mut alice1, &mut alice2, &mut bob],
        RoomKind::Breakout(0.into()),
    )
    .await;

    // Alice forwards a message
    let message = sample_mls_messages();
    alice1
        .send_command::<E2eeModule>(E2eeCommand::Message(message.clone()), None)
        .await
        .unwrap();

    // Alice2 should receive the message
    let replica_event = alice2.receive_event::<E2eeModule>().await.unwrap().payload;
    assert_eq!(replica_event, E2eeEvent::MlsMessages(message.clone()));

    // Bob should receive the welcome message
    let bob_event = bob.receive_event::<E2eeModule>().await.unwrap().payload;
    assert_eq!(bob_event, E2eeEvent::MlsMessages(message.clone()));

    // No additional messages should be sent
    assert!(alice1.received_nothing());
    assert!(alice2.received_nothing());
    assert!(bob.received_nothing());
    assert!(dave.received_nothing());
}
