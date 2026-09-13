// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::assert_matches;

use axum::{Json, routing::post};
use opentalk_roomserver_crypto_provider::ensure_crypto_provider;
use opentalk_roomserver_module_recording::RecordingModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events, flush_disconnected_events},
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        event::BreakoutEvent,
    },
    core::CoreEvent,
    disconnect_reason::DisconnectReason,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_recording::{
    RECORDING_MODULE_ID, RecordingStatus, StreamStatus,
    command::RecordingCommand,
    event::{RecordingError, RecordingEvent},
    peer_state::RecordingPeerState,
    service::{
        command::RecordingServiceCommand, event::RecordingServiceEvent,
        state::ServiceStreamingTarget,
    },
    settings::RecordingSettings,
    state::RecordingState,
};
use opentalk_service_auth::{ApiKey, service::ApiKeys};
use opentalk_types_api_internal::recording::RecordingTarget;
use opentalk_types_common::streaming::{
    RoomStreamingTarget, StreamingTarget, StreamingTargetId, StreamingTargetKind,
};
use pretty_assertions::assert_eq;
use reqwest::Url;
use tokio::{net::TcpListener, sync::mpsc, task::JoinHandle};

struct MockRecorderTask {
    url: Url,
    recorder_request_rx: mpsc::Receiver<RecordingTarget>,
    join_handle: JoinHandle<()>,
}

impl MockRecorderTask {
    async fn spawn() -> MockRecorderTask {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let (recorder_request_tx, recorder_request_rx) = mpsc::channel(1);

        let middleware = ApiKeys::new(vec![ApiKey::new("recorder", "secret")])
            .auth_middleware()
            .unwrap();

        let router = axum::Router::new().nest(
            "/v1",
            axum::Router::new()
                .route(
                    "/init",
                    post(|request: Json<RecordingTarget>| async move {
                        recorder_request_tx.send(request.0).await.unwrap();
                    }),
                )
                .layer(middleware),
        );

        let join_handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        Self {
            url: format!("http://127.0.0.1:{port}/").parse().unwrap(),
            recorder_request_rx,
            join_handle,
        }
    }
}

impl Drop for MockRecorderTask {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

macro_rules! expect_recording_event {
    ($expr:expr, $val:expr) => {{
        let recording_event = $expr.receive::<RecordingEvent>().await.unwrap();
        assert_eq!(recording_event.payload, $val);
    }};
}

fn create_room(mock_recorder: &MockRecorderTask) -> TestRoom {
    ensure_crypto_provider();

    let recording_settings = RecordingSettings {
        url: mock_recorder.url.clone(),
        api_key: ApiKey::new("recorder", "secret"),
    };

    TestRoom::builder()
        .add_init_module_data(&recording_settings)
        .expect("valid recording module settings")
        .streaming_target(RoomStreamingTarget {
            id: StreamingTargetId::nil(),
            streaming_target: StreamingTarget {
                name: "My Test Streaming Target".into(),
                kind: StreamingTargetKind::Custom {
                    streaming_endpoint: "rtmp://localhost".parse().unwrap(),
                    streaming_key: "mykey".parse().unwrap(),
                    public_url: "http://localhost:8000/mystream".parse().unwrap(),
                },
            },
        })
        .register_module::<RecordingModule>()
        .spawn()
}

async fn recorder_update_recording_status(
    recorder: &mut MockParticipantJoined,
    other: &mut [&mut MockParticipantJoined],
    new_status: RecordingStatus,
) {
    recorder
        .send_command::<RecordingModule>(
            RecordingCommand::Service {
                command: RecordingServiceEvent::RecordingUpdated(new_status.clone()),
            },
            None,
        )
        .await
        .unwrap();

    expect_recording_event!(
        recorder,
        RecordingEvent::RecordingUpdated(new_status.clone())
    );

    for other in other {
        expect_recording_event!(other, RecordingEvent::RecordingUpdated(new_status.clone()));
    }
}

async fn start_recording(
    mock_recorder: &mut MockRecorderTask,
    room: &mut TestRoom,
    issuer: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    issuer
        .send_command::<RecordingModule>(RecordingCommand::StartRecording, None)
        .await
        .unwrap();

    expect_recording_event!(
        issuer,
        RecordingEvent::RecordingUpdated(RecordingStatus::Requested)
    );

    for other in others {
        expect_recording_event!(
            other,
            RecordingEvent::RecordingUpdated(RecordingStatus::Requested)
        );
    }

    // Expect HTTP request for a new recorder
    let recording_target = mock_recorder.recorder_request_rx.recv().await.unwrap();
    assert_eq!(recording_target.room_id, room.id());
    assert_eq!(recording_target.breakout_room, None);
}

async fn recorder_update_streaming_status(
    recorder: &mut MockParticipantJoined,
    other: &mut [&mut MockParticipantJoined],
    new_status: StreamStatus,
) {
    recorder
        .send_command::<RecordingModule>(
            RecordingCommand::Service {
                command: RecordingServiceEvent::StreamUpdated {
                    target_id: StreamingTargetId::nil(),
                    status: new_status.clone(),
                },
            },
            None,
        )
        .await
        .unwrap();

    expect_recording_event!(
        recorder,
        RecordingEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: new_status.clone()
        }
    );

