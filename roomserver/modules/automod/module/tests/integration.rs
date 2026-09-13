// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::assert_matches;

use opentalk_roomserver_module_automod::AutomodModule;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_test_util_livekit as test_util;
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        event::BreakoutEvent,
    },
    core::CoreEvent,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_automod::{
    command::{AutomodCommand, Select},
    config::{FrontendConfig, Parameter, SelectionStrategy},
    event::{AutomodError, AutomodEvent, StoppedReason},
    state::AutomodState,
};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;

pub async fn build_room() -> (test_util::ContainerGuard, TestRoom, String) {
    let (container, settings) = test_util::create_livekit_container().await;

    let room = TestRoom::builder()
        .room_id(RoomId::generate())
        .register_module::<LiveKitModule>()
        .register_module::<AutomodModule>()
        .add_init_module_data(&settings)
        .unwrap()
        .spawn();

    (container, room, settings.public_url)
}

async fn start_automod(
    participant: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
    parameter: Parameter,
    allow_list: Option<Vec<ParticipantId>>,
    playlist: Option<Vec<ParticipantId>>,
) {
    participant
        .send_command::<AutomodModule>(
            AutomodCommand::Start {
                parameter: parameter.clone(),
                allow_list: allow_list.clone(),
                playlist: playlist.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let remaining = match parameter.selection_strategy {
        SelectionStrategy::Playlist => playlist.unwrap(),
        _ => allow_list.unwrap(),
    };

    let event = participant
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::Started(
            FrontendConfig {
                parameter: parameter.clone(),
                history: Vec::new(),
                remaining: remaining.clone(),
                issued_by: participant.id(),
            }
            .into_public()
        )
    );

    for p in others {
        let event = p.receive_event::<AutomodModule>().await.unwrap().payload;
        assert_eq!(
            event,
            AutomodEvent::Started(
                FrontendConfig {
                    parameter: parameter.clone(),
                    history: Vec::new(),
                    remaining: remaining.clone(),
                    issued_by: participant.id(),
                }
                .into_public()
            )
        );
    }
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions_start() {
    let mut room = TestRoom::builder()
        .register_module::<AutomodModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob is not a moderator, so he cannot start automod
    bob.send_command::<AutomodModule>(
        AutomodCommand::Start {
            parameter: Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_remaining: true,
                time_limit: None,
                allow_double_selection: true,
                auto_append_on_join: true,
            },
            allow_list: None,
            playlist: None,
        },
        None,
    )
    .await
    .unwrap();

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::Error(AutomodError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_insufficient_permissions_edit() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let allow_list = vec![bob.id(), alice.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob],
        parameter,
        Some(allow_list),
        None,
    )
    .await;

    // Bob is not a moderator, so he cannot edit the automod session
    bob.send_command::<AutomodModule>(
        AutomodCommand::Edit {
            allow_list: Some(vec![bob.id()]),
            playlist: None,
        },
        None,
    )
    .await
    .unwrap();

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::Error(AutomodError::InsufficientPermissions)
    );

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_insufficient_permissions_stop() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let allow_list = vec![bob.id(), alice.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob],
        parameter,
        Some(allow_list),
        None,
    )
    .await;

    // Bob is not a moderator, so he cannot stop the automod session
    bob.send_command::<AutomodModule>(AutomodCommand::Stop, None)
        .await
        .unwrap();

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::Error(AutomodError::InsufficientPermissions)
    );

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_insufficient_permissions_select() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let allow_list = vec![bob.id(), alice.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob],
        parameter,
        Some(allow_list),
        None,
    )
    .await;

    // Bob is not a moderator, so he cannot select the next speaker
    bob.send_command::<AutomodModule>(AutomodCommand::Select(Select::Next), None)
        .await
        .unwrap();

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::Error(AutomodError::InsufficientPermissions)
    );

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_insufficient_permissions_yield() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let allow_list = vec![alice.id(), bob.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob],
        parameter,
        Some(allow_list.clone()),
        None,
    )
    .await;

    // Only the current speaker is allowed to yield
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: alice.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(allow_list.clone())
        }
    );

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(allow_list)
        }
    );

    bob.send_command::<AutomodModule>(
        AutomodCommand::Yield {
            next: Some(bob.id()),
        },
        None,
    )
    .await
    .unwrap();

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::Error(AutomodError::InsufficientPermissions)
    );

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_session_already_running() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let allow_list = vec![alice.id()];
    start_automod(
        &mut alice,
        &mut [],
        parameter.clone(),
        Some(allow_list.clone()),
        None,
    )
    .await;

    // Starting another session is not allowed
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Start {
                parameter: parameter.clone(),
                allow_list: Some(allow_list.clone()),
                playlist: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::Error(AutomodError::SessionAlreadyRunning)
    );
}

