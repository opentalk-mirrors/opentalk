// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{assert_matches, collections::BTreeSet};

use livekit::{RoomEvent, RoomOptions};
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_test_util_livekit as mocking;
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    core::CoreEvent,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::LiveKitState;
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
};
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;

use crate::common::build_room;

mod common;

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_microphones_are_restricted() {
    let (_container, mut room, public_url) = build_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Bob joins the meeting
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
    // Bob should be able to publish audio
    mocking::publish_audio(&bob_room, &mut room_events)
        .await
        .unwrap();

    let unrestricted = BTreeSet::from([alice.id()]);

    // Alice sends the command
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableMicrophoneRestrictions {
                unrestricted_participants: unrestricted.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let success_event = alice.receive_event::<ModerationModule>().await.unwrap();

    assert_eq!(
        success_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted.clone(),
        }
    );

    // Bob should receive restriction state update
    let mute_event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        mute_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted
        }
    );

    // Bob should not be able to send audio
    assert!(
        mocking::publish_audio(&bob_room, &mut room_events)
            .await
            .is_err()
    );
}

/// When the restricted state is updated, the permissions are adjusted accordingly.
///
/// e.g. bob was restricted before, but is added to the unrestricted set.
#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_permissions_are_updated() {
    let (_container, mut room, public_url) = build_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");
    flush_connected_events(&mut [&mut alice]).await;

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
    // Bob should be able to publish audio
    mocking::publish_audio(&bob_room, &mut room_events)
        .await
        .unwrap();

    let unrestricted = BTreeSet::from([alice.id()]);

    // Alice sends the command to restrict bob
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableMicrophoneRestrictions {
                unrestricted_participants: unrestricted.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let success_event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        success_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted.clone()
        }
    );

    // Bob should receive restriction state update
    let mute_event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        mute_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted.clone()
        }
    );

    let unrestricted = BTreeSet::from([alice.id(), bob.id()]);

    // Alice sends the command to lift bobs restrictions
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableMicrophoneRestrictions {
                unrestricted_participants: unrestricted.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let success_event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        success_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted.clone()
        }
    );

    // Bob should receive restriction state update
    let mute_event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        mute_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted
        }
    );

    mocking::publish_audio(&bob_room, &mut room_events)
        .await
        .expect("Publishing audio must work again");
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_enable_unknown_participant() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = build_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    let disconnected = BTreeSet::from([disconnected_participant]);

    // Alice sends the command
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableMicrophoneRestrictions {
                unrestricted_participants: disconnected.clone(),
            },
            None,
        )
        .await
        .unwrap();

    // We allow for unknown participants in the unrestricted set. If participants disconnect their
    // permission is kept.
    let success_event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        success_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: disconnected
        }
    );
}