    for other in other {
        expect_recording_event!(
            other,
            RecordingEvent::StreamUpdated {
                target_id: StreamingTargetId::nil(),
                status: new_status.clone(),
            }
        );
    }
}

async fn start_streaming(
    mock_recorder: &mut MockRecorderTask,
    room: &mut TestRoom,
    issuer: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    issuer
        .send_command::<RecordingModule>(
            RecordingCommand::StartStream {
                target_ids: [StreamingTargetId::nil()].into(),
            },
            None,
        )
        .await
        .unwrap();

    expect_recording_event!(
        issuer,
        RecordingEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::Requested
        }
    );

    for other in others {
        expect_recording_event!(
            other,
            RecordingEvent::StreamUpdated {
                target_id: StreamingTargetId::nil(),
                status: StreamStatus::Requested
            }
        );
    }

    // Expect HTTP request for a new recorder
    let recording_target = mock_recorder.recorder_request_rx.recv().await.unwrap();
    assert_eq!(recording_target.room_id, room.id());
    assert_eq!(recording_target.breakout_room, None);
}

#[test_log::test(tokio::test)]
async fn start_and_stop_recording() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;

    start_recording(&mut mock_recorder, &mut room, &mut alice, &mut []).await;

    let mut recorder = room.join_recorder(RoomKind::Main, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Recorder updates recording status
    recorder_update_recording_status(&mut recorder, &mut [&mut alice], RecordingStatus::Active)
        .await;
    recorder_update_recording_status(&mut recorder, &mut [&mut alice], RecordingStatus::Paused)
        .await;

    // Stop recording
    alice
        .send_command::<RecordingModule>(RecordingCommand::StopRecording, None)
        .await
        .unwrap();

    expect_recording_event!(
        recorder,
        RecordingEvent::Service {
            event: RecordingServiceCommand::StopRecording
        }
    );

    recorder_update_recording_status(&mut recorder, &mut [&mut alice], RecordingStatus::Inactive)
        .await;

    let recorder_id = recorder.id();
    let recorder_connection_id = recorder.connection_id();
    recorder.disconnect().await.unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason
        } if participant_id == recorder_id
         && connection_id == recorder_connection_id
         && reason == DisconnectReason::Leave
    );

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn reset_recording_state_on_recorder_disconnect() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;

    // Start recording
    start_recording(&mut mock_recorder, &mut room, &mut alice, &mut []).await;
    start_streaming(&mut mock_recorder, &mut room, &mut alice, &mut []).await;

    let mut recorder = room.join_recorder(RoomKind::Main, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    recorder_update_recording_status(&mut recorder, &mut [&mut alice], RecordingStatus::Active)
        .await;

    recorder.disconnect().await.unwrap();

    expect_recording_event!(
        alice,
        RecordingEvent::RecordingUpdated(RecordingStatus::Inactive)
    );

    expect_recording_event!(
        alice,
        RecordingEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::Inactive,
        }
    );
}

