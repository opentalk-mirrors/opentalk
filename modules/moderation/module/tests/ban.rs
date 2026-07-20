// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{assert_matches, collections::HashSet};

use http::StatusCode;
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{
    core::{CoreEvent, LeftWaitingRoom},
    disconnect_reason::DisconnectReason,
    room_parameters::WaitingRoom,
    signaling::websocket::CloseFrame,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{BannedParticipantInfo, ModerationError, ModerationEvent},
    state::ModerationState,
};
use opentalk_types_common::users::{DisplayName, UserId};
use opentalk_types_signaling::ParticipantId;
use uuid::Uuid;

#[test_log::test(tokio::test)]
async fn join_info() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::Ban { target: bob.id() }, None)
        .await
        .unwrap();

    // Bob receives the banned event
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(event.payload, ModerationEvent::Banned);

    assert_eq!(
        bob.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice receives the disconnect event
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        } if participant_id == bob.id() && reason == DisconnectReason::Banned,
    );

    // Alice receives the event that a bob got banned
    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        ModerationEvent::ParticipantBanned(BannedParticipantInfo {
            participant_id,
            banned_participant
        }) if participant_id == bob.id() && &banned_participant.display_name == bob.display_name()
    );

    alice.disconnect().await.unwrap();

    // reconnect to get the join success
    let alice = room.join_alice_moderator(0).await;

    let moderator_data = alice
        .join_success()
        .module_data
        .get::<ModerationState>()
        .expect("Module data must be present")
        .expect("Module data must not be none")
        .moderator_data
        .expect("Moderator data must be present");

    assert_eq!(moderator_data.banned_participants.len(), 1);

    let ban_info = moderator_data
        .banned_participants
        .first()
        .expect("Expected entry in banned participants");

    assert_eq!(ban_info.participant_id, bob.id());
    assert_eq!(ban_info.banned_participant.banned_by, alice.id());
    assert_eq!(
        &ban_info.banned_participant.display_name,
        bob.display_name()
    );
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;
    let charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut bob]).await;

    // Bob tries to ban charlie
    bob.send_command::<ModerationModule>(
        ModerationCommand::Ban {
            target: charlie.id(),
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives an error because he is not a moderator
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn cannot_ban_room_owner() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut frank = room.join_frank_moderator(0).await;
    // Alice is the room owner for the TestRoom
    let alice = room.join_alice_moderator(0).await;

    flush_connected_events(&mut [&mut frank]).await;

    // Frank tries to ban Alice
    frank
        .send_command::<ModerationModule>(ModerationCommand::Ban { target: alice.id() }, None)
        .await
        .unwrap();

    // Frank receives an error because he cannot ban the room owner
    let event = frank.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::CannotBanRoomOwner)
    );
}

#[test_log::test(tokio::test)]
async fn cannot_ban_guest() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let gustav = room.join_gustav_guest().await;

    flush_connected_events(&mut [&mut alice]).await;

    // Alice tries to ban gustav
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Ban {
                target: gustav.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives an error because guests cannot be banned
    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::CannotBanGuests)
    );
}

#[test_log::test(tokio::test)]
async fn unknown_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Ban {
                target: ParticipantId::from_u128(0xb0b),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::UnknownParticipant)
    );
}

#[test_log::test(tokio::test)]
async fn ban_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::Ban { target: bob.id() }, None)
        .await
        .unwrap();

    // Bob receives the banned event
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(event.payload, ModerationEvent::Banned);

    assert_eq!(
        bob.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice receives the disconnect event
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        } if participant_id == bob.id() && reason == DisconnectReason::Banned,
    );

    // Alice receives the event that a bob got banned
    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        ModerationEvent::ParticipantBanned(BannedParticipantInfo {
            participant_id,
            banned_participant
        }) if participant_id == bob.id() && &banned_participant.display_name == bob.display_name()
    );

    let error = room
        .room_handle
        .reject_if_banned(UserId::from(Uuid::from(bob.id())))
        .await
        .unwrap_err();

    assert_eq!(error.status, StatusCode::FORBIDDEN);
}

#[test_log::test(tokio::test)]
async fn ban_waiting_participant() {
    let mut room = TestRoom::builder()
        .waiting_room(WaitingRoom::ForEveryone)
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.waiting_room_bob(0).await;
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::Ban { target: bob.id() }, None)
        .await
        .unwrap();

    // Bob receives the banned event
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(event.payload, ModerationEvent::Banned);

    assert_eq!(
        bob.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice receives the disconnect event
    let event = alice.receive::<CoreEvent>().await.unwrap();

    assert_matches!(
        event.payload,
        CoreEvent::LeftWaitingRoom(LeftWaitingRoom { id, connection_ids })
        if id == bob.id() &&
        connection_ids == HashSet::from_iter([bob.connection_id()])
    );

    // Alice receives the event that a bob got banned
    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        ModerationEvent::ParticipantBanned(BannedParticipantInfo {
            participant_id,
            banned_participant
        })
        if participant_id == bob.id()
        // Can't get display name from bob because he never joined
        && banned_participant.display_name == DisplayName::from_str_lossy("Bob the bold")
    );

    let error = room
        .room_handle
        .reject_if_banned(UserId::from(Uuid::from(bob.id())))
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::FORBIDDEN);
}

#[test_log::test(tokio::test)]
async fn ban_lobby_participant() {
    let mut room = TestRoom::builder()
        .waiting_room(WaitingRoom::ForEveryone)
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.lobby_bob(0).await;
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::Ban { target: bob.id() }, None)
        .await
        .unwrap();

    // Bob receives the banned event
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(event.payload, ModerationEvent::Banned);

    assert_eq!(
        bob.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice receives the event that a bob got banned
    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        ModerationEvent::ParticipantBanned(BannedParticipantInfo {
            participant_id,
            banned_participant
        })
        if participant_id == bob.id()
        // Can't get display name from bob because he never joined
        && banned_participant.display_name == DisplayName::from_str_lossy("Bob the bold")
    );

    let error = room
        .room_handle
        .reject_if_banned(UserId::from(Uuid::from(bob.id())))
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::FORBIDDEN);

    // Alice does not receive a disconnect event for bob because he was only was in the lobby
    assert!(alice.received_nothing())
}

#[test_log::test(tokio::test)]
async fn ban_affects_lobby_and_conference() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob_conference = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut bob_lobby = room.lobby_bob(1).await;

    // Alice bans Bob
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Ban {
                target: bob_conference.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Bob receives the banned event in the conference
    let event = bob_conference
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Banned);

    // Bob receives the banned event in the lobby
    let event = bob_lobby
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Banned);

    // Bobs connection in the conference is closed
    assert_eq!(
        bob_conference.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Bobs connection in the lobby is closed. If the lobby connection received a
    // `DisplayNameAssigned` event from the conference connection disconnecting, this would fail
    // with an unexpected message instead of a close frame.
    assert_eq!(
        bob_lobby.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice only gets notified about bob disconnecting from the conference
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        } if participant_id == bob_conference.id() && reason == DisconnectReason::Banned,
    );

    // Alice receives the event that bob got banned
    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        ModerationEvent::ParticipantBanned(BannedParticipantInfo {
            participant_id,
            banned_participant
        }) if participant_id == bob_conference.id()
            && &banned_participant.display_name == bob_conference.display_name()
    );

    assert!(alice.received_nothing());

    let error = room
        .room_handle
        .reject_if_banned(UserId::from(Uuid::from(bob_conference.id())))
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::FORBIDDEN);
}
