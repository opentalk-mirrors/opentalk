// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{assert_matches, time::Duration};

use insta::assert_snapshot;
use opentalk_roomserver_module_training_participation_report::TrainingParticipationReportModule;
use opentalk_roomserver_room::mocking::{
    participant,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    room_kind::RoomKind,
    room_parameters::EventContext,
};
use opentalk_roomserver_types_training_participation_report::{
    TrainingParticipationReportCommand, TrainingParticipationReportEvent,
    TrainingParticipationReportParameterSet, TrainingParticipationReportState,
    event::{
        PresenceLoggingEndedReason, PresenceLoggingStartedReason, TrainingParticipationReportError,
    },
    settings::TrainingParticipationReportSettings,
    state::ParticipationLoggingState,
};
use opentalk_types_common::{
    events::{EventDescription, EventId, EventTitle},
    training_participation_report::TimeRange,
};

#[test_log::test(tokio::test)]
async fn autostart() {
    let autostart = Some(TrainingParticipationReportParameterSet {
        initial_checkpoint_delay: TimeRange::new_with_clamped_durations(
            Duration::from_secs(100),
            Duration::from_secs(200),
        ),
        checkpoint_interval: TimeRange::new_with_clamped_durations(
            Duration::from_secs(300),
            Duration::from_secs(400),
        ),
    });
    let mut room = TestRoom::builder()
        .add_init_module_data(&TrainingParticipationReportSettings {
            autostart: autostart.clone(),
        })
        .unwrap()
        .register_module::<TrainingParticipationReportModule>()
        .owner(participant::alice_public_profile())
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;

    // Alice sees the whole state because she is the room owner.
    let state = alice
        .join_success()
        .get_module::<TrainingParticipationReportState>()
        .expect("TrainingParticipationReportState must be deserializable")
        .expect("TrainingParticipationReportState must be present");
    assert_eq!(
        state,
        TrainingParticipationReportState {
            state: ParticipationLoggingState::Enabled,
            parameters: autostart
        }
    );

    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;

    // Alice is notified that presence logging has started automatically.
    assert!(
        matches!(
            event,
            TrainingParticipationReportEvent::PresenceLoggingStarted {
                reason: Some(PresenceLoggingStartedReason::Autostart),
                ..
            }
        ),
        "{event:?}"
    );

    let mut bob = room.join_bob(0).await;

    // Bob only sees the state without the parameter set.
    let state = bob
        .join_success()
        .get_module::<TrainingParticipationReportState>()
        .expect("TrainingParticipationReportState must be deserializable")
        .expect("TrainingParticipationReportState must be present");
    assert_eq!(
        state,
        TrainingParticipationReportState {
            state: ParticipationLoggingState::Enabled,
            parameters: None
        }
    );

    // Bob does not not receive the PresenceLoggingStarted event because he joined after presence
    // logging was already started.
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn enable_presence_logging_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<TrainingParticipationReportModule>()
        .owner(participant::bob_public_user_profile())
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::EnablePresenceLogging {
                initial_checkpoint_delay: None,
                checkpoint_interval: None,
            },
            None,
        )
        .await
        .unwrap();

    // Alice is not allowed to enable presence logging, even tough she is a moderator, because she
    // is not the room owner.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::Error(
            TrainingParticipationReportError::InsufficientPermissions
        )
    );
}

#[test_log::test(tokio::test)]
async fn enable_presence_logging_already_enabled() {
    let mut room = TestRoom::builder()
        .add_init_module_data(&TrainingParticipationReportSettings {
            autostart: Some(TrainingParticipationReportParameterSet {
                initial_checkpoint_delay: TimeRange::new_with_clamped_durations(
                    Duration::from_secs(100),
                    Duration::from_secs(200),
                ),
                checkpoint_interval: TimeRange::new_with_clamped_durations(
                    Duration::from_secs(300),
                    Duration::from_secs(400),
                ),
            }),
        })
        .unwrap()
        .register_module::<TrainingParticipationReportModule>()
        .owner(participant::alice_public_profile())
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice is notified that presence logging has started automatically.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(
        event,
        TrainingParticipationReportEvent::PresenceLoggingStarted {
            reason: Some(PresenceLoggingStartedReason::Autostart),
            ..
        }
    );

    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::EnablePresenceLogging {
                initial_checkpoint_delay: None,
                checkpoint_interval: None,
            },
            None,
        )
        .await
        .unwrap();

    // Presence logging is already enabled because the room is configured to autostart presence
    // logging
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::Error(
            TrainingParticipationReportError::PresenceLoggingAlreadyEnabled
        )
    );
}

#[test_log::test(tokio::test)]
async fn enable_presence_logging() {
    let mut room = TestRoom::builder()
        .register_module::<TrainingParticipationReportModule>()
        .owner(participant::alice_public_profile())
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::EnablePresenceLogging {
                initial_checkpoint_delay: None,
                checkpoint_interval: None,
            },
            None,
        )
        .await
        .unwrap();

    // Alice is notified that presence logging has started.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(
        event,
        TrainingParticipationReportEvent::PresenceLoggingStarted {
            reason: Some(PresenceLoggingStartedReason::StartedManually),
            ..
        }
    );
}