#[test_log::test(tokio::test)]
async fn reset_state_on_recorder_request_failure() {
    let mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    // Force failure by stopping the mock recorder
    drop(mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<RecordingModule>(RecordingCommand::StartRecording, None)
        .await
        .unwrap();

    expect_recording_event!(
        alice,
        RecordingEvent::RecordingUpdated(RecordingStatus::Requested)
    );

    expect_recording_event!(
        alice,
        RecordingEvent::RecordingUpdated(RecordingStatus::Inactive)
    );

    expect_recording_event!(
        alice,
        RecordingEvent::Error(RecordingError::FailedToRequestRecordingService)
    );
}

#[test_log::test(tokio::test)]
async fn do_not_use_recorder_of_a_different_breakout_room() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;

    // Join a recorder for a different room
    let mut wrong_recorder = room.join_recorder(RoomKind::Breakout(123.into()), 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Make sure the recorder for the breakout room cannot modify the recording states of the main
    // room
    wrong_recorder
        .send_command::<RecordingModule>(
            RecordingCommand::Service {
                command: RecordingServiceEvent::RecordingUpdated(RecordingStatus::Paused),
            },
            None,
        )
        .await
        .unwrap();

    expect_recording_event!(
        wrong_recorder,
        RecordingEvent::Error(RecordingError::InsufficientPermissions)
    );

    // Start recording, receive http request even though another recorder is connected (but for a
    // different breakout room)
    start_recording(
        &mut mock_recorder,
        &mut room,
        &mut alice,
        &mut [&mut wrong_recorder],
    )
    .await;

    // Join a recorder which has been actually requested for the main room
    let mut allowed_recorder = room.join_recorder(RoomKind::Main, 1).await;
    flush_connected_events(&mut [&mut alice, &mut wrong_recorder]).await;

    recorder_update_recording_status(
        &mut allowed_recorder,
        &mut [&mut alice],
        RecordingStatus::Active,
    )
    .await;
}

#[test_log::test(tokio::test)]
async fn consent_peer_events_is_set() {
    let mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    let bob = room.join_bob(1).await;

    // Check that alice can see the consent status of bob (default false, no consent)
    let CoreEvent::ParticipantConnected {
        participant_id,
        connection_id,
        peer_data,
    } = alice.receive::<CoreEvent>().await.unwrap().payload
    else {
        panic!("expected CoreEvent::ParticipantConnected");
    };

    assert_eq!(participant_id, bob.id());
    assert_eq!(connection_id, bob.connection_id());
    let bob_peer_state = serde_json::from_value::<RecordingPeerState>(
        peer_data.get(&RECORDING_MODULE_ID).unwrap().clone_inner(),
    )
    .unwrap();

    assert!(!bob_peer_state.consents_recording);
}

#[test_log::test(tokio::test)]
async fn recorder_can_see_consent() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(1).await;

    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<RecordingModule>(RecordingCommand::SetConsent { consent: true }, None)
        .await
        .unwrap();

    expect_recording_event!(
        alice,
        RecordingEvent::ConsentUpdated {
            participant: alice.id(),
            consents: true
        }
    );
    expect_recording_event!(
        bob,
        RecordingEvent::ConsentUpdated {
            participant: alice.id(),
            consents: true
        }
    );

    // Start recording
    start_recording(&mut mock_recorder, &mut room, &mut alice, &mut [&mut bob]).await;

    let recorder = room.join_recorder(RoomKind::Main, 0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Check participant consent states
    {
        let participants = &recorder.join_success().participants;

        // Check alice
        let alice = participants.iter().find(|p| p.id == alice.id()).unwrap();
        let alice_state = alice.get_module::<RecordingPeerState>().unwrap().unwrap();
        assert!(alice_state.consents_recording);

        // Check bob
        let bob = participants.iter().find(|p| p.id == bob.id()).unwrap();
        let bob_state = bob.get_module::<RecordingPeerState>().unwrap().unwrap();
        assert!(!bob_state.consents_recording);
    }
}

#[test_log::test(tokio::test)]
async fn consent_retains_after_disconnect() {
    let mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(1).await;

    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<RecordingModule>(RecordingCommand::SetConsent { consent: true }, None)
        .await
        .unwrap();

    expect_recording_event!(
        alice,
        RecordingEvent::ConsentUpdated {
            participant: alice.id(),
            consents: true
        }
    );
    expect_recording_event!(
        bob,
        RecordingEvent::ConsentUpdated {
            participant: alice.id(),
            consents: true
        }
    );

    // Reconnect alice and verify that the consent is retained
    alice.disconnect().await.unwrap();

    flush_disconnected_events(&mut [&mut bob]).await;

    let alice = room.join_alice_moderator(0).await;

    // Check that bob can see the old consent status of alice
    let CoreEvent::ParticipantConnected {
        participant_id,
        connection_id,
        peer_data,
    } = bob.receive::<CoreEvent>().await.unwrap().payload
    else {
        panic!("expected CoreEvent::ParticipantConnected");
    };

    assert_eq!(participant_id, alice.id());
    assert_eq!(connection_id, alice.connection_id());
    let peer_state = serde_json::from_value::<RecordingPeerState>(
        peer_data.get(&RECORDING_MODULE_ID).unwrap().clone_inner(),
    )
    .unwrap();

    assert!(peer_state.consents_recording);
}

#[test_log::test(tokio::test)]
async fn only_recorder_participants_can_see_service_streaming_targets() {
    let mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    let recorder = room.join_recorder(RoomKind::Main, 0).await;

    flush_connected_events(&mut [&mut alice]).await;

    // Assert alice (even as moderator) cannot see the streaming configuration meant for services
    let recorder_state = alice
        .join_success()
        .get_module::<RecordingState>()
        .unwrap()
        .unwrap();
    assert!(recorder_state.service.is_none());

    // Assert that the recorder can see the streaming configuration
    let recorder_state = recorder
        .join_success()
        .get_module::<RecordingState>()
        .unwrap()
        .unwrap();

    let mut streaming_targets = recorder_state.service.unwrap().streaming_targets;
    assert_eq!(streaming_targets.len(), 1);
    assert_eq!(
        streaming_targets.remove(&StreamingTargetId::nil()).unwrap(),
        ServiceStreamingTarget {
            location: "rtmp://localhost/mykey".parse().unwrap()
        }
    );
}

#[test_log::test(tokio::test)]
async fn only_moderators_can_control_recordings() {
    let mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;

    flush_connected_events(&mut [&mut alice]).await;

    for command in [
        RecordingCommand::StartRecording,
        RecordingCommand::PauseRecording,
        RecordingCommand::StopRecording,
        RecordingCommand::StartStream {
            target_ids: [StreamingTargetId::nil()].into(),
        },
        RecordingCommand::PauseStream {
            target_ids: [StreamingTargetId::nil()].into(),
        },
        RecordingCommand::StopStream {
            target_ids: [StreamingTargetId::nil()].into(),
        },
    ] {
        bob.send_command::<RecordingModule>(command, None)
            .await
            .unwrap();

        expect_recording_event!(
            bob,
            RecordingEvent::Error(RecordingError::InsufficientPermissions)
        );
    }
}

#[test_log::test(tokio::test)]
async fn only_recorders_can_issue_state_updates() {
    let mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;

    for command in [
        RecordingServiceEvent::RecordingUpdated(RecordingStatus::Active),
        RecordingServiceEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::Active,
        },
    ] {
        alice
            .send_command::<RecordingModule>(RecordingCommand::Service { command }, None)
            .await
            .unwrap();

        expect_recording_event!(
            alice,
            RecordingEvent::Error(RecordingError::InsufficientPermissions)
        );
    }
}