/// Alice enables restrictions except for bob, bob leaves the meeting, restrictions are lifted.
/// At the time of lifting the restrictions bob is still in the state but not a participant anymore.
#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_disable_unknown_participant() {
    let (_container, mut room, public_url) = build_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Bob joins the meeting
    let bob = room.join_bob(0).await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");
    flush_connected_events(&mut [&mut alice]).await;

    {
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
        // Bob should be able to publish audio
        mocking::publish_audio(&bob_room, &mut room_events)
            .await
            .unwrap();

        let unrestricted = BTreeSet::from([alice.id()]);

        // Alice sends the command
        alice
            .send_command::<ModerationModule>(
                ModerationCommand::EnableMicrophoneRestrictions {
                    unrestricted_participants: unrestricted,
                },
                None,
            )
            .await
            .unwrap();
        let _restrictions_enabled = alice.receive_event::<ModerationModule>().await.unwrap();
    }
    bob.disconnect().await.unwrap();
    let _bob_left = alice.receive::<CoreEvent>().await.unwrap();

    // Alice sends the command
    alice
        .send_command::<ModerationModule>(ModerationCommand::DisableMicrophoneRestrictions, None)
        .await
        .unwrap();

    // We allow for unknown participants in the unrestricted set. If participants disconnect their
    // permission is kept.
    let success_event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        success_event.payload,
        ModerationEvent::MicrophoneRestrictionsDisabled
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_disable_insufficient_permissions() {
    let (_container, mut room, _public_url) = build_room().await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;

    // Bob sends the command
    bob.send_command::<ModerationModule>(ModerationCommand::DisableMicrophoneRestrictions, None)
        .await
        .unwrap();

    let error_event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        error_event.payload,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_enable_insufficient_permissions() {
    let disconnected_participant = ParticipantId::from_u128(0x461ba262_6bb1_4c85_bbd5_b3d010b1a076);
    let (_container, mut room, _public_url) = build_room().await;

    // Bob joins the meeting
    let mut bob = room.join_bob(0).await;

    let unrestricted = BTreeSet::from([disconnected_participant]);

    // Bob sends the command
    bob.send_command::<ModerationModule>(
        ModerationCommand::EnableMicrophoneRestrictions {
            unrestricted_participants: unrestricted,
        },
        None,
    )
    .await
    .unwrap();

    let error_event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        error_event.payload,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

/// The [`LiveKitModule::ongoing_microphone_restrictions`] barrier should be freed after the
/// operation finished.
#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_barrier_should_be_freed() {
    let (_container, mut room, _public_url) = build_room().await;

    // Alice joins the meeting
    let mut alice = room.join_alice_moderator(0).await;

    // Alice sends the command
    for _ in 0..2 {
        alice
            .send_command::<ModerationModule>(
                ModerationCommand::EnableMicrophoneRestrictions {
                    unrestricted_participants: BTreeSet::new(),
                },
                None,
            )
            .await
            .unwrap();

        // wait till the command succeeded
        let success_event = alice.receive_event::<ModerationModule>().await.unwrap();
        assert_eq!(
            success_event.payload,
            ModerationEvent::MicrophoneRestrictionsEnabled {
                unrestricted_participants: BTreeSet::new()
            }
        );
    }
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_alice_in_breakout_bob_in_main() {
    let (_container, mut room, public_url) = build_room().await;

    // Alice and Bob join the meeting
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");
    flush_connected_events(&mut [&mut alice]).await;

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
    // Bob should be able to publish audio
    mocking::publish_audio(&bob_room, &mut room_events)
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

    let unrestricted = BTreeSet::from([alice.id()]);

    // Alice sends the command
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableMicrophoneRestrictions {
                unrestricted_participants: unrestricted.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let success_event = alice.receive_event::<ModerationModule>().await.unwrap();

    assert_eq!(
        success_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted.clone(),
        }
    );

    // Bob should not receive restriction state update since he is in another room
    assert!(bob.received_nothing());

    // Bob should be able to publish audio since he is in another breakout room.
    mocking::publish_audio(&bob_room, &mut room_events)
        .await
        .unwrap();
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_alice_and_bob_in_breakout() {
    let (_container, mut room, public_url) = build_room().await;

    // Alice and Bob join the meeting
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");
    flush_connected_events(&mut [&mut alice]).await;

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
    // Bob should be able to publish audio
    mocking::publish_audio(&bob_room, &mut room_events)
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
    bob.switch_breakout_room(&mut [&mut alice], RoomKind::Breakout(0.into()))
        .await;

    let unrestricted = BTreeSet::from([alice.id()]);

    // Alice sends the command
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableMicrophoneRestrictions {
                unrestricted_participants: unrestricted.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let success_event = alice.receive_event::<ModerationModule>().await.unwrap();

    assert_eq!(
        success_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted.clone(),
        }
    );

    // Bob should receive restriction state update
    let mute_event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        mute_event.payload,
        ModerationEvent::MicrophoneRestrictionsEnabled {
            unrestricted_participants: unrestricted
        }
    );

    // Bob should not be able to send audio
    assert!(
        mocking::publish_audio(&bob_room, &mut room_events)
            .await
            .is_err()
    );
}
