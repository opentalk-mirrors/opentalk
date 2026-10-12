// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{assert_matches, time::Duration};

use axum::{Json, routing::post};
use insta::assert_snapshot;
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
    config::MeetingProtocolConfig,
    event::{TranscriptionError, TranscriptionEvent},
    segment::TranscriptionSegment,
    service::{command::TranscriptionServiceCommand, event::TranscriptionServiceEvent},
    settings::TranscriptionSettings,
    state::{MeetingProtocolState, TranscriptionState},
    status::{MeetingProtocolMode, TranscriptionModes, TranscriptionStatus},
};
use opentalk_service_auth::{ApiKey, service::ApiKeys};
use opentalk_transcription_web_api::v1::TranscriptionTarget;
use opentalk_types_common::{
    time::{TimeZone, Timestamp},
    transcription::RequestId,
};
use pretty_assertions::assert_eq;
use reqwest::Url;
use tokio::{net::TcpListener, sync::mpsc, task::JoinHandle};

const ROOM_IDLE_TIMEOUT: Duration = Duration::from_secs(10);

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
        .room_idle_timeout(ROOM_IDLE_TIMEOUT)
        .spawn()
}

async fn start_transcription(
    transcription: &mut MockTranscriptionTask,
    room: &mut TestRoom,
    room_kind: RoomKind,
    issuer: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) -> RequestId {
    issuer
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::Start {
                language: Some("de".into()),
            },
            None,
        )
        .await
        .unwrap();

    for participant in std::iter::once(issuer).chain(others.iter_mut().map(|other| &mut **other)) {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: live_transcription_modes()
            }
        );
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Requested {
                    language: Some("de".into())
                }
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

    transcription_target.request_id
}

async fn start_meeting_protocol(
    transcription: &mut MockTranscriptionTask,
    room: &mut TestRoom,
    room_kind: RoomKind,
    language: Option<String>,
    pseudonymous: bool,
    issuer: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) -> RequestId {
    issuer
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::StartMeetingProtocol {
                language: language.clone(),
                timezone: Some(TimeZone::utc()),
                pseudonymous,
            },
            None,
        )
        .await
        .unwrap();

    for participant in std::iter::once(issuer).chain(others.iter_mut().map(|other| &mut **other)) {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: meeting_protocol_modes(pseudonymous, false)
            }
        );
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Requested {
                    language: language.clone()
                }
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

    transcription_target.request_id
}

/// Stops the meeting protocol as the last active mode and returns the generated PDF.
///
/// The service is no longer needed and told to stop.
async fn stop_meeting_protocol(
    room: &TestRoom,
    transcription: &mut MockParticipantJoined,
    issuer: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) -> Vec<u8> {
    issuer
        .send_command::<TranscriptionModule>(TranscriptionCommand::StopMeetingProtocol, None)
        .await
        .unwrap();

    for participant in std::iter::once(&mut *issuer)
        .chain(others.iter_mut().map(|other| &mut **other))
        .chain([&mut *transcription])
    {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes::default()
            }
        );
    }

    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    let event = issuer.receive::<TranscriptionEvent>().await.unwrap();
    let TranscriptionEvent::PdfCreated { asset_id, .. } = event.payload else {
        panic!("Expected PdfCreated, got {:?}", event.payload);
    };

    for participant in others
        .iter_mut()
        .map(|other| &mut **other)
        .chain([transcription])
    {
        assert_matches!(
            participant.receive::<TranscriptionEvent>().await.unwrap().payload,
            TranscriptionEvent::PdfCreated { asset_id: id, .. } if id == asset_id
        );
    }

    room.stored_asset(asset_id)
        .await
        .expect("Meeting protocol PDF must be stored")
}

/// Starts the live transcription, stops it and starts it again before the first service stopped.
///
/// Returns the old service, which was told to stop but is still connected, and the new service,
/// which was requested by the second start and is running.
async fn restart_before_service_stopped(
    transcription_task: &mut MockTranscriptionTask,
    room: &mut TestRoom,
    room_kind: RoomKind,
    alice: &mut MockParticipantJoined,
) -> (MockParticipantJoined, MockParticipantJoined) {
    let request_id = start_transcription(transcription_task, room, room_kind, alice, &mut []).await;

    let mut old_service = join_transcription_service(
        room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [alice],
    )
    .await;

    // Stop the transcription, the old service is told to stop
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Stop, None)
        .await
        .unwrap();

    for participant in [&mut *alice, &mut old_service] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes::default()
            }
        );
    }

    expect_transcription_event!(
        old_service,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    // Start again before the old service stopped, a new service is requested
    let request_id = start_transcription(
        transcription_task,
        room,
        room_kind,
        alice,
        &mut [&mut old_service],
    )
    .await;

    let new_service = join_transcription_service(
        room,
        room_kind,
        1,
        request_id,
        Some("de".into()),
        &mut [alice, &mut old_service],
    )
    .await;

    (old_service, new_service)
}

/// Sets the meeting protocol consent of `participant`, the participant and all `others` are
/// notified about it.
async fn set_consent(
    consent: bool,
    participant: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
) {
    participant
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::SetMeetingProtocolConsent { consent },
            None,
        )
        .await
        .unwrap();

    let participant_id = participant.id();
    for receiver in std::iter::once(participant).chain(others.iter_mut().map(|other| &mut **other))
    {
        expect_transcription_event!(
            receiver,
            TranscriptionEvent::MeetingProtocolConsentUpdated {
                participant: participant_id,
                consents: consent,
            }
        );
    }
}

