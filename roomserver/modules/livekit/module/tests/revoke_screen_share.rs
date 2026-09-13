// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use livekit::RoomOptions;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::{LiveKitCommand, LiveKitError, LiveKitEvent, LiveKitState};
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;

mod common;

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_unknown_participant() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = common::build_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Alice sends the command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::RevokeScreenSharePermission {
                participants: BTreeSet::from([disconnected_participant]),
            },
            None,
        )
        .await
        .unwrap();

    let error_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        error_event.payload,
        LiveKitEvent::Error(LiveKitError::UnknownParticipant {
            participant: BTreeSet::from([disconnected_participant])
        })
    )
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_insufficient_permissions() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = common::build_room().await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;

    // Bob sends the command
    bob.send_command::<LiveKitModule>(
        LiveKitCommand::RevokeScreenSharePermission {
            participants: BTreeSet::from([disconnected_participant]),
        },
        None,
    )
    .await
    .unwrap();

    let error_event = bob.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        error_event.payload,
        LiveKitEvent::Error(LiveKitError::InsufficientPermissions)
    )
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_revoke_bob() {
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

    // join livekit to ensure participant exists
    let (_room, _room_events) = livekit::Room::connect(
        &public_url,
        &bob_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();

    // Alice sends the command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::RevokeScreenSharePermission {
                participants: BTreeSet::from([bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        event.payload,
        LiveKitEvent::ScreenSharePermissionsUpdated {
            grant: false,
            participants: BTreeSet::from([bob.id()]),
        }
    );

    // Bob is notified by the livekit signaling session (separate to our signaling)
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_alice_in_breakout_bob_in_main() {
    let (_container, mut room, public_url) = common::build_room().await;

    // Alice and Bob join the meeting
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");
    flush_connected_events(&mut [&mut alice]).await;

    // join livekit to ensure participant exists
    let (_room, _room_events) = livekit::Room::connect(
        &public_url,
        &bob_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();

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

    // Alice sends the command
    alice
        .send_command::<LiveKitModule>(
            LiveKitCommand::RevokeScreenSharePermission {
                participants: BTreeSet::from([bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert_eq!(
        event.payload,
        LiveKitEvent::ScreenSharePermissionsUpdated {
            grant: false,
            participants: BTreeSet::from([bob.id()]),
        }
    );

    // Bob is notified by the livekit signaling session (separate to our signaling)
    assert!(bob.received_nothing());
}