#[test_log::test(tokio::test)]
async fn start_stream_with_invalid_id() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<RecordingModule>(
            RecordingCommand::StartStream {
                target_ids: [
                    StreamingTargetId::nil(),
                    // request invalid streaming id
                    StreamingTargetId::from_u128(0xDEAD),
                ]
                .into(),
            },
            None,
        )
        .await
        .unwrap();

    // Expect error
    let RecordingEvent::Error(RecordingError::InvalidStreamingId) =
        alice.receive().await.unwrap().payload
    else {
        panic!()
    };

    // Nothing else happens but the error
    assert!(alice.received_nothing());

    // Try again with valid streaming id
    start_streaming(&mut mock_recorder, &mut room, &mut alice, &mut []).await;
}

#[test_log::test(tokio::test)]
async fn pause_stream_with_invalid_id() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    start_streaming(&mut mock_recorder, &mut room, &mut alice, &mut []).await;
    let mut recorder = room.join_recorder(RoomKind::Main, 0).await;
    flush_connected_events(&mut [&mut alice]).await;
    recorder_update_streaming_status(&mut recorder, &mut [&mut alice], StreamStatus::Active).await;

    alice
        .send_command::<RecordingModule>(
            RecordingCommand::PauseStream {
                target_ids: [
                    StreamingTargetId::nil(),
                    // request invalid streaming id
                    StreamingTargetId::from_u128(0xDEAD),
                ]
                .into(),
            },
            None,
        )
        .await
        .unwrap();

    // Expect error
    expect_recording_event!(
        alice,
        RecordingEvent::Error(RecordingError::InvalidStreamingId)
    );

    // Nothing else happens but the error
    assert!(alice.received_nothing());
    assert!(recorder.received_nothing());

    // Try again without the invalid id
    alice
        .send_command::<RecordingModule>(
            RecordingCommand::PauseStream {
                target_ids: [StreamingTargetId::nil()].into(),
            },
            None,
        )
        .await
        .unwrap();

    assert!(alice.received_nothing());

    expect_recording_event!(
        recorder,
        RecordingEvent::Service {
            event: RecordingServiceCommand::PauseStreams {
                target_ids: [StreamingTargetId::nil()].into()
            },
        }
    );
}