/// Joins a transcription service that starts the transcription for `request_id`.
///
/// Services of the same room need different devices, the device determines the participant id.
async fn join_transcription_service(
    room: &mut TestRoom,
    room_kind: RoomKind,
    device_number: usize,
    request_id: RequestId,
    language: Option<String>,
    participants: &mut [&mut MockParticipantJoined],
) -> MockParticipantJoined {
    // Join with transcription participant
    let mut transcription = room.join_transcription(room_kind, device_number).await;
    flush_connected_events(participants).await;

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started {
                    request_id,
                    language: language.clone(),
                },
            },
            None,
        )
        .await
        .unwrap();

    // The transcription service receives the status update as well
    for participant in participants
        .iter_mut()
        .map(|participant| &mut **participant)
        .chain([&mut transcription])
    {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Running {
                    language: language.clone()
                }
            }
        );
    }

    transcription
}

fn live_transcription_modes() -> TranscriptionModes {
    TranscriptionModes {
        live_transcription: true,
        meeting_protocol: None,
    }
}

/// Returns the transcription state that was sent with the [`BreakoutEvent::SwitchedRoom`] event.
fn switched_room_transcription_state(event: BreakoutEvent) -> TranscriptionState {
    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Expected SwitchedRoom, got {event:?}");
    };

    own_data
        .get::<TranscriptionState>()
        .expect("Transcription state must be valid")
        .expect("Transcription state must not be none")
}

fn meeting_protocol_modes(pseudonymous: bool, paused: bool) -> TranscriptionModes {
    TranscriptionModes {
        live_transcription: false,
        meeting_protocol: Some(MeetingProtocolMode {
            config: MeetingProtocolConfig { pseudonymous },
            paused,
        }),
    }
}

#[test_log::test(tokio::test)]
async fn start_and_stop_transcription() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;

    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice],
    )
    .await;

    // Stop the transcription
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Stop, None)
        .await
        .unwrap();

    // Participants are immediately notified that the live transcription mode was disabled
    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes::default()
        }
    );
    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );
    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes::default()
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

    // Participants are notified once the service reported that it stopped
    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Inactive
            }
        );
    }

    transcription.disconnect().await.unwrap();
    flush_disconnected_events(&mut [&mut alice]).await;
}

#[test_log::test(tokio::test)]
async fn transcription_segment() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;

    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    let transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice],
    )
    .await;

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

    let request_id = start_transcription(
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
                event: TranscriptionServiceEvent::Started {
                    request_id,
                    language: Some("de".into()),
                },
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running {
                language: Some("de".into())
            }
        }
    );
    expect_transcription_event!(
        transcription,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Running {
                language: Some("de".into())
            }
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
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes::default()
        }
    );
    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );
    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes::default()
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

    // Participants are notified once the service reported that it stopped
    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Inactive
            }
        );
    }

    transcription.disconnect().await.unwrap();

    flush_disconnected_events(&mut [&mut alice, &mut bob]).await;
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn transcription_segment_insufficient_permission() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;

    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    let _transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice],
    )
    .await;

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
        TranscriptionEvent::ModesUpdated {
            modes: live_transcription_modes()
        }
    );
    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Requested { language: None }
        }
    );

    expect_transcription_event!(
        alice,
        TranscriptionEvent::Error(TranscriptionError::ServiceRequestFailed)
    );

    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes::default()
        }
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

    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    let transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice],
    )
    .await;
    let transcription_participant_id = transcription.id();

    transcription.disconnect().await.unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::Error(TranscriptionError::ServiceDisconnected)
    );

    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes::default()
        }
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

#[test_log::test(tokio::test)]
async fn unexpected_service_stop() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;
    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice],
    )
    .await;

    // The service reports that it stopped without being told to stop
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Stopped,
            },
            None,
        )
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::Error(TranscriptionError::ServiceDisconnected)
        );
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes::default()
            }
        );
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Inactive
            }
        );
    }

    // The service disconnecting afterwards does not trigger another error
    transcription.disconnect().await.unwrap();
    flush_disconnected_events(&mut [&mut alice]).await;

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn meeting_protocol_join_success() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let state = alice
        .join_success()
        .get_module::<TranscriptionState>()
        .expect("Transcription state must be valid")
        .expect("Transcription state must not be none");
    assert_eq!(
        state,
        TranscriptionState {
            status: TranscriptionStatus::Inactive,
            live_transcription: false,
            meeting_protocol: None,
        }
    );

    let language = Some("en".to_owned());
    let room_kind = RoomKind::Main;
    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        true,
        &mut alice,
        &mut [],
    )
    .await;

    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let state = bob
        .join_success()
        .get_module::<TranscriptionState>()
        .expect("Transcription state must be valid")
        .expect("Transcription state must not be none");
    assert_eq!(
        state,
        TranscriptionState {
            status: TranscriptionStatus::Requested {
                language: language.clone()
            },
            live_transcription: false,
            meeting_protocol: Some(MeetingProtocolState {
                config: MeetingProtocolConfig { pseudonymous: true },
                consents: false,
                paused: false,
            })
        }
    );

    let _transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        language.clone(),
        &mut [&mut alice],
    )
    .await;

    let charlie = room.join_charlie(0).await;
    let state = charlie
        .join_success()
        .get_module::<TranscriptionState>()
        .expect("Transcription state must be valid")
        .expect("Transcription state must not be none");
    assert_eq!(
        state,
        TranscriptionState {
            status: TranscriptionStatus::Running {
                language: language.clone()
            },
            live_transcription: false,
            meeting_protocol: Some(MeetingProtocolState {
                config: MeetingProtocolConfig { pseudonymous: true },
                consents: false,
                paused: false,
            })
        }
    );
}