#[test_log::test(tokio::test)]
async fn disable_presence_logging_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<TrainingParticipationReportModule>()
        .owner(participant::alice_public_profile())
        .add_init_module_data(&TrainingParticipationReportSettings {
            autostart: Some(TrainingParticipationReportParameterSet {
                initial_checkpoint_delay: TimeRange::new_with_clamped_durations(
                    Duration::from_secs(100),
                    Duration::from_secs(200),
                ),
                checkpoint_interval: TimeRange::new_with_clamped_durations(
                    Duration::from_secs(300),
                    Duration::from_secs(400),
                ),
            }),
        })
        .unwrap()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob is notified that presence logging has started automatically.
    let event = bob
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(
            event,
            // Bob does not see the reason because he is not the room owner.
            TrainingParticipationReportEvent::PresenceLoggingStarted { reason: None, .. }
        ),
        "{event:?}"
    );

    // Bob tries to disable presence logging
    bob.send_command::<TrainingParticipationReportModule>(
        TrainingParticipationReportCommand::DisablePresenceLogging,
        None,
    )
    .await
    .unwrap();

    // Bob is not allowed to disable presence logging because he is not the room owner.
    let event = bob
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::Error(
            TrainingParticipationReportError::InsufficientPermissions
        )
    );
}

#[test_log::test(tokio::test)]
async fn disable_presence_logging_not_enabled() {
    let mut room = TestRoom::builder()
        .register_module::<TrainingParticipationReportModule>()
        .owner(participant::alice_public_profile())
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to disable presence logging
    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::DisablePresenceLogging,
            None,
        )
        .await
        .unwrap();

    // Presence logging is not enabled.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::Error(
            TrainingParticipationReportError::PresenceLoggingNotEnabled
        )
    );
}

#[test_log::test(tokio::test)]
async fn confirm_presence_not_enabled() {
    let mut room = TestRoom::builder()
        .register_module::<TrainingParticipationReportModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to confirm presence
    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::ConfirmPresence,
            None,
        )
        .await
        .unwrap();

    // Presence logging is not enabled.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::Error(
            TrainingParticipationReportError::PresenceLoggingNotEnabled
        )
    );
}

#[test_log::test(tokio::test(start_paused = true))]
async fn stop_presence_logging_manual() {
    let mut room = TestRoom::builder()
        .register_module::<TrainingParticipationReportModule>()
        .owner(participant::alice_public_profile())
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice enables presence logging
    let first_checkpoint_delay = Duration::from_mins(1);
    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::EnablePresenceLogging {
                // Create a checkpoint immediately
                initial_checkpoint_delay: Some(TimeRange::new_with_clamped_durations(
                    first_checkpoint_delay,
                    Duration::ZERO,
                )),
                checkpoint_interval: Some(TimeRange::new_with_clamped_durations(
                    Duration::from_mins(30),
                    Duration::ZERO,
                )),
            },
            None,
        )
        .await
        .unwrap();

    // Alice is notified that presence logging has started.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(
            event,
            TrainingParticipationReportEvent::PresenceLoggingStarted {
                reason: Some(PresenceLoggingStartedReason::StartedManually),
                ..
            }
        ),
        "{event:?}"
    );

    // Bob is notified that presence logging has started.
    let event = bob
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(
            event,
            TrainingParticipationReportEvent::PresenceLoggingStarted { reason: None, .. }
        ),
        "{event:?}"
    );

    // Advance time to trigger the first checkpoint
    tokio::time::advance(first_checkpoint_delay).await;

    // Alice and Bob are requested to confirm their presence.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::PresenceConfirmationRequested
    );

    let event = bob
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::PresenceConfirmationRequested
    );

    // Alice confirms her presence
    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::ConfirmPresence,
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::PresenceConfirmationLogged
    );

    // Bob does not confirm his presence and Alice stops presence logging
    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::DisablePresenceLogging,
            None,
        )
        .await
        .unwrap();

    // Alice is notified that presence logging has stopped.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::PresenceLoggingEnded {
            reason: PresenceLoggingEndedReason::StoppedManually
        }
    );

    // Bob is notified that presence logging has stopped.
    let event = bob
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::PresenceLoggingEnded {
            reason: PresenceLoggingEndedReason::StoppedManually
        }
    );

    // Alice is notified that a report has been generated.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    let TrainingParticipationReportEvent::PdfCreated { asset_id, .. } = event else {
        panic!("Expected `TrainingParticipationReportEvent::PdfAsset`, received {event:?}");
    };

    // Bob is not notified about the report because he is not the room owner.
    assert!(bob.received_nothing());

    let report = room.stored_asset(asset_id).await.unwrap();
    let content = pdf_extract::extract_text_from_mem(&report).unwrap();

    insta::with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{2}-[0-9]{2}", "[date]"),
            (r"[0-9]{2}:[0-9]{2}", "[time]"),
        ]}, {
        assert_snapshot!(content, @r"
        Training participation report
         Meeting:

        Description: —

        Report timezone: Europe/Berlin

        Training start: [date] [time]

        Training end: [date] [time]

        Participation checkpoints
         № Person [time]

        1 Alice the angry [time]

        2 Bob the bold —
        ");
    });
}