#[test_log::test(tokio::test)]
async fn stop_stream_with_invalid_id() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    start_streaming(&mut mock_recorder, &mut room, &mut alice, &mut []).await;
    let mut recorder = room.join_recorder(RoomKind::Main, 0).await;
    flush_connected_events(&mut [&mut alice]).await;
    recorder_update_streaming_status(&mut recorder, &mut [&mut alice], StreamStatus::Active).await;

    alice
        .send_command::<RecordingModule>(
            RecordingCommand::StopStream {
                target_ids: [
                    StreamingTargetId::nil(),
                    // request invalid streaming id
                    StreamingTargetId::from_u128(0xDEAD),
                ]
                .into(),
            },
            None,
        )
        .await
        .unwrap();

    // Expect error
    expect_recording_event!(
        alice,
        RecordingEvent::Error(RecordingError::InvalidStreamingId)
    );

    // Nothing else happens but the error
    assert!(alice.received_nothing());
    assert!(recorder.received_nothing());

    // Try again without the invalid id
    alice
        .send_command::<RecordingModule>(
            RecordingCommand::StopStream {
                target_ids: [StreamingTargetId::nil()].into(),
            },
            None,
        )
        .await
        .unwrap();

    assert!(alice.received_nothing());

    expect_recording_event!(
        recorder,
        RecordingEvent::Service {
            event: RecordingServiceCommand::StopStreams {
                target_ids: [StreamingTargetId::nil()].into()
            },
        }
    );
}