#[test_log::test(tokio::test)]
async fn pseudonymous_meeting_protocol() {
    let text = record_meeting_protocol(true).await;

    insta::with_settings!({
       filters => vec![
          (r"[0-9]{2}:[0-9]{2}:[0-9]{2}", "[time_seconds]"),
          (r"[0-9]{2}:[0-9]{2}", "[time_minutes]"),
          (r"[0-9]{2}\.[0-9]{2}\.[0-9]{4}", "[date]"),
       ]
    }, { assert_snapshot!(text)});
}

#[test_log::test(tokio::test)]
async fn named_meeting_protocol() {
    let text = record_meeting_protocol(false).await;

    insta::with_settings!({
       filters => vec![
          (r"[0-9]{2}:[0-9]{2}:[0-9]{2}", "[time_seconds]"),
          (r"[0-9]{2}:[0-9]{2}", "[time_minutes]"),
          (r"[0-9]{2}\.[0-9]{2}\.[0-9]{4}", "[date]"),
       ]
    }, { assert_snapshot!(text)});
}

/// Participants joining after the meeting protocol started are counted as present.
#[test_log::test(tokio::test)]
async fn late_joining_participant_is_counted_as_present() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);
    let mut alice = room.join_alice_moderator(0).await;

    let language = Some("en".to_owned());
    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        RoomKind::Main,
        language.clone(),
        true,
        &mut alice,
        &mut [],
    )
    .await;
    let mut transcription = join_transcription_service(
        &mut room,
        RoomKind::Main,
        0,
        request_id,
        language,
        &mut [&mut alice],
    )
    .await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice, &mut transcription]).await;
    set_consent(true, &mut alice, &mut [&mut bob, &mut transcription]).await;

    let pdf = stop_meeting_protocol(&room, &mut transcription, &mut alice, &mut [&mut bob]).await;
    let text = pdf_extract::extract_text_from_mem(&pdf).expect("PDF must contain text");

    assert!(text.contains("Participants 2"));
}

/// Records a meeting protocol that contains every kind of protocol entry and returns the
/// generated PDF.
async fn record_meeting_protocol(pseudonymous: bool) -> String {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let language = Some("en".to_owned());
    let room_kind = RoomKind::Main;

    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        pseudonymous,
        &mut alice,
        &mut [&mut bob],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        language.clone(),
        &mut [&mut alice, &mut bob],
    )
    .await;

    // Only participants that consented are included in the meeting protocol
    set_consent(true, &mut alice, &mut [&mut bob, &mut transcription]).await;
    set_consent(true, &mut bob, &mut [&mut alice, &mut transcription]).await;

    // Consecutive segments of the same speaker are merged. Pseudonyms are assigned in the order the
    // participants first spoke. Live transcription is disabled, so no segments are broadcast.
    for (speaker, text) in [
        (alice.id(), "Hello everyone."),
        (alice.id(), "Let's get started."),
        (bob.id(), "Hi Alice."),
        (alice.id(), "First topic."),
    ] {
        transcription
            .send_command::<TranscriptionModule>(
                TranscriptionCommand::TranscriptionServiceEvent {
                    event: TranscriptionServiceEvent::Segment(TranscriptionSegment {
                        participant_id: speaker,
                        track_id: "track1".into(),
                        starts_at: Timestamp::unix_epoch(),
                        ends_at: Timestamp::unix_epoch(),
                        text: text.into(),
                    }),
                },
                None,
            )
            .await
            .unwrap();
    }

    // Pause the meeting protocol, the service is no longer needed and gets stopped
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::PauseMeetingProtocol, None)
        .await
        .unwrap();

    // All participants get notified that the meeting protocol is paused
    for participant in [&mut alice, &mut bob, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: meeting_protocol_modes(pseudonymous, true)
            }
        );
    }

    // The service is told to stop
    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    // Segments that arrive while the protocol is paused must not be written to the protocol
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Segment(TranscriptionSegment {
                    participant_id: alice.id(),
                    track_id: "track1".into(),
                    starts_at: Timestamp::unix_epoch(),
                    ends_at: Timestamp::unix_epoch(),
                    text: "This is off the record.".into(),
                }),
            },
            None,
        )
        .await
        .unwrap();

    // The transcription service stops
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Stopped,
            },
            None,
        )
        .await
        .unwrap();

    // Participants are notified once the service reported that it stopped
    for participant in [&mut alice, &mut bob, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Inactive
            }
        );
    }

    transcription.disconnect().await.unwrap();
    flush_disconnected_events(&mut [&mut alice, &mut bob]).await;

    // Resume the meeting protocol, a new service gets requested
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::ResumeMeetingProtocol, None)
        .await
        .unwrap();

    // The service is no longer connected, so the participants are notified that a new one has been
    // requested
    for participant in [&mut alice, &mut bob] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: meeting_protocol_modes(pseudonymous, false)
            }
        );
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Requested {
                    language: language.clone()
                }
            }
        );
    }

    let transcription_target = transcription_task
        .transcription_request_rx
        .recv()
        .await
        .unwrap();
    assert_eq!(transcription_target.room_id, room.id());
    assert_eq!(transcription_target.breakout_room, None);
    assert_eq!(transcription_target.language, language);

    // Participants are notified that the service is running again
    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        transcription_target.request_id,
        language,
        &mut [&mut alice, &mut bob],
    )
    .await;

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Segment(TranscriptionSegment {
                    participant_id: bob.id(),
                    track_id: "track1".into(),
                    starts_at: Timestamp::unix_epoch(),
                    ends_at: Timestamp::unix_epoch(),
                    text: "Welcome back.".into(),
                }),
            },
            None,
        )
        .await
        .unwrap();

    let pdf = stop_meeting_protocol(&room, &mut transcription, &mut alice, &mut [&mut bob]).await;

    pdf_extract::extract_text_from_mem(&pdf).expect("PDF must contain text")
}

