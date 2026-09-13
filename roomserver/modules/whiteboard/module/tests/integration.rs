// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{assert_matches, time::Duration};

use opentalk_roomserver_module_whiteboard::WhiteboardModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};
use opentalk_roomserver_types_whiteboard::{
    WhiteboardCommand, WhiteboardError, WhiteboardEvent, WhiteboardState,
};
use pretty_assertions::assert_eq;

mod common;

#[test_log::test(tokio::test)]
async fn whiteboard_join_success() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    // Alice does not receive any state from the whiteboard module, because the initialize command
    // has not been sent
    assert!(
        alice
            .join_success()
            .get_module::<WhiteboardState>()
            .expect("WhiteboardState must be serializable")
            .is_none()
    );

    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });

    let bob = room.join_bob(0).await;
    let state = bob
        .join_success()
        .get_module::<WhiteboardState>()
        .expect("Whiteboard state must be serializable")
        .expect("Whiteboard state must be present");
    assert_matches!(state, WhiteboardState::Initialized(..));
}

#[test_log::test(tokio::test)]
async fn initialize_insufficient_permissions() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut bob = room.join_bob(0).await;

    bob.send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    // Bob is not allowed to initialize the whiteboard because he isn't a moderator
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::InsufficientPermissions)
    )
}

#[test_log::test(tokio::test)]
async fn generate_pdf_insufficient_permissions() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts the whiteboard
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    // Both receive the initialization started event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    // Both receive the initialized event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });

    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });

    // Bob tries to generate a PDF
    bob.send_command::<WhiteboardModule>(WhiteboardCommand::GeneratePdf, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    // Bob is not allowed to generate a PDF because he isn't a moderator
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn currently_initializing() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    // Alice tries to initialize the whiteboard twice
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    // Alice receives the initialization started event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    // Alice receives the error response immediately...
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::CurrentlyInitializing)
    );

    // ...and the initialization event when initialization completed
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });
}

#[test_log::test(tokio::test)]
async fn already_initialized() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    // Alice initializes the whiteboard
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });

    // Alice tries to initialize the whiteboard again
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    // The whiteboard was already initialized
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::AlreadyInitialized)
    );
}

#[test_log::test(tokio::test)]
async fn generate_pdf_not_initialized() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to generate a PDF without initializing the whiteboard
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::GeneratePdf, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    // The whiteboard was not initialized yet
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::NotInitialized)
    );
}

#[test_log::test(tokio::test)]
async fn generate_pdf_while_initializing() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    // Alice initializes the whiteboard
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    // Alice tries to generate a PDF while the whiteboard is still initializing
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::GeneratePdf, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    // Alice receives the error response immediately...
    assert_eq!(
        event,
        WhiteboardEvent::Error(WhiteboardError::CurrentlyInitializing)
    );
    // ...and the initialization event when initialization completed
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });
}

#[test_log::test(tokio::test)]
async fn generate_pdf() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice initializes the whiteboard
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    // Both receive the initialization started and initialized event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });

    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });

    // Alice generates a PDF
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::GeneratePdf, None)
        .await
        .unwrap();

    // Both receive the PDF generated event
    let event = alice
        .receive_event_with_timeout::<WhiteboardModule>(Duration::from_secs(10))
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::PdfCreated { .. });

    let event = bob
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::PdfCreated { .. });

    assert_eq!(room.file_count().await, 1);
}

#[test_log::test(tokio::test)]
async fn alice_in_breakout_bob_in_main() {
    let (_container, mut room) = common::build_whiteboard_room().await;
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Breakout Room 1".to_string(),
                    assignments: Vec::new(),
                }],
                duration: None,
            },
        )
        .await;

    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice initializes the whiteboard in the breakout room
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::Initialize, None)
        .await
        .unwrap();

    // Alice receives the initialization started and initialized event
    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, WhiteboardEvent::InitializationStarted);

    let event = alice
        .receive_event::<WhiteboardModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::Initialized { .. });

    // Bob does not receive any state from the whiteboard module, because he is in the main room
    assert!(bob.received_nothing());

    // Alice generates a PDF in the breakout room
    alice
        .send_command::<WhiteboardModule>(WhiteboardCommand::GeneratePdf, None)
        .await
        .unwrap();

    // Alice receives the PDF generated event
    let event = alice
        .receive_event_with_timeout::<WhiteboardModule>(Duration::from_secs(5))
        .await
        .unwrap()
        .payload;
    assert_matches!(event, WhiteboardEvent::PdfCreated { .. });

    // Bob does not receive any event from the whiteboard module, because he is in the main room
    assert!(bob.received_nothing());

    // Bob switches to the breakout room
    let event = bob
        .switch_breakout_room(&mut [&mut alice], RoomKind::Breakout(0.into()))
        .await;

    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Expected SwitchedRoom event, got {event:#?}");
    };
    let state = own_data
        .get::<WhiteboardState>()
        .expect("Whiteboard data must be serializable")
        .expect("Whiteboard data must be present");
    assert_matches!(state, WhiteboardState::Initialized { .. });
}