#[test_log::test(tokio::test)]
async fn streaming_target_breakout_in_use_state() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;

    flush_connected_events(&mut [&mut alice]).await;

    let BreakoutEvent::Started { rooms, .. } = alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "My Room".into(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await
    else {
        panic!()
    };

    // Start stream with bob and alice in the main room
    start_streaming(&mut mock_recorder, &mut room, &mut alice, &mut [&mut bob]).await;

    // Switch bob to the breakout room
    let BreakoutEvent::SwitchedRoom { own_data, .. } = bob
        .switch_breakout_room(&mut [&mut alice], RoomKind::Breakout(rooms[0].id))
        .await
    else {
        panic!();
    };

    // In the breakout room, bob shouldn't see the specific state anymore, just InUse
    let bob_state = own_data.get::<RecordingState>().unwrap().unwrap();
    assert_eq!(
        bob_state
            .stream_states
            .get(&StreamingTargetId::nil())
            .unwrap()
            .status,
        StreamStatus::InUse
    );

    // Switch bob back to main room
    let BreakoutEvent::SwitchedRoom { own_data, .. } = bob
        .switch_breakout_room(&mut [&mut alice], RoomKind::Main)
        .await
    else {
        panic!();
    };

    // In the main room bob show now see the requested state again
    let bob_state = own_data.get::<RecordingState>().unwrap().unwrap();
    assert_eq!(
        bob_state
            .stream_states
            .get(&StreamingTargetId::nil())
            .unwrap()
            .status,
        StreamStatus::Requested
    );
}

#[test_log::test(tokio::test)]
async fn streaming_target_breakout_in_use_events() {
    let mut mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;

    flush_connected_events(&mut [&mut alice]).await;

    let BreakoutEvent::Started { rooms, .. } = alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "My Room".into(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await
    else {
        panic!()
    };

    bob.switch_breakout_room(&mut [&mut alice], RoomKind::Breakout(rooms[0].id))
        .await;

    start_streaming(&mut mock_recorder, &mut room, &mut alice, &mut []).await;

    // Verify that bob, in a different breakout room receives the status not as "Requested", but as
    // "InUse"
    expect_recording_event!(
        bob,
        RecordingEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::InUse
        }
    );

    let mut recorder = room.join_recorder(RoomKind::Main, 0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;
    recorder_update_streaming_status(&mut recorder, &mut [&mut alice], StreamStatus::Active).await;

    // Verify that bob, in a different breakout room doesn't receive updates if it doesn't change
    // the "InUse" state
    assert!(bob.received_nothing());

    recorder_update_streaming_status(&mut recorder, &mut [&mut alice], StreamStatus::Inactive)
        .await;

    // Verify that bob, in a different breakout room receives the status not as "Requested", but as
    // "InUse"
    expect_recording_event!(
        bob,
        RecordingEvent::StreamUpdated {
            target_id: StreamingTargetId::nil(),
            status: StreamStatus::Inactive
        }
    );
}

#[test_log::test(tokio::test)]
async fn joining_contains_own_consent_state() {
    let mock_recorder = MockRecorderTask::spawn().await;
    let mut room = create_room(&mock_recorder);

    // Join and set consent to true

    let alice = room.join_alice_moderator(0).await;

    let recording_state = alice
        .join_success()
        .get_module::<RecordingState>()
        .unwrap()
        .unwrap();

    assert!(!recording_state.consents_recording);

    alice
        .send_command::<RecordingModule>(RecordingCommand::SetConsent { consent: true }, None)
        .await
        .unwrap();

    alice.disconnect().await.unwrap();

    // Join and check that consent is still true

    let alice = room.join_alice_moderator(0).await;

    let recording_state = alice
        .join_success()
        .get_module::<RecordingState>()
        .unwrap()
        .unwrap();

    assert!(recording_state.consents_recording);
}