/// Pausing and resuming the meeting protocol while live transcription is running must not affect
/// the transcription service.
#[test_log::test(tokio::test)]
async fn pause_and_resume_meeting_protocol_with_live_transcription() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    let room_kind = RoomKind::Main;

    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice],
    )
    .await;

    // Start the meeting protocol, the running service is used for it
    alice
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::StartMeetingProtocol {
                language: None,
                timezone: Some(TimeZone::utc()),
                pseudonymous: false,
            },
            None,
        )
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes {
                    live_transcription: true,
                    ..meeting_protocol_modes(false, false)
                }
            }
        );
    }

    // Pause the meeting protocol, the service is still needed for the live transcription
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::PauseMeetingProtocol, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes {
                    live_transcription: true,
                    ..meeting_protocol_modes(false, true)
                }
            }
        );
    }

    // Segments are still broadcast while the meeting protocol is paused. The service must not be
    // told to stop and the service status must not change, so the next event is the segment.
    let segment = TranscriptionSegment {
        participant_id: alice.id(),
        track_id: "track1".into(),
        starts_at: Timestamp::unix_epoch(),
        ends_at: Timestamp::unix_epoch(),
        text: "This is off the record.".into(),
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

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(participant, TranscriptionEvent::Segment(segment.clone()));
    }

    // Resume the meeting protocol, the service is still connected so it resumes immediately
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::ResumeMeetingProtocol, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes {
                    live_transcription: true,
                    ..meeting_protocol_modes(false, false)
                }
            }
        );
    }

    let segment = TranscriptionSegment {
        text: "Welcome back.".into(),
        ..segment
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

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(participant, TranscriptionEvent::Segment(segment.clone()));
    }

    // Stop the meeting protocol, the service keeps running for the live transcription
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::StopMeetingProtocol, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: live_transcription_modes()
            }
        );
    }

    // The next event is the PDF, the service is not told to stop
    for participant in [&mut alice, &mut transcription] {
        assert_matches!(
            participant
                .receive::<TranscriptionEvent>()
                .await
                .unwrap()
                .payload,
            TranscriptionEvent::PdfCreated { .. }
        );
    }
}

/// When the transcription service disconnects unexpectedly, the meeting protocol is stopped and the
/// PDF is generated, so that no data is lost.
#[test_log::test(tokio::test)]
async fn unexpected_disconnect_meeting_protocol() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    let language = Some("en".to_owned());
    let room_kind = RoomKind::Main;
    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    let transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        language,
        &mut [&mut alice],
    )
    .await;
    let transcription_id = transcription.id();

    // The service disconnects without being told to stop
    transcription.disconnect().await.unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::Error(TranscriptionError::ServiceDisconnected)
    );
    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes::default()
        }
    );
    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive
        }
    );

    assert_matches!(
        alice.receive::<CoreEvent>().await.unwrap().payload,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            ..
        } if participant_id == transcription_id
    );

    // The meeting protocol recorded so far is not lost
    let event = alice.receive::<TranscriptionEvent>().await.unwrap();
    let TranscriptionEvent::PdfCreated { asset_id, .. } = event.payload else {
        panic!("Expected PdfCreated, got {:?}", event.payload);
    };

    room.stored_asset(asset_id)
        .await
        .expect("Meeting protocol PDF must be stored");
}

/// Only participants that consented are included in the meeting protocol.
#[test_log::test(tokio::test)]
async fn consent_meeting_protocol() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let language = Some("en".to_owned());
    let room_kind = RoomKind::Main;
    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        false,
        &mut alice,
        &mut [&mut bob],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        language,
        &mut [&mut alice, &mut bob],
    )
    .await;

    // Bob consents, Alice does not
    set_consent(true, &mut bob, &mut [&mut alice, &mut transcription]).await;

    for (speaker, text) in [
        (bob.id(), "Consented speech"),
        (alice.id(), "Unconsented speech"),
    ] {
        transcription
            .send_command::<TranscriptionModule>(
                TranscriptionCommand::TranscriptionServiceEvent {
                    event: TranscriptionServiceEvent::Segment(TranscriptionSegment {
                        participant_id: speaker,
                        track_id: "track1".into(),
                        starts_at: Timestamp::unix_epoch(),
                        ends_at: Timestamp::unix_epoch(),
                        text: text.into(),
                    }),
                },
                None,
            )
            .await
            .unwrap();
    }

    // Bob revokes his consent, his following speech is no longer included
    set_consent(false, &mut bob, &mut [&mut alice, &mut transcription]).await;

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Segment(TranscriptionSegment {
                    participant_id: bob.id(),
                    track_id: "track1".into(),
                    starts_at: Timestamp::unix_epoch(),
                    ends_at: Timestamp::unix_epoch(),
                    text: "Revoked speech".into(),
                }),
            },
            None,
        )
        .await
        .unwrap();

    let pdf = stop_meeting_protocol(&room, &mut transcription, &mut alice, &mut [&mut bob]).await;
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();

    assert!(content.contains("Consented speech"), "{content}");
    assert!(!content.contains("Unconsented speech"), "{content}");
    assert!(!content.contains("Revoked speech"), "{content}");
}

