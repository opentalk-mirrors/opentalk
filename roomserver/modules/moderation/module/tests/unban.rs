// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::assert_matches;

use http::StatusCode;
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{
    core::CoreEvent, disconnect_reason::DisconnectReason, signaling::websocket::CloseFrame,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{BannedParticipantInfo, ModerationError, ModerationEvent},
};
use opentalk_types_common::users::UserId;
use opentalk_types_signaling::ParticipantId;
use uuid::Uuid;

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;
    let charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut bob]).await;

    // Bob tries to unban charlie
    bob.send_command::<ModerationModule>(
        ModerationCommand::Unban {
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
async fn already_unbanned() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Unban {
                target: ParticipantId::from_u128(0xb0b),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::AlreadyUnbanned)
    );
}

#[test_log::test(tokio::test)]
async fn unban_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut frank = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice, &mut frank]).await;

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

    // Frank also receives the disconnected event
    let frank_event = frank.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        frank_event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        } if participant_id == bob.id() && reason == DisconnectReason::Banned,
    );

    // Alice receives the event that bob got banned
    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert_matches!(
        &event.payload,
        ModerationEvent::ParticipantBanned(BannedParticipantInfo {
            participant_id,
            banned_participant
        })
        if participant_id == &bob.id() &&
        & banned_participant.display_name == bob.display_name()
    );

    // Frank receives the same `ParticipantBanned` event
    let frank_event = frank.receive::<ModerationEvent>().await.unwrap();
    assert_eq!(event.payload, frank_event.payload);

    let error = room
        .room_handle
        .reject_if_banned(UserId::from(Uuid::from(bob.id())))
        .await
        .unwrap_err();
    assert_eq!(error.status, StatusCode::FORBIDDEN);

    alice
        .send_command::<ModerationModule>(ModerationCommand::Unban { target: bob.id() }, None)
        .await
        .unwrap();

    // Alice receives the unbanned event
    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::ParticipantUnbanned {
            participant_id: bob.id()
        }
    );

    // Frank receives the same `ParticipantUnbanned` event
    let frank_event = frank.receive::<ModerationEvent>().await.unwrap();
    assert_eq!(event.payload, frank_event.payload);

    room.room_handle
        .reject_if_banned(UserId::from(Uuid::from(bob.id())))
        .await
        .expect("Bob must not be banned");
}
