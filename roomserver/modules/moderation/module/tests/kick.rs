// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::assert_matches;

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipant,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    core::CoreEvent, disconnect_reason::DisconnectReason, kick_reason::KickReason,
    room_parameters::WaitingRoom, signaling::websocket::CloseFrame,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
    state::ModeratorJoinInfo,
};
use opentalk_types_signaling::{ParticipantId, Role};

async fn receive_waiting_room_updated<S>(
    participants: &mut [&mut MockParticipant<S>],
    new_state: WaitingRoom,
) {
    for participant in participants {
        let event = participant
            .receive_event::<ModerationModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(event, ModerationEvent::WaitingRoomUpdated { new_state });
    }
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;
    let charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut bob]).await;

    // Bob tries to kick charlie
    bob.send_command::<ModerationModule>(
        ModerationCommand::Kick {
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
async fn non_room_owner_can_not_kick_moderator() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Alice grants Bob moderator rights
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::UpdateRole {
                participant_id: bob.id(),
                new_role: Role::Moderator,
            },
            None,
        )
        .await
        .unwrap();

    // Bob receives the role updated event
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::RoleUpdated {
            participant_id: bob.id(),
            new_role: Role::Moderator,
            issued_by: alice.id(),
            moderator_data: Some(ModeratorJoinInfo {
                waiting_room: WaitingRoom::Disabled,
                guest_access: true,
                waiting_room_participants: Vec::new(),
                banned_participants: Vec::new(),
            }),
        }
    );

    // Alice grants Charlie moderator rights
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::UpdateRole {
                participant_id: charlie.id(),
                new_role: Role::Moderator,
            },
            None,
        )
        .await
        .unwrap();

    // Bob receives the role update event
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::RoleUpdated {
            participant_id: charlie.id(),
            new_role: Role::Moderator,
            issued_by: alice.id(),
            moderator_data: None,
        }
    );

    // Bob tries to kick Charlie
    bob.send_command::<ModerationModule>(
        ModerationCommand::Kick {
            target: charlie.id(),
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives an error because the room owner can not be kicked
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
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
            ModerationCommand::Kick {
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
async fn kick_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::Kick { target: bob.id() }, None)
        .await
        .unwrap();

    // The waiting room gets enabled
    receive_waiting_room_updated(&mut [&mut alice, &mut bob], WaitingRoom::ForEveryone).await;

    // Bob receives the kicked event
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Kicked {
            reason: KickReason::Kicked
        }
    );

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
        } if participant_id == bob.id() && reason == DisconnectReason::Kicked,
    );
}

#[test_log::test(tokio::test)]
async fn kick_guest() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Kick {
                target: gustav.id(),
            },
            None,
        )
        .await
        .unwrap();

    // The waiting room gets enabled
    receive_waiting_room_updated(&mut [&mut alice, &mut gustav], WaitingRoom::ForGuests).await;

    // Gustav receives the kicked event
    let event = gustav.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Kicked {
            reason: KickReason::Kicked
        }
    );

    assert_eq!(
        gustav.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice receives the disconnect event
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        } if participant_id == gustav.id() && reason == DisconnectReason::Kicked,
    ));
}

#[test_log::test(tokio::test)]
async fn cannot_kick_room_owner() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut frank = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Frank tries to kick Alice
    frank
        .send_command::<ModerationModule>(ModerationCommand::Kick { target: alice.id() }, None)
        .await
        .unwrap();

    let event = frank
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    // Alice cannot be kicked because she is the room owner
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::CannotKickRoomOwner)
    );

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn kick_affects_lobby_and_conference() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob_conference = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut bob_lobby = room.lobby_bob(1).await;

    // Alice kicks Bob
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Kick {
                target: bob_conference.id(),
            },
            None,
        )
        .await
        .unwrap();

    receive_waiting_room_updated(
        &mut [&mut alice, &mut bob_conference],
        WaitingRoom::ForEveryone,
    )
    .await;

    // Bob receives the kicked event in the conference
    let event = bob_conference
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Kicked {
            reason: KickReason::Kicked
        }
    );

    // Bob receives the kicked event in the lobby
    let event = bob_lobby
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Kicked {
            reason: KickReason::Kicked
        }
    );

    // Bobs connection in the lobby is closed
    assert_eq!(
        bob_conference.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Bobs connection in the lobby is closed
    assert_eq!(
        bob_lobby.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_string(),
        })
    );

    // Alice only gets notified about bob disconnecting from the conference
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert!(matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            reason,
            ..
        } if participant_id == bob_conference.id() && reason == DisconnectReason::Kicked,
    ));

    assert!(alice.received_nothing());
}
