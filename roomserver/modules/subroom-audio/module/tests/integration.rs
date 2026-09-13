// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::assert_matches;

use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_module_subroom_audio::SubroomAudioModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_test_util_livekit as test_util;
use opentalk_roomserver_types_subroom_audio::{
    WhisperId,
    command::{ParticipantTargets, SubroomAudioCommand},
    event::{SubroomAudioError, SubroomAudioEvent},
};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;

pub async fn build_room() -> (test_util::ContainerGuard, TestRoom, String) {
    let (container, settings) = test_util::create_livekit_container().await;

    let room = TestRoom::builder()
        .room_id(RoomId::generate())
        .register_module::<LiveKitModule>()
        .register_module::<SubroomAudioModule>()
        .add_init_module_data(&settings)
        .unwrap()
        .spawn();

    (container, room, settings.public_url)
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_create_whisper_group_and_invite() {
    let (_container, mut room, _public_url) = build_room().await;

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice creates a whisper group with Bob
    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: [bob.id()].into_iter().collect(),
            },
            None,
        )
        .await
        .unwrap();

    // Bob should receive an invite
    let event = bob
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload;

    match event {
        SubroomAudioEvent::WhisperInvite { issuer, .. } => {
            assert_eq!(issuer, alice.id());
        }
        _ => panic!("Expected WhisperInvite event"),
    }
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_accept_and_leave_whisper_group() {
    let (_container, mut room, _public_url) = build_room().await;

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: [bob.id()].into_iter().collect(),
            },
            None,
        )
        .await
        .unwrap();

    let whisper_id = match bob
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload
    {
        SubroomAudioEvent::WhisperInvite { group, .. } => group.whisper_id,
        _ => panic!("Expected WhisperInvite event"),
    };

    // Bob accepts the invite
    bob.send_command::<SubroomAudioModule>(
        SubroomAudioCommand::AcceptWhisperInvite { whisper_id },
        None,
    )
    .await
    .unwrap();

    // Bob receives a token
    let token_event = bob
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload;
    match token_event {
        SubroomAudioEvent::WhisperToken {
            whisper_id: received_whisper_id,
            token,
        } => {
            assert_eq!(received_whisper_id, whisper_id);
            assert!(!token.is_empty());
        }
        _ => panic!("Expected WhisperToken event, got {:?}", token_event),
    }

    assert_matches!(
        alice
            .receive_event::<SubroomAudioModule>()
            .await
            .unwrap()
            .payload,
        SubroomAudioEvent::WhisperGroupCreated { .. }
    );

    assert_matches!(
        alice
            .receive_event::<SubroomAudioModule>()
            .await
            .unwrap()
            .payload,
        SubroomAudioEvent::WhisperInviteAccepted { .. }
    );

    // Bob leaves the whisper group
    bob.send_command::<SubroomAudioModule>(
        SubroomAudioCommand::LeaveWhisperGroup { whisper_id },
        None,
    )
    .await
    .unwrap();

    // Alice receives notification that Bob left
    let left_event = alice
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload;
    match left_event {
        SubroomAudioEvent::LeftWhisperGroup(info) => {
            assert_eq!(info.participant_id, bob.id());
        }
        _ => panic!("Expected LeftWhisperGroup event: {left_event:?}"),
    }
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_cannot_create_whisper_group_with_empty_participants() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: [].into_iter().collect(),
            },
            None,
        )
        .await
        .unwrap();

    // Try to create a whisper group with no participants
    let event = alice
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        SubroomAudioEvent::Error(SubroomAudioError::EmptyParticipantList)
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_cannot_invite_nonexistent_participant() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    let fake_id = ParticipantId::generate();

    // Try to invite a participant that does not exist
    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: [fake_id].into_iter().collect(),
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice
            .receive_event::<SubroomAudioModule>()
            .await
            .unwrap()
            .payload,
        SubroomAudioEvent::Error(SubroomAudioError::InvalidParticipantTargets {
            participant_ids: vec![fake_id]
        })
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_cannot_kick_without_permission() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice creates a whisper group with Bob
    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: [bob.id()].into_iter().collect(),
            },
            None,
        )
        .await
        .unwrap();

    let whisper_id = match bob
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload
    {
        SubroomAudioEvent::WhisperInvite { group, .. } => group.whisper_id,
        _ => panic!("Expected WhisperInvite event"),
    };

    // Bob tries to kick Alice (should fail)
    bob.send_command::<SubroomAudioModule>(
        SubroomAudioCommand::KickWhisperParticipants(ParticipantTargets {
            whisper_id,
            participant_ids: [alice.id()].into_iter().collect(),
        }),
        None,
    )
    .await
    .unwrap();

    let kick_event = bob
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        kick_event,
        SubroomAudioEvent::Error(SubroomAudioError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_cannot_invite_to_nonexistent_group() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let fake_whisper_id = WhisperId::generate();

    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::InviteToWhisperGroup(ParticipantTargets {
                whisper_id: fake_whisper_id,
                participant_ids: [bob.id()].into_iter().collect(),
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        SubroomAudioEvent::Error(SubroomAudioError::InvalidWhisperId)
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_cannot_accept_whisper_invite_twice() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: [bob.id()].into_iter().collect(),
            },
            None,
        )
        .await
        .unwrap();

    let whisper_id = match bob
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload
    {
        SubroomAudioEvent::WhisperInvite { group, .. } => group.whisper_id,
        _ => panic!("Expected WhisperInvite event"),
    };

    // Bob accepts the invite
    bob.send_command::<SubroomAudioModule>(
        SubroomAudioCommand::AcceptWhisperInvite { whisper_id },
        None,
    )
    .await
    .unwrap();

    assert_matches!(
        bob.receive_event::<SubroomAudioModule>()
            .await
            .unwrap()
            .payload,
        SubroomAudioEvent::WhisperToken { .. }
    );

    // Bob tries to accept again
    bob.send_command::<SubroomAudioModule>(
        SubroomAudioCommand::AcceptWhisperInvite { whisper_id },
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        SubroomAudioEvent::Error(SubroomAudioError::AlreadyAccepted)
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_cannot_accept_invite_when_not_invited() {
    let (_container, mut room, _public_url) = build_room().await;

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Alice creates a whisper group with Bob (but not Charlie)
    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: [bob.id()].into_iter().collect(),
            },
            None,
        )
        .await
        .unwrap();

    // Bob receives the invite and gets the whisper_id
    let whisper_id = match bob
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload
    {
        SubroomAudioEvent::WhisperInvite { group, .. } => group.whisper_id,
        _ => panic!("Expected WhisperInvite event"),
    };

    // Charlie tries to accept the invite, but was not invited
    charlie
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::AcceptWhisperInvite { whisper_id },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        charlie
            .receive_event::<SubroomAudioModule>()
            .await
            .unwrap()
            .payload,
        SubroomAudioEvent::Error(SubroomAudioError::NotInvited)
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_cannot_kick_self() {
    let (_container, mut room, _public_url) = build_room().await;

    let mut alice = room.join_alice_moderator(0).await;
    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice creates a whisper group with herself
    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::CreateWhisperGroup {
                participant_ids: [bob.id()].into_iter().collect(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the group created event and gets the whisper_id
    let whisper_id = match alice
        .receive_event::<SubroomAudioModule>()
        .await
        .unwrap()
        .payload
    {
        SubroomAudioEvent::WhisperGroupCreated { group, .. } => group.whisper_id,
        _ => panic!("Expected WhisperGroupCreated event"),
    };

    // Alice tries to kick herself
    alice
        .send_command::<SubroomAudioModule>(
            SubroomAudioCommand::KickWhisperParticipants(ParticipantTargets {
                whisper_id,
                participant_ids: [alice.id()].into_iter().collect(),
            }),
            None,
        )
        .await
        .unwrap();

    // Alice should NOT receive a Kicked event for herself
    assert!(alice.received_nothing());
}
