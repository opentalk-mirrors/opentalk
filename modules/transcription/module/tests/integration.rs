// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::assert_matches;

use axum::{Json, routing::post};
use opentalk_roomserver_crypto_provider::ensure_crypto_provider;
use opentalk_roomserver_module_transcription::TranscriptionModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events, flush_disconnected_events},
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        event::BreakoutEvent,
    },
    core::CoreEvent,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_transcription::{
    command::TranscriptionCommand,
    event::{TranscriptionError, TranscriptionEvent},
    segment::TranscriptionSegment,
    service::{command::TranscriptionServiceCommand, event::TranscriptionServiceEvent},
    settings::TranscriptionSettings,
    state::TranscriptionStatus,
};
use opentalk_service_auth::{ApiKey, service::ApiKeys};
use opentalk_transcription_web_api::v1::TranscriptionTarget;
use opentalk_types_common::time::Timestamp;
use reqwest::Url;
use tokio::{net::TcpListener, sync::mpsc, task::JoinHandle};

struct MockTranscriptionTask {
    url: Url,
    transcription_request_rx: mpsc::Receiver<TranscriptionTarget>,
    join_handle: JoinHandle<()>,
}

impl MockTranscriptionTask {
    async fn spawn() -> MockTranscriptionTask {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let (transcription_request_tx, transcription_request_rx) = mpsc::channel(1);

        let middleware = ApiKeys::new(vec![ApiKey::new("transcription", "secret")])
            .auth_middleware()
            .unwrap();

        let router = axum::Router::new().nest(
            "/v1",
            axum::Router::new()
                .route(
                    "/init",
                    post(|request: Json<TranscriptionTarget>| async move {
                        transcription_request_tx.send(request.0).await.unwrap();
                    }),
                )
                .layer(middleware),
        );

        let join_handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        Self {
            url: format!("http://127.0.0.1:{port}/").parse().unwrap(),
            transcription_request_rx,
            join_handle,
        }
    }
}

impl Drop for MockTranscriptionTask {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

macro_rules! expect_transcription_event {
    ($expr:expr, $val:expr) => {{
        let transcription_event = $expr.receive::<TranscriptionEvent>().await.unwrap();
        assert_eq!(transcription_event.payload, $val);
    }};
}

fn create_room(mock_transcription: &MockTranscriptionTask) -> TestRoom {
    ensure_crypto_provider();

    let transcription_settings = TranscriptionSettings {
        url: mock_transcription.url.clone(),
        api_key: ApiKey::new("transcription", "secret"),
    };

    TestRoom::builder()
        .add_init_module_data(&transcription_settings)
        .expect("valid transcription module settings")
        .register_module::<TranscriptionModule>()
        .spawn()
}

async fn start_transcription(
    transcription: &mut MockTranscriptionTask,
    room: &mut TestRoom,
    room_kind: RoomKind,
    issuer: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    issuer
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::Start {
                language: Some("de".into()),
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        issuer,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Requested
        }
    );

    for other in others {
        expect_transcription_event!(
            other,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Requested
            }
        );
    }

    // Expect HTTP request for a new transcription service
    let transcription_target = transcription.transcription_request_rx.recv().await.unwrap();
    assert_eq!(transcription_target.room_id, room.id());

    match room_kind {
        RoomKind::Main => assert_eq!(transcription_target.breakout_room, None),
        RoomKind::Breakout(breakout_id) => {
            assert_eq!(transcription_target.breakout_room, Some(breakout_id.into()))
        }
    }
}

#[test_log::test(tokio::test)]
async fn start_and_stop_transcription() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;

    start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    // Join with transcription participant
    let mut transcription = room.join_transcription(room_kind, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Send the service event that the transcription has started (this would normally be sent by the
    // transcription service)
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started,
            },
            None,
        )
        .await
        .unwrap();

    // All participants receive the started event
    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running
        }
    );
    expect_transcription_event!(
        transcription,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running
        }
    );

    // Stop the transcription
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Stop, None)
        .await
        .unwrap();

    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Stopped,
            },
            None,
        )
        .await
        .unwrap();

    transcription.disconnect().await.unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive
        }
    );
}