/// Setting the consent fails when no meeting protocol is running.
#[test_log::test(tokio::test)]
async fn consent_without_meeting_protocol() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    // No transcription at all
    alice
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::SetMeetingProtocolConsent { consent: true },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::Error(TranscriptionError::NotActive)
    );

    // Only live transcription
    start_transcription(
        &mut transcription_task,
        &mut room,
        RoomKind::Main,
        &mut alice,
        &mut [],
    )
    .await;

    alice
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::SetMeetingProtocolConsent { consent: true },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::Error(TranscriptionError::NotActive)
    );
}

/// A participant that reconnects keeps their meeting protocol consent.
#[test_log::test(tokio::test)]
async fn reconnect_keeps_consent() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    let language = Some("en".to_owned());

    start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        RoomKind::Main,
        language.clone(),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    set_consent(true, &mut alice, &mut []).await;

    let alice_id = alice.id();
    alice.disconnect().await.unwrap();

    let alice = room.join_alice_moderator(0).await;
    assert_eq!(alice.id(), alice_id);

    let state = alice
        .join_success()
        .get_module::<TranscriptionState>()
        .expect("Transcription state must be valid")
        .expect("Transcription state must not be none");
    assert_eq!(
        state,
        TranscriptionState {
            status: TranscriptionStatus::Requested { language },
            live_transcription: false,
            meeting_protocol: Some(MeetingProtocolState {
                config: MeetingProtocolConfig {
                    pseudonymous: false
                },
                consents: true,
                paused: false,
            })
        }
    );
}

/// The consent only applies to the meeting protocol it was given for.
#[test_log::test(tokio::test)]
async fn stopping_meeting_protocol_clears_consent() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    let language = Some("en".to_owned());
    let room_kind = RoomKind::Main;
    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        language.clone(),
        &mut [&mut alice],
    )
    .await;

    set_consent(true, &mut alice, &mut [&mut transcription]).await;
    stop_meeting_protocol(&room, &mut transcription, &mut alice, &mut []).await;

    // Start a new meeting protocol, the previous consent does not carry over
    start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        false,
        &mut alice,
        &mut [&mut transcription],
    )
    .await;

    alice.disconnect().await.unwrap();
    let alice = room.join_alice_moderator(0).await;

    let consents = alice
        .join_success()
        .get_module::<TranscriptionState>()
        .expect("Transcription state must be valid")
        .expect("Transcription state must not be none")
        .meeting_protocol
        .expect("Meeting protocol must be ongoing")
        .consents;
    assert!(!consents);
}

/// Starting live transcription while the meeting protocol is running reuses the running service.
#[test_log::test(tokio::test)]
async fn start_live_transcription_while_meeting_protocol_running() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    let language = Some("en".to_owned());
    let room_kind = RoomKind::Main;
    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        language,
        &mut [&mut alice],
    )
    .await;

    alice
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::Start {
                language: Some("de".into()),
            },
            None,
        )
        .await
        .unwrap();

    // Only the modes change, no new service is requested
    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes {
                    live_transcription: true,
                    ..meeting_protocol_modes(false, false)
                }
            }
        );
    }

    // Segments are broadcast now. The service status did not change, so the next event is the
    // segment.
    let segment = TranscriptionSegment {
        participant_id: alice.id(),
        track_id: "track1".into(),
        starts_at: Timestamp::unix_epoch(),
        ends_at: Timestamp::unix_epoch(),
        text: "Hello everyone.".into(),
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

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(participant, TranscriptionEvent::Segment(segment.clone()));
    }
}

/// Stopping live transcription while the meeting protocol is running keeps the service running
/// for the meeting protocol.
#[test_log::test(tokio::test)]
async fn stop_live_transcription_keeps_meeting_protocol() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    let room_kind = RoomKind::Main;
    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice],
    )
    .await;

    alice
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::StartMeetingProtocol {
                language: None,
                timezone: Some(TimeZone::utc()),
                pseudonymous: false,
            },
            None,
        )
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes {
                    live_transcription: true,
                    ..meeting_protocol_modes(false, false)
                }
            }
        );
    }

    set_consent(true, &mut alice, &mut [&mut transcription]).await;

    let alice_id = alice.id();

    // Stop the live transcription, the service is still needed for the meeting protocol
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Stop, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: meeting_protocol_modes(false, false)
            }
        );
    }

    // Segments are only recorded, not broadcast
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Segment(TranscriptionSegment {
                    participant_id: alice_id,
                    track_id: "track1".into(),
                    starts_at: Timestamp::unix_epoch(),
                    ends_at: Timestamp::unix_epoch(),
                    text: "Still recorded".into(),
                }),
            },
            None,
        )
        .await
        .unwrap();

    // The service was not told to stop and no segment was broadcast, so the next events are the
    // modes update and the stop command
    let pdf = stop_meeting_protocol(&room, &mut transcription, &mut alice, &mut []).await;
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();

    assert!(content.contains("Still recorded"), "{content}");
}

