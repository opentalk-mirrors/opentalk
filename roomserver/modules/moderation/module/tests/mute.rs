// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{assert_matches, collections::BTreeSet};

use livekit::{RoomEvent, RoomOptions};
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_test_util_livekit as livekit_mocking;
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::LiveKitState;
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
};
use opentalk_types_signaling::ParticipantId;

mod common;

#[test_log::test(tokio::test)]
async fn unknown_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();

    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);

    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Mute {
                participants: BTreeSet::from([disconnected_participant]),
            },
            None,
        )
        .await
        .unwrap();

    let error_event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        error_event.payload,
        ModerationEvent::Error(ModerationError::UnknownParticipants {
            participants: BTreeSet::from([disconnected_participant])
        })
    )
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_mute_bob() {
    let (_container, mut room, public_url) = common::build_room().await;

    // Alice and Bob join the meeting
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    // join livekit to ensure participant exists and can be muted.
    let (bob_room, mut room_events) = livekit::Room::connect(
        &public_url,
        &bob_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();
    let connected = room_events.recv().await;
    assert_matches!(connected, Some(RoomEvent::Connected { .. }));

    // Publish a track for Bob to ensure he can be muted
    let track = livekit_mocking::publish_audio(&bob_room, &mut room_events)
        .await
        .unwrap();
    track.unmute();

    // Alice sends the mute command
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Mute {
                participants: BTreeSet::from([bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    assert!(alice.received_nothing());

    let mute_event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        mute_event.payload,
        ModerationEvent::Muted {
            moderator: alice.id()
        }
    );

    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    bob.send_command::<ModerationModule>(
        ModerationCommand::Mute {
            participants: BTreeSet::from_iter([alice.id()]),
        },
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    )
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_alice_in_breakout_bob_in_main() {
    let (_container, mut room, public_url) = common::build_room().await;

    // Alice and Bob join the meeting
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Connect to livekit to ensure participants exist and can be muted.
    let token = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be present")
        .expect("LiveKit state must not be none")
        .credentials
        .token;
    let (bob_room, mut room_events) =
        livekit::Room::connect(&public_url, &token, RoomOptions::default())
            .await
            .unwrap();
    let connected = room_events.recv().await;
    assert_matches!(connected, Some(RoomEvent::Connected { .. }));

    // Publish a track for Bob to ensure he can be muted
    let track = livekit_mocking::publish_audio(&bob_room, &mut room_events)
        .await
        .unwrap();
    track.unmute();

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_string(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice sends the mute command
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Mute {
                participants: BTreeSet::from([bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let mute_event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        mute_event.payload,
        ModerationEvent::Muted {
            moderator: alice.id()
        }
    );
}
