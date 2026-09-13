// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{assert_matches, collections::BTreeSet, time::Duration};

use opentalk_roomserver_module_meeting_notes::MeetingNotesModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};
use opentalk_roomserver_types_meeting_notes::{
    MEETING_NOTES_MODULE_ID, MeetingNotesCommand, MeetingNotesError, MeetingNotesEvent,
    MeetingNotesPeerState, MeetingNotesSettings,
};
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;

use crate::common::ETHERPAD_API_KEY;

mod common;

#[test_log::test(tokio::test)]
async fn join_success() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice receives the AccessChanged event because she is a moderator
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![bob.connection_id()],
            writers: vec![alice.connection_id()]
        }
    );

    let frank = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let frank_state_for_alice = frank
        .join_success()
        .participants
        .iter()
        .find(|p| p.id == alice.id())
        .unwrap()
        .get_module::<MeetingNotesPeerState>()
        .expect("Meeting notes peer data must be deserializable")
        .expect("Meeting notes peer data must be present");
    assert!(!frank_state_for_alice.readonly);

    let frank_state_for_bob = frank
        .join_success()
        .participants
        .iter()
        .find(|p| p.id == bob.id())
        .unwrap()
        .get_module::<MeetingNotesPeerState>()
        .expect("Meeting notes peer data must be deserializable")
        .expect("Meeting notes peer data must be present");

    assert!(frank_state_for_bob.readonly);
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions_select_writer() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings {
            base_url: "http://localhost:9001".parse().unwrap(),
            api_key: ETHERPAD_API_KEY.into(),
        })
        .unwrap()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to initialize the meeting notes by setting himself as a writer
    bob.send_command::<MeetingNotesModule>(
        MeetingNotesCommand::GrantWriteAccess {
            participant_ids: BTreeSet::from_iter([bob.id()]),
        },
        None,
    )
    .await
    .unwrap();

    // Bob does not have the required permissions, because he isn't a moderator
    let event = bob
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions_deselect_writer() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings {
            base_url: "http://localhost:9001".parse().unwrap(),
            api_key: ETHERPAD_API_KEY.into(),
        })
        .unwrap()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to initialize the meeting notes by setting himself as a writer
    bob.send_command::<MeetingNotesModule>(
        MeetingNotesCommand::RevokeWriteAccess {
            participant_ids: BTreeSet::from_iter([bob.id()]),
        },
        None,
    )
    .await
    .unwrap();

    // Bob does not have the required permissions, because he isn't a moderator
    let event = bob
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn insufficient_permissions_generate_pdf() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings {
            base_url: "http://localhost:9001".parse().unwrap(),
            api_key: ETHERPAD_API_KEY.into(),
        })
        .unwrap()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to initialize the meeting notes by setting himself as a writer
    bob.send_command::<MeetingNotesModule>(MeetingNotesCommand::GeneratePdf, None)
        .await
        .unwrap();

    // Bob does not have the required permissions, because he isn't a moderator
    let event = bob
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn unknown_participant() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings {
            base_url: "http://localhost:9001".parse().unwrap(),
            api_key: ETHERPAD_API_KEY.into(),
        })
        .unwrap()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to make a non existing participant a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([ParticipantId::from_u128(0xb0b)]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::InvalidParticipantSelection)
    );
}

#[test_log::test(tokio::test)]
async fn grant_access_no_participants() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings {
            base_url: "http://localhost:9001".parse().unwrap(),
            api_key: ETHERPAD_API_KEY.into(),
        })
        .unwrap()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to grant access with an empty participant list
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::new(),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::InvalidParticipantSelection)
    );
}

#[test_log::test(tokio::test)]
async fn grant_access_while_initializing() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;

    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts the meeting notes by selecting herself as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                // Only alice is a writer
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice tries to also select Bob as a writer before the initialization is complete
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: [bob.id()].into(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives an error for Bob
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::CurrentlyInitializing)
    );

    // Alice still receives a write URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Bob receives a read-only URL
    let event = bob
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::ReadAccessReceived { .. }),
        "{event:#?}"
    );
}