/// Starting an already active mode only notifies the sender about the current modes.
#[test_log::test(tokio::test)]
async fn start_when_already_active() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let room_kind = RoomKind::Main;

    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [&mut bob],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice, &mut bob],
    )
    .await;

    // Alice tries to start live transcription again, but it is already active. She only receives
    // the current modes, Bob does not receive any event.
    alice
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::Start {
                language: Some("de".into()),
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: live_transcription_modes()
        }
    );
    assert!(bob.received_nothing());
    assert!(transcription.received_nothing());

    let start_meeting_protocol = TranscriptionCommand::StartMeetingProtocol {
        language: None,
        timezone: Some(TimeZone::utc()),
        pseudonymous: false,
    };
    alice
        .send_command::<TranscriptionModule>(start_meeting_protocol.clone(), None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut bob, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes {
                    live_transcription: true,
                    ..meeting_protocol_modes(false, false)
                }
            }
        );
    }

    // The meeting protocol is already active
    alice
        .send_command::<TranscriptionModule>(start_meeting_protocol, None)
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes {
                live_transcription: true,
                ..meeting_protocol_modes(false, false)
            }
        }
    );
    assert!(bob.received_nothing());
    assert!(transcription.received_nothing());
}

/// Commands that require an active transcription mode fail when the mode is not active.
#[test_log::test(tokio::test)]
async fn commands_when_not_active() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    // No transcription at all
    assert_command_error(
        &mut alice,
        TranscriptionCommand::Stop,
        TranscriptionError::NotActive,
    )
    .await;
    assert_command_error(
        &mut alice,
        TranscriptionCommand::ChangeLanguage {
            language: Some("en".into()),
        },
        TranscriptionError::NotActive,
    )
    .await;
    assert_command_error(
        &mut alice,
        TranscriptionCommand::PauseMeetingProtocol,
        TranscriptionError::NotActive,
    )
    .await;
    assert_command_error(
        &mut alice,
        TranscriptionCommand::ResumeMeetingProtocol,
        TranscriptionError::NotActive,
    )
    .await;
    assert_command_error(
        &mut alice,
        TranscriptionCommand::StopMeetingProtocol,
        TranscriptionError::NotActive,
    )
    .await;

    // Only live transcription, the meeting protocol commands fail
    start_transcription(
        &mut transcription_task,
        &mut room,
        RoomKind::Main,
        &mut alice,
        &mut [],
    )
    .await;

    assert_command_error(
        &mut alice,
        TranscriptionCommand::PauseMeetingProtocol,
        TranscriptionError::NotActive,
    )
    .await;
    assert_command_error(
        &mut alice,
        TranscriptionCommand::ResumeMeetingProtocol,
        TranscriptionError::NotActive,
    )
    .await;
    assert_command_error(
        &mut alice,
        TranscriptionCommand::StopMeetingProtocol,
        TranscriptionError::NotActive,
    )
    .await;
}

/// Only moderators can control the meeting protocol.
#[test_log::test(tokio::test)]
async fn meeting_protocol_commands_insufficient_permission() {
    let transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut bob = room.join_bob(0).await;

    assert_command_error(
        &mut bob,
        TranscriptionCommand::StartMeetingProtocol {
            language: None,
            timezone: Some(TimeZone::utc()),
            pseudonymous: false,
        },
        TranscriptionError::InsufficientPermissions,
    )
    .await;
    assert_command_error(
        &mut bob,
        TranscriptionCommand::PauseMeetingProtocol,
        TranscriptionError::InsufficientPermissions,
    )
    .await;
    assert_command_error(
        &mut bob,
        TranscriptionCommand::ResumeMeetingProtocol,
        TranscriptionError::InsufficientPermissions,
    )
    .await;
    assert_command_error(
        &mut bob,
        TranscriptionCommand::StopMeetingProtocol,
        TranscriptionError::InsufficientPermissions,
    )
    .await;
}

/// Sends `command` and expects the sender to receive `error`.
async fn assert_command_error(
    participant: &mut MockParticipantJoined,
    command: TranscriptionCommand,
    error: TranscriptionError,
) {
    participant
        .send_command::<TranscriptionModule>(command, None)
        .await
        .unwrap();

    expect_transcription_event!(participant, TranscriptionEvent::Error(error));
}

/// Pausing a paused meeting protocol and resuming a running one only notifies the sender.
#[test_log::test(tokio::test)]
async fn pause_twice_and_resume_unpaused() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let language = Some("en".to_owned());
    let room_kind = RoomKind::Main;
    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        false,
        &mut alice,
        &mut [&mut bob],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        language,
        &mut [&mut alice, &mut bob],
    )
    .await;

    // Resuming a running meeting protocol only notifies the sender
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::ResumeMeetingProtocol, None)
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: meeting_protocol_modes(false, false)
        }
    );
    assert!(bob.received_nothing());
    assert!(transcription.received_nothing());

    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::PauseMeetingProtocol, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut bob, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: meeting_protocol_modes(false, true)
            }
        );
    }

    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    // Pausing a paused meeting protocol only notifies the sender
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::PauseMeetingProtocol, None)
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: meeting_protocol_modes(false, true)
        }
    );
    assert!(bob.received_nothing());
    assert!(transcription.received_nothing());
}