#[test_log::test(tokio::test)]
async fn session_not_running() {
    let mut room = TestRoom::builder()
        .register_module::<AutomodModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Editing a session without starting one first is not possible
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Edit {
                allow_list: None,
                playlist: Some(vec![alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::SessionNotRunning));
}

#[test_log::test(tokio::test)]
async fn invalid_selection_start() {
    let mut room = TestRoom::builder()
        .register_module::<AutomodModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    // In `nomination` mode, the `allow_list` must not be `None`
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Start {
                parameter: parameter.clone(),
                allow_list: None,
                playlist: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));
    // In `nomination` mode, the `playlist` is not considered, the `allow_list` must not
    // be `None`.
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Start {
                parameter: parameter.clone(),
                allow_list: None,
                playlist: Some(vec![alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // In `nomination` mode, the `allow_list` must not be empty
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Start {
                parameter: parameter.clone(),
                allow_list: Some(Vec::new()),
                playlist: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Playlist,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    // In `playlist` mode, the `playlist` must not be `None`
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Start {
                parameter: parameter.clone(),
                allow_list: Some(vec![alice.id()]),
                playlist: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // In `playlist` mode, the `playlist` must not be empty
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Start {
                parameter: parameter.clone(),
                allow_list: None,
                playlist: Some(Vec::new()),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));
}

#[test_log::test(tokio::test)]
async fn start_across_breakout_room() {
    let mut room = TestRoom::builder()
        .register_module::<AutomodModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_string(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;

    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Start {
                parameter: Parameter {
                    selection_strategy: SelectionStrategy::None,
                    show_remaining: true,
                    time_limit: None,
                    allow_double_selection: true,
                    auto_append_on_join: false,
                },
                allow_list: Some(vec![bob.id()]),
                playlist: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_invalid_edit() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let allow_list = vec![alice.id()];
    start_automod(&mut alice, &mut [], parameter, Some(allow_list), None).await;

    // In `nomination` mode, the `allow_list` must not be `None`
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Edit {
                allow_list: None,
                playlist: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidEdit));

    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Edit {
                allow_list: None,
                playlist: Some(vec![alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidEdit));

    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Edit {
                allow_list: Some(Vec::new()),
                playlist: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidEdit));
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_invalid_select() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let allow_list = vec![alice.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob],
        parameter.clone(),
        Some(allow_list),
        None,
    )
    .await;

    // Selecting a participant that does not exist
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: ParticipantId::nil(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Selecting a participant that is not in the allow list
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: bob.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Selecting a participant that is in a different breakout room is not allowed
    // First, start a breakout room session
    let bob_id = bob.id();
    let event = alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".into(),
                    assignments: vec![bob_id],
                }],
                duration: None,
            },
        )
        .await;
    assert_matches!(event, BreakoutEvent::Started { .. });

    // Bob switches to breakout room 0
    bob.switch_breakout_room(&mut [&mut alice], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    // Selecting bob fails because he is in the breakout room
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: bob.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_stop() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Playlist,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: false,
    };
    let playlist = vec![alice.id()];
    start_automod(&mut alice, &mut [], parameter, None, Some(playlist)).await;

    alice
        .send_command::<AutomodModule>(AutomodCommand::Stop, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::Stopped(StoppedReason::StoppedByModerator {
            issued_by: alice.id()
        })
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_join_success() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    assert!(
        alice
            .join_success()
            .get_module::<AutomodState>()
            .expect("Automod state must be valid")
            .is_none()
    );

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Playlist,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: false,
    };
    let playlist = vec![alice.id()];
    start_automod(
        &mut alice,
        &mut [],
        parameter.clone(),
        None,
        Some(playlist.clone()),
    )
    .await;

    let bob = room.join_bob(0).await;
    let state = bob
        .join_success()
        .get_module::<AutomodState>()
        .expect("Automod state must be valid")
        .expect("Automod state must not be none");

    assert_eq!(
        state,
        AutomodState {
            config: FrontendConfig {
                parameter: parameter.clone(),
                history: Vec::new(),
                remaining: playlist,
                issued_by: alice.id()
            }
            .into_public(),
            speaker: None
        }
    );

    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Next), None)
        .await
        .unwrap();

    let dave = room.join_dave(0).await;
    let state = dave
        .join_success()
        .get_module::<AutomodState>()
        .expect("Automod state must be valid")
        .expect("Automod state must not be none");
    assert_eq!(
        state,
        AutomodState {
            config: FrontendConfig {
                parameter,
                history: vec![alice.id()],
                remaining: Vec::new(),
                issued_by: alice.id()
            }
            .into_public(),
            speaker: Some(alice.id())
        }
    )
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_join_auto_append() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    // Auto append on join is enabled
    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Playlist,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let playlist = vec![alice.id()];
    // Automod starts
    start_automod(
        &mut alice,
        &mut [],
        parameter.clone(),
        None,
        Some(playlist.clone()),
    )
    .await;

    // The first speaker is selected
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Next), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(Vec::new())
        }
    );

    // Bob joins the room
    let mut bob = room.join_bob(0).await;
    let state = bob
        .join_success()
        .get_module::<AutomodState>()
        .expect("Automod state must be valid")
        .expect("Automod state must not be none");
    assert_eq!(
        state,
        AutomodState {
            config: FrontendConfig {
                parameter: parameter.clone(),
                history: vec![alice.id()],
                remaining: vec![bob.id()],
                issued_by: alice.id(),
            }
            .into_public(),
            speaker: Some(alice.id())
        }
    );

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantConnected { participant_id, connection_id, .. }
            if participant_id == bob.id() && connection_id == bob.connection_id()
    );

    // Bob was appended to the `remaining` list
    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::RemainingUpdated {
            remaining: vec![bob.id()]
        }
    );

    alice
        .send_command::<AutomodModule>(AutomodCommand::Yield { next: None }, None)
        .await
        .unwrap();

    // Bob is selected as speaker
    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(bob.id()),
            history: Some(vec![alice.id(), bob.id()]),
            remaining: Some(Vec::new())
        }
    );

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(bob.id()),
            history: Some(vec![alice.id(), bob.id()]),
            remaining: Some(Vec::new())
        }
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_join_no_auto_append() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    // Auto append on join is disabled
    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Playlist,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: false,
    };
    let playlist = vec![alice.id()];
    start_automod(&mut alice, &mut [], parameter.clone(), None, Some(playlist)).await;

    // The first speaker is selected
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Next), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(Vec::new())
        }
    );

    // Bob joins the room
    let bob = room.join_bob(0).await;
    let state = bob
        .join_success()
        .get_module::<AutomodState>()
        .expect("Automod state must be valid")
        .expect("Automod state must not be none");
    assert_eq!(
        state,
        AutomodState {
            config: FrontendConfig {
                parameter: parameter.clone(),
                history: vec![alice.id()],
                remaining: Vec::new(),
                issued_by: alice.id(),
            }
            .into_public(),
            speaker: Some(alice.id())
        }
    );

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantConnected { participant_id, connection_id, .. }
            if participant_id == bob.id() && connection_id == bob.connection_id()
    );

    // Bob is not appended to the `remaining` list
    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_breakout_room() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let event = alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "breakout room".into(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    assert_matches!(event, BreakoutEvent::Started { .. });

    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    // Alice starts automod in the breakout room
    let parameter = Parameter {
        selection_strategy: SelectionStrategy::None,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: false,
        auto_append_on_join: false,
    };
    let allow_list = vec![alice.id()];
    start_automod(
        &mut alice,
        &mut [],
        parameter.clone(),
        Some(allow_list),
        None,
    )
    .await;

    // Bob does not receive any automod events in the main room
    assert!(bob.received_nothing());

    // Bob switches to the breakout room too
    let event = bob
        .switch_breakout_room(&mut [&mut alice], RoomKind::Breakout(BreakoutId::from(0)))
        .await;
    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Received wrong event: {:?}", event);
    };

    // Bob receives the automod state from the breakout room
    let state = own_data
        .get::<AutomodState>()
        .expect("Automod state must be valid")
        .expect("Automod state must not be none");
    assert_eq!(
        state,
        AutomodState {
            config: FrontendConfig {
                parameter,
                history: Vec::new(),
                remaining: vec![alice.id()],
                issued_by: alice.id()
            }
            .into_public(),
            speaker: None,
        }
    )
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_participant_disconnect() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Playlist,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: false,
    };
    let playlist = vec![alice.id(), bob.id(), charlie.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob, &mut charlie],
        parameter.clone(),
        None,
        Some(playlist.clone()),
    )
    .await;

    // Bob disconnects
    let bob_id = bob.id();
    bob.disconnect().await.unwrap();

    // Bob was removed from the `remaining` list
    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::RemainingUpdated {
            remaining: vec![alice.id(), charlie.id()]
        }
    );

    let event = charlie
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::RemainingUpdated {
            remaining: vec![alice.id(), charlie.id()]
        }
    );

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert!(
        matches!(event, CoreEvent::ParticipantDisconnected { participant_id, .. }
            if participant_id == bob_id)
    );

    let event = charlie.receive::<CoreEvent>().await.unwrap().payload;
    assert!(
        matches!(event, CoreEvent::ParticipantDisconnected { participant_id, .. }
            if participant_id == bob_id)
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_selection_strategy_none() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::None,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: true,
        auto_append_on_join: true,
    };
    let allow_list = vec![alice.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob],
        parameter.clone(),
        Some(allow_list.clone()),
        None,
    )
    .await;

    // Trying to select Bob
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: bob.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    // But Bob is not in the allow list
    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    assert!(bob.received_nothing());

    // Selecting next is not allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Next), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Selecting Alice is allowed
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: alice.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(allow_list.clone())
        }
    );

    // Yielding is not allowed
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Yield {
                next: Some(alice.id()),
            },
            None,
        )
        .await
        .unwrap();
    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(allow_list.clone())
        }
    );

    // Selecting None is allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::None), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: None,
            history: Some(vec![alice.id()]),
            remaining: Some(allow_list.clone())
        }
    );

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: None,
            history: Some(vec![alice.id()]),
            remaining: Some(allow_list.clone())
        }
    );
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_selection_strategy_playlist() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob, &mut charlie]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Playlist,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: false,
        auto_append_on_join: false,
    };
    let playlist = vec![alice.id(), bob.id(), gustav.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob, &mut charlie, &mut gustav],
        parameter.clone(),
        None,
        Some(playlist.clone()),
    )
    .await;

    // Selecting next selects the first participant in the playlist
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Next), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(vec![bob.id(), gustav.id()])
        }
    );

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(vec![bob.id(), gustav.id()])
        }
    );

    // Selecting charlie is not allowed, because he is not in the playlist
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: charlie.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Selecting a specific participant is allowed
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: bob.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(bob.id()),
            history: Some(vec![alice.id(), bob.id()]),
            remaining: Some(vec![bob.id(), gustav.id()])
        }
    );

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(bob.id()),
            history: Some(vec![alice.id(), bob.id()]),
            remaining: Some(vec![bob.id(), gustav.id()])
        }
    );

    // Yielding with specifying the next participant is not allowed
    bob.send_command::<AutomodModule>(
        AutomodCommand::Yield {
            next: Some(gustav.id()),
        },
        None,
    )
    .await
    .unwrap();

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Yielding without specifying the next participant is allowed
    bob.send_command::<AutomodModule>(AutomodCommand::Yield { next: None }, None)
        .await
        .unwrap();

    let event = bob.receive_event::<AutomodModule>().await.unwrap().payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(bob.id()),
            history: Some(vec![alice.id(), bob.id(), bob.id()]),
            remaining: Some(vec![gustav.id()])
        }
    );

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(bob.id()),
            history: Some(vec![alice.id(), bob.id(), bob.id()]),
            remaining: Some(vec![gustav.id()])
        }
    );

    // Selecting none is allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::None), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: None,
            history: Some(vec![alice.id(), bob.id(), bob.id()]),
            remaining: Some(vec![gustav.id()])
        }
    );

    // Selecting a random participant is allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Random), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(gustav.id()),
            history: Some(vec![alice.id(), bob.id(), bob.id(), gustav.id()]),
            remaining: Some(Vec::new())
        }
    );

    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Next), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Stopped(StoppedReason::SessionFinished));
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_selection_strategy_random() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Random,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: false,
        auto_append_on_join: false,
    };
    let allow_list = vec![alice.id(), bob.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob, &mut charlie],
        parameter.clone(),
        Some(allow_list.clone()),
        None,
    )
    .await;

    // Selecting Charlie is not allowed, because he is not in the allow list
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: charlie.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Selecting a specific participant is allowed
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: alice.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(vec![alice.id(), bob.id()])
        }
    );

    // Yielding with specifying the next participant is not allowed
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Yield {
                next: Some(bob.id()),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Yielding without specifying a participant selects a random participant from the allow list
    alice
        .send_command::<AutomodModule>(AutomodCommand::Yield { next: None }, None)
        .await
        .unwrap();
    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(
        event,
        AutomodEvent::SpeakerUpdated { speaker, .. }
            if speaker == Some(alice.id()) || speaker == Some(bob.id())
    );

    // Selecting none is allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::None), None)
        .await
        .unwrap();
    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, AutomodEvent::SpeakerUpdated { speaker: None, .. });

    // Selecting a random participant is allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Random), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(
        event,
        AutomodEvent::SpeakerUpdated { speaker, .. }
            if speaker == Some(alice.id()) || speaker == Some(bob.id())
    );

    // Selecting next is allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Random), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Stopped(StoppedReason::SessionFinished));
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_selection_strategy_nomination() {
    let (_container, mut room, _public_url) = build_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob, &mut charlie]).await;

    let parameter = Parameter {
        selection_strategy: SelectionStrategy::Nomination,
        show_remaining: true,
        time_limit: None,
        allow_double_selection: false,
        auto_append_on_join: false,
    };
    let allow_list = vec![alice.id(), bob.id(), gustav.id()];
    start_automod(
        &mut alice,
        &mut [&mut bob, &mut charlie, &mut gustav],
        parameter.clone(),
        Some(allow_list.clone()),
        None,
    )
    .await;

    // Selecting next is not allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Next), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Selecting Charlie is not allowed, because he is not in the allow list
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: charlie.id(),
                keep_in_remaining: true,
            }),
            None,
        )
        .await
        .unwrap();
    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, AutomodEvent::Error(AutomodError::InvalidSelection));

    // Selecting a specific participant is allowed
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Select(Select::Specific {
                participant: alice.id(),
                keep_in_remaining: false,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(alice.id()),
            history: Some(vec![alice.id()]),
            remaining: Some(vec![bob.id(), gustav.id()])
        }
    );

    // Yielding with specifying the next participant is allowed
    alice
        .send_command::<AutomodModule>(
            AutomodCommand::Yield {
                next: Some(bob.id()),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(bob.id()),
            history: Some(vec![alice.id(), bob.id()]),
            remaining: Some(vec![gustav.id()])
        }
    );

    // Selecting none is allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::None), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: None,
            history: Some(vec![alice.id(), bob.id()]),
            remaining: Some(vec![gustav.id()])
        }
    );

    // Selecting a random participant is allowed
    alice
        .send_command::<AutomodModule>(AutomodCommand::Select(Select::Random), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<AutomodModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        AutomodEvent::SpeakerUpdated {
            speaker: Some(gustav.id()),
            history: Some(vec![alice.id(), bob.id(), gustav.id()]),
            remaining: Some(Vec::new())
        }
    );
}