#[test_log::test(tokio::test)]
async fn transcription_segment() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;

    start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    // Join with transcription participant
    let transcription = room.join_transcription(room_kind, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Send the service event that the transcription has started (this would normally be sent by the
    // transcription service)
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started,
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running
        }
    );

    let segment = TranscriptionSegment {
        participant_id: alice.id(),
        track_id: "track1".into(),
        starts_at: Timestamp::unix_epoch(),
        ends_at: Timestamp::unix_epoch(),
        text: "Hello there".into(),
    };

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Segment(segment.clone()),
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(alice, TranscriptionEvent::Segment(segment));
}

#[test_log::test(tokio::test)]
async fn breakout_room_transcription() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "breakout".into(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;
    let room_kind = RoomKind::Breakout(BreakoutId::from(0));

    let event = alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    let BreakoutEvent::SwitchedRoom { .. } = event else {
        panic!("Received wrong breakout event");
    };

    start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    // Join with transcription participant
    let mut transcription = room.join_transcription(room_kind, 0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;
    transcription
        .switch_breakout_room(&mut [&mut alice, &mut bob], room_kind)
        .await;

    // Send the service event that the transcription has started (this would normally be sent by the
    // transcription service)
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started,
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running
        }
    );
    expect_transcription_event!(
        transcription,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running
        }
    );

    assert!(bob.received_nothing());

    let segment = TranscriptionSegment {
        participant_id: alice.id(),
        track_id: "track1".into(),
        starts_at: Timestamp::unix_epoch(),
        ends_at: Timestamp::unix_epoch(),
        text: "Hello there".into(),
    };

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Segment(segment.clone()),
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(alice, TranscriptionEvent::Segment(segment.clone()));
    expect_transcription_event!(transcription, TranscriptionEvent::Segment(segment));
    assert!(bob.received_nothing());

    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Stop, None)
        .await
        .unwrap();

    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Stopped,
            },
            None,
        )
        .await
        .unwrap();

    transcription.disconnect().await.unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive
        }
    );

    flush_disconnected_events(&mut [&mut alice, &mut bob]).await;
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn transcription_segment_insufficient_permission() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;

    start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    // Join with transcription participant
    let transcription = room.join_transcription(room_kind, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Send the service event that the transcription has started (this would normally be sent by the
    // transcription service)
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started,
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running
        }
    );

    let segment = TranscriptionSegment {
        participant_id: alice.id(),
        track_id: "track1".into(),
        starts_at: Timestamp::unix_epoch(),
        ends_at: Timestamp::unix_epoch(),
        text: "Hello there".into(),
    };

    alice
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Segment(segment.clone()),
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::Error(TranscriptionError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn failed_transcription_request() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    transcription_task.url = "http://127.0.0.1/force_failed_request".parse().unwrap();

    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Start { language: None }, None)
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Requested
        }
    );

    expect_transcription_event!(
        alice,
        TranscriptionEvent::Error(TranscriptionError::ServiceRequestFailed)
    );

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive
        }
    );
}

#[test_log::test(tokio::test)]
async fn unexpected_service_disconnect() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;

    start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    // Join with transcription participant
    let transcription = room.join_transcription(room_kind, 0).await;
    let transcription_participant_id = transcription.id();

    flush_connected_events(&mut [&mut alice]).await;

    // Send the service event that the transcription has started (this would normally be sent by the
    // transcription service)
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started,
            },
            None,
        )
        .await
        .unwrap();

    // All participants receive the started event
    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running
        }
    );

    transcription.disconnect().await.unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::Error(TranscriptionError::ServiceDisconnected)
    );

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive
        }
    );

    let event = alice.receive::<CoreEvent>().await.unwrap();

    assert_matches!(
        event.payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            ..
        } if participant_id == transcription_participant_id
    );
}