/// Changing the language when the transcription service is not connected results in an error
#[test_log::test(tokio::test)]
async fn change_language_not_active() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);
    let mut alice = room.join_alice_moderator(0).await;

    start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        RoomKind::Main,
        Some("en".into()),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    // Alice tries to change the language while the service is not yet connected
    assert_command_error(
        &mut alice,
        TranscriptionCommand::ChangeLanguage { language: None },
        TranscriptionError::NotActive,
    )
    .await;
}

/// Changing the language is forwarded to the service, the participants are notified once the
/// service confirmed the change.
#[test_log::test(tokio::test)]
async fn change_language() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    let room_kind = RoomKind::Main;
    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        Some("de".into()),
        &mut [&mut alice],
    )
    .await;

    alice
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::ChangeLanguage {
                language: Some("en".into()),
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::ChangeLanguage {
                language: Some("en".into())
            }
        }
    );

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Updated {
                    language: Some("en".into()),
                },
            },
            None,
        )
        .await
        .unwrap();

    // The first event the participants receive is the confirmed language
    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Running {
                    language: Some("en".into())
                }
            }
        );
    }
}

/// A meeting protocol in a breakout room does not affect the main room.
#[test_log::test(tokio::test)]
async fn breakout_room_meeting_protocol() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    alice
        .start_breakout_rooms(
            &mut [],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "breakout".into(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;

    let breakout_room = RoomKind::Breakout(BreakoutId::from(0));
    alice.switch_breakout_room(&mut [], breakout_room).await;

    let language = Some("en".to_owned());
    start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        breakout_room,
        language.clone(),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    // The main room has no transcription
    let event = alice.switch_breakout_room(&mut [], RoomKind::Main).await;
    assert_eq!(
        switched_room_transcription_state(event),
        TranscriptionState {
            status: TranscriptionStatus::Inactive,
            live_transcription: false,
            meeting_protocol: None,
        }
    );

    // The meeting protocol in the breakout room is still running
    let event = alice.switch_breakout_room(&mut [], breakout_room).await;
    assert_eq!(
        switched_room_transcription_state(event),
        TranscriptionState {
            status: TranscriptionStatus::Requested { language },
            live_transcription: false,
            meeting_protocol: Some(MeetingProtocolState {
                config: MeetingProtocolConfig {
                    pseudonymous: false
                },
                consents: false,
                paused: false,
            }),
        }
    );
}

/// Closing the room while a meeting protocol is ongoing generates the meeting protocol PDF.
#[test_log::test(tokio::test(start_paused = true))]
async fn closing_room_generates_meeting_protocol() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        RoomKind::Main,
        Some("en".to_owned()),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    // Alice leaves
    alice.disconnect().await.unwrap();

    // Wait for the room to close
    tokio::time::advance(ROOM_IDLE_TIMEOUT).await;
    (&mut room.join_handle).await.unwrap();

    // The pdf is automatically generated when the last participant leaves
    let assets = room.stored_assets().await;
    assert_eq!(assets.len(), 1);
}

/// Closing the breakout rooms while a meeting protocol is ongoing in a breakout room generates the
/// meeting protocol PDF.
#[test_log::test(tokio::test)]
async fn closing_breakout_room_generates_meeting_protocol() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    alice
        .start_breakout_rooms(
            &mut [],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "breakout".into(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;

    let breakout_room = RoomKind::Breakout(BreakoutId::from(0));
    alice.switch_breakout_room(&mut [], breakout_room).await;

    start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        breakout_room,
        Some("en".to_owned()),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    // Alice switches back to the main room
    alice.switch_breakout_room(&mut [], RoomKind::Main).await;

    // Alice closes the breakout room
    let event = alice.stop_breakout_rooms(&mut []).await;
    assert_matches!(event, BreakoutEvent::Closing { issued_by } if issued_by == Some(alice.id()));

    let event = alice.receive::<BreakoutEvent>().await.unwrap().payload;
    assert_matches!(event, BreakoutEvent::Closed);

    let event = alice.receive::<TranscriptionEvent>().await.unwrap();
    let TranscriptionEvent::PdfCreated { asset_id, .. } = event.payload else {
        panic!("Expected PdfCreated, got {:?}", event.payload);
    };

    room.stored_asset(asset_id)
        .await
        .expect("Meeting protocol PDF must be stored");
}

/// Stopping the transcription before the requested service connected notifies the participants
/// immediately and not a second time when the late service stops.
#[test_log::test(tokio::test)]
async fn stop_before_service_connected() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;
    let request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    // Stop the transcription while the service is still requested
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Stop, None)
        .await
        .unwrap();

    expect_transcription_event!(
        alice,
        TranscriptionEvent::ModesUpdated {
            modes: TranscriptionModes::default()
        }
    );

    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive
        }
    );

    // The late service connects and is told to stop
    let mut transcription = room.join_transcription(room_kind, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started {
                    request_id,
                    language: Some("de".into()),
                },
            },
            None,
        )
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

    // Alice is not notified a second time that the transcription service stopped
    assert!(alice.received_nothing());
}