#[test_log::test(tokio::test)]
async fn revoke_access_while_initializing() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the meeting notes by selecting herself as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                // Only alice is a writer
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice tries to deselect herself as a writer before the initialization is complete
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::RevokeWriteAccess {
                participant_ids: [alice.id()].into(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives an error for the deselect request
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::CurrentlyInitializing)
    );

    // Alice still receives a write URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );
}

#[test_log::test(tokio::test)]
async fn generate_pdf_currently_initializing() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the meeting notes by selecting herself as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                // Only alice is a writer
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice tries to generate a PDF before the initialization is complete
    alice
        .send_command::<MeetingNotesModule>(MeetingNotesCommand::GeneratePdf, None)
        .await
        .unwrap();

    // Alice receives an error for the generate PDF request
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::CurrentlyInitializing)
    );

    // Alice still receives a write URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );
}

#[test_log::test(tokio::test)]
async fn revoke_access_not_initialized() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings {
            base_url: "http://localhost:9001".parse().unwrap(),
            api_key: ETHERPAD_API_KEY.into(),
        })
        .unwrap()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to deselect herself as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::RevokeWriteAccess {
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives an error, because the meeting notes have not been initialized
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::NotInitialized)
    );
}

#[test_log::test(tokio::test)]
async fn generate_pdf_not_initialized() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings {
            base_url: "http://localhost:9001".parse().unwrap(),
            api_key: ETHERPAD_API_KEY.into(),
        })
        .unwrap()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to generate a PDF
    alice
        .send_command::<MeetingNotesModule>(MeetingNotesCommand::GeneratePdf, None)
        .await
        .unwrap();

    // Alice receives an error, because the meeting notes have not been initialized
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::NotInitialized)
    );
}

#[test_log::test(tokio::test)]
async fn enable_meeting_notes() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;

    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                // Only alice is a writer
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives a write URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice receives the AccessChanged event because she is a moderator
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![bob.connection_id()],
            writers: vec![alice.connection_id()]
        }
    );

    // Bob receives a read-only URL
    let event = bob
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::ReadAccessReceived { .. }),
        "{event:#?}"
    );
}

#[test_log::test(tokio::test)]
async fn grant_access() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts the meeting notes by selecting herself as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                // Only alice is a writer
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives a write URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice receives the AccessChanged event because she is a moderator
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![bob.connection_id()],
            writers: vec![alice.connection_id()]
        }
    );

    // Bob receives a read-only URL
    let event = bob
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, MeetingNotesEvent::ReadAccessReceived { .. });

    // Alice selects Bob as a writer too
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![],
            writers: vec![bob.connection_id()]
        }
    );

    // Bob receives a write URL
    let event = bob
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );
}

#[test_log::test(tokio::test)]
async fn revoke_access() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives a write URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice receives the AccessChanged event because she is a moderator
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![],
            writers: vec![alice.connection_id()]
        }
    );

    // Alice deselects herself as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::RevokeWriteAccess {
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives a read-only URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::ReadAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice receives the AccessChanged event because she is a moderator
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![alice.connection_id()],
            writers: vec![]
        }
    );
}

#[test_log::test(tokio::test)]
async fn generate_pdf() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives a write URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice receives the AccessChanged event because she is a moderator
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![],
            writers: vec![alice.connection_id()]
        }
    );

    // Alice generates a PDF
    alice
        .send_command::<MeetingNotesModule>(MeetingNotesCommand::GeneratePdf, None)
        .await
        .unwrap();

    // Alice receives a PDF URL
    let event = alice
        .receive_event_with_timeout::<MeetingNotesModule>(Duration::from_secs(5))
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::PdfCreated { .. }),
        "{event:#?}"
    );
}