#[test_log::test(tokio::test(start_paused = true))]
async fn stop_presence_logging_auto() {
    let first_checkpoint_delay = Duration::from_mins(1);
    let mut room = TestRoom::builder()
        .register_module::<TrainingParticipationReportModule>()
        .event(EventContext {
            id: EventId::generate(),
            title: EventTitle::from_str_lossy("Training Session"),
            description: EventDescription::from_str_lossy("Description"),
            is_adhoc: true,
            starts_at: None,
            ends_at: None,
            shared_folder: None,
        })
        .owner(participant::alice_public_profile())
        // The room is configured to autostart presence logging when the first participant joins.
        .add_init_module_data(&TrainingParticipationReportSettings {
            autostart: Some(TrainingParticipationReportParameterSet {
                initial_checkpoint_delay: TimeRange::new_with_clamped_durations(
                    first_checkpoint_delay,
                    Duration::ZERO,
                ),
                checkpoint_interval: TimeRange::new_with_clamped_durations(
                    Duration::from_mins(30),
                    Duration::from_mins(1),
                ),
            }),
        })
        .unwrap()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(
        event,
        TrainingParticipationReportEvent::PresenceLoggingStarted {
            reason: Some(PresenceLoggingStartedReason::Autostart),
            ..
        }
    );

    // Advance time to trigger the first checkpoint
    tokio::time::advance(first_checkpoint_delay).await;

    // Alice is requested to confirm her presence.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::PresenceConfirmationRequested
    );

    // Alice leaves
    alice.disconnect().await.unwrap();

    // The pdf is automatically generated when the last participant leaves
    let assets = room.stored_assets().await;
    assert_eq!(assets.len(), 1);
    let content = pdf_extract::extract_text_from_mem(&assets[0]).unwrap();
    insta::with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{2}-[0-9]{2}", "[date]"),
            (r"[0-9]{2}:[0-9]{2}", "[time]"),
        ]}, {
        assert_snapshot!(content, @r"
        Training participation report
         Meeting: Training Session

        Description: Description

        Report timezone: Europe/Berlin

        Training start: [date] [time]

        Training end: [date] [time]

        Participation checkpoints
         № Person [time]

        1 Alice the angry —
        ");
    });
}

#[test_log::test(tokio::test(start_paused = true))]
async fn alice_in_breakout_bob_in_main() {
    let mut room = TestRoom::builder()
        .register_module::<TrainingParticipationReportModule>()
        .owner(participant::alice_public_profile())
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".into(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;

    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice enables presence logging in the breakout room
    let first_checkpoint_delay = Duration::from_mins(1);
    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::EnablePresenceLogging {
                initial_checkpoint_delay: Some(TimeRange::new_with_clamped_durations(
                    first_checkpoint_delay,
                    Duration::ZERO,
                )),
                checkpoint_interval: Some(TimeRange::new_with_clamped_durations(
                    Duration::from_mins(30),
                    Duration::from_mins(1),
                )),
            },
            None,
        )
        .await
        .unwrap();

    // Alice is notified that presence logging has started.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(
        event,
        TrainingParticipationReportEvent::PresenceLoggingStarted {
            reason: Some(PresenceLoggingStartedReason::StartedManually),
            ..
        }
    );

    tokio::time::advance(first_checkpoint_delay).await;

    // Alice is requested to confirm her presence.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;

    assert_eq!(
        event,
        TrainingParticipationReportEvent::PresenceConfirmationRequested
    );

    // Bob is not notified because he is in the main room.
    assert!(bob.received_nothing());

    // Alice disables presence logging
    alice
        .send_command::<TrainingParticipationReportModule>(
            TrainingParticipationReportCommand::DisablePresenceLogging,
            None,
        )
        .await
        .unwrap();

    // Alice is notified that presence logging has stopped.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        TrainingParticipationReportEvent::PresenceLoggingEnded {
            reason: PresenceLoggingEndedReason::StoppedManually
        }
    );

    // Bob is not notified because he is in the main room.
    assert!(bob.received_nothing());

    // Alice is notified that a report has been generated.
    let event = alice
        .receive_event::<TrainingParticipationReportModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, TrainingParticipationReportEvent::PdfCreated { .. });

    // Bob is not notified because he is in the main room and not the room owner.
    assert!(bob.received_nothing());
}