/// Stopping the transcription after the requested service connected, but before it reported that it
/// started, must still tell the participants that the transcription service is inactive.
///
/// Otherwise the participants are stuck with the `requested` status: the service is told to stop,
/// but its `Stopped` event arrives when the room has no transcription state or service left to
/// update, so nothing is broadcast.
#[test_log::test(tokio::test)]
async fn stop_after_service_connected_before_started() {
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

    // The requested service connects, but has not sent `Started` yet
    let mut transcription = room.join_transcription(room_kind, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Stop the transcription while the service is still requested
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Stop, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes::default()
            }
        );
    }

    // The service is told to stop
    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    // The service reports that it stopped, as it was told to
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Stopped,
            },
            None,
        )
        .await
        .unwrap();

    // The participants must learn that the service is no longer requested
    expect_transcription_event!(
        alice,
        TranscriptionEvent::StateUpdated {
            status: TranscriptionStatus::Inactive
        }
    );

    // ...but only once
    assert!(alice.received_nothing());
}

/// Resuming the meeting protocol before the old service confirmed that it stopped must not treat
/// the late `Stopped` of the old service as unexpected.
#[test_log::test(tokio::test)]
async fn resume_before_service_stopped() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;

    let language = Some("en".to_owned());
    let room_kind = RoomKind::Main;
    let request_id = start_meeting_protocol(
        &mut transcription_task,
        &mut room,
        room_kind,
        language.clone(),
        false,
        &mut alice,
        &mut [],
    )
    .await;

    let mut transcription = join_transcription_service(
        &mut room,
        room_kind,
        0,
        request_id,
        language.clone(),
        &mut [&mut alice],
    )
    .await;

    // Pause the meeting protocol, the service is told to stop
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::PauseMeetingProtocol, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: meeting_protocol_modes(false, true)
            }
        );
    }

    expect_transcription_event!(
        transcription,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    // Resume the meeting protocol before the service reported that it stopped
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::ResumeMeetingProtocol, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut transcription] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: meeting_protocol_modes(false, false)
            }
        );

        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Requested {
                    language: language.clone()
                }
            }
        )
    }

    // The old service stops as it was told to
    transcription
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Stopped,
            },
            None,
        )
        .await
        .unwrap();

    assert!(alice.received_nothing());
}

/// A service that was told to stop before it reported that it started must not take over the
/// transcription that was started again in the meantime.
///
/// The service can send `Started` before it handles the `Stop`, so its `Started` arrives after the
/// next service was requested.
#[test_log::test(tokio::test)]
async fn late_start_of_cancelled_service() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let room_kind = RoomKind::Main;
    let canceled_request_id = start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [],
    )
    .await;

    // The requested service connects, but has not sent `Started` yet
    let mut cancelled_service = room.join_transcription(room_kind, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Stop the transcription while the service is still requested, the service is told to stop
    alice
        .send_command::<TranscriptionModule>(TranscriptionCommand::Stop, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut cancelled_service] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::ModesUpdated {
                modes: TranscriptionModes::default()
            }
        );
    }

    expect_transcription_event!(
        cancelled_service,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    for participant in [&mut alice, &mut cancelled_service] {
        expect_transcription_event!(
            participant,
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Inactive
            }
        );
    }

    // Start the transcription again, a new service is requested
    start_transcription(
        &mut transcription_task,
        &mut room,
        room_kind,
        &mut alice,
        &mut [&mut cancelled_service],
    )
    .await;

    // The cancelled service reports that it started, because it did so before it handled the
    // `Stop`
    cancelled_service
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started {
                    request_id: canceled_request_id,
                    language: Some("de".into()),
                },
            },
            None,
        )
        .await
        .unwrap();

    // The cancelled service is told to stop
    expect_transcription_event!(
        cancelled_service,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    assert!(alice.received_nothing());
}

/// A service that reports `Started` for another request is told to stop and does not take over the
/// requested transcription.
#[test_log::test(tokio::test)]
async fn started_with_unknown_request_id() {
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

    let mut unknown_service = room.join_transcription(room_kind, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    unknown_service
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Started {
                    request_id: RequestId::nil(),
                    language: Some("de".into()),
                },
            },
            None,
        )
        .await
        .unwrap();

    expect_transcription_event!(
        unknown_service,
        TranscriptionEvent::ServiceCommand {
            command: TranscriptionServiceCommand::Stop
        }
    );

    assert!(alice.received_nothing());
}

/// The disconnect of a replaced service must not end the transcription of the new service.
#[test_log::test(tokio::test)]
async fn old_service_disconnects_after_restart() {
    let mut transcription_task = MockTranscriptionTask::spawn().await;
    let mut room = create_room(&transcription_task);

    let mut alice = room.join_alice_moderator(0).await;
    let (old_service, mut new_service) = restart_before_service_stopped(
        &mut transcription_task,
        &mut room,
        RoomKind::Main,
        &mut alice,
    )
    .await;

    // The old service stops and leaves, as it was told to
    old_service
        .send_command::<TranscriptionModule>(
            TranscriptionCommand::TranscriptionServiceEvent {
                event: TranscriptionServiceEvent::Stopped,
            },
            None,
        )
        .await
        .unwrap();

    old_service.disconnect().await.unwrap();
    flush_disconnected_events(&mut [&mut alice, &mut new_service]).await;

    // The transcription keeps running
    assert!(alice.received_nothing());
    assert!(new_service.received_nothing());
}