#[test_log::test(tokio::test)]
async fn alice_in_breakout_bob_in_main() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

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

    // Alice starts the meeting notes by selecting herself as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives a write URL
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice receives the AccessChanged event because she is a moderator
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![],
            writers: vec![alice.connection_id()]
        }
    );

    // Bob does not receive any event, because he is in the main room
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn can_not_grant_access_from_other_room() {
    let mut room = TestRoom::builder()
        .register_module::<MeetingNotesModule>()
        .add_init_module_data(&MeetingNotesSettings {
            base_url: "http://localhost:9001".parse().unwrap(),
            api_key: ETHERPAD_API_KEY.into(),
        })
        .unwrap()
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
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice tries to start the meeting notes by selecting bob as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([bob.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::Error(MeetingNotesError::InvalidParticipantSelection)
    );

    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn switch_breakout_room() {
    let (_container, mut room) = common::build_etherpad_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut frank = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut frank, &mut bob],
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
        .switch_breakout_room(&mut [&mut frank, &mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice starts the meeting notes by selecting herself as a writer
    alice
        .send_command::<MeetingNotesModule>(
            MeetingNotesCommand::GrantWriteAccess {
                participant_ids: BTreeSet::from_iter([alice.id()]),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice receives the AccessChanged event because she is a moderator
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        MeetingNotesEvent::AccessChanged {
            readers: vec![],
            writers: vec![alice.connection_id()]
        }
    );

    // Frank switches to the breakout room
    let event = frank
        .switch_breakout_room(&mut [&mut alice, &mut bob], RoomKind::Breakout(0.into()))
        .await;

    let BreakoutEvent::SwitchedRoom { peer_data, .. } = event else {
        panic!("Received wrong breakout event: {event:#?}");
    };
    let alice_json = peer_data
        .get(&alice.id())
        .unwrap()
        .get(&MEETING_NOTES_MODULE_ID)
        .expect("Meeting notes peer data must be present");
    let frank_state_for_alice: MeetingNotesPeerState =
        serde_json::from_value(alice_json.clone_inner())
            .expect("Meeting notes peer data must be deserializable");
    assert!(!frank_state_for_alice.readonly);

    let bob_json = peer_data
        .get(&bob.id())
        .unwrap()
        .get(&MEETING_NOTES_MODULE_ID)
        .expect("Meeting notes peer data must be present");
    let frank_state_for_bob: MeetingNotesPeerState = serde_json::from_value(bob_json.clone_inner())
        .expect("Meeting notes peer data must be deserializable");
    assert!(frank_state_for_bob.readonly);

    // Frank receives a write URL
    let event = frank
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, MeetingNotesEvent::WriteAccessReceived { .. }),
        "{event:#?}"
    );

    // Alice and Frank receive the AccessChanged event because they are moderators
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    let expected = MeetingNotesEvent::AccessChanged {
        readers: vec![],
        writers: vec![frank.connection_id()],
    };
    assert_eq!(event, expected);

    let event = frank
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, expected);

    // Bob switches to the breakout room
    let event = bob
        .switch_breakout_room(&mut [&mut alice, &mut frank], RoomKind::Breakout(0.into()))
        .await;
    let BreakoutEvent::SwitchedRoom { peer_data, .. } = event else {
        panic!("Received wrong breakout event: {event:#?}");
    };

    // Bob does not receive any peer data because he isn't a moderator
    assert!(!peer_data.contains_key(&alice.id()));
    assert!(!peer_data.contains_key(&frank.id()));

    // Bob receives a read-only URL
    let event = bob
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, MeetingNotesEvent::ReadAccessReceived { .. });

    // Alice and Frank receive the AccessChanged event because they are moderators
    let event = alice
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    let expected = MeetingNotesEvent::AccessChanged {
        readers: vec![bob.connection_id()],
        writers: vec![],
    };
    assert_eq!(event, expected);

    let event = frank
        .receive_event::<MeetingNotesModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, expected);
}
