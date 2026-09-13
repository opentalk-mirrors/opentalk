// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{assert_matches, num::NonZero};

use opentalk_roomserver_module_chat::ChatModule;
use opentalk_roomserver_room::mocking::{
    participant::{JoinedState, MockParticipant, MockParticipantJoined},
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_signaling::signaling_module::SignalingModule;
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        event::BreakoutEvent,
    },
    core::CoreEvent,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_chat::{
    ChatSettings, MessageId, RateLimitSettings, Scope,
    command::ChatCommand,
    event::{ChatError, ChatEvent},
    state::{
        BreakoutHistory, CHAT_CHUNK_SIZE, ChatChunk, ChatState, PrivateHistory, StoredMessage,
    },
};
use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;

// This macro shall make comparing messages more readable.
/// Compare the [`MessageSent`] struct with the provided scope, content and source.
/// The macro also ensures that the panic still shows the call site as code location.
/// If a function would be used, the panic would show the function as location regardless
/// of the test it's used in.
macro_rules! assert_message_eq {
    ($expected_scope:expr, $expected_content:expr, $expected_source:expr, $event:expr$(,)?) => {{
        // Evaluate the event expression exactly once (prevents double awaiting).
        let event = $event;
        if let &ChatEvent::MessageSent { id, .. } = event {
            pretty_assertions::assert_eq!(
                event,
                &ChatEvent::MessageSent {
                    id,
                    source: $expected_source,
                    content: $expected_content.to_string(),
                    scope: $expected_scope.clone(),
                }
            );
        } else {
            panic!("Expected ChatEvent::MessageSent, but got: {:?}", event);
        }
    }};
}

/// Once the chat is disabled, messages cannot be sent.
#[test_log::test(tokio::test)]
async fn chat_is_disabled() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ChatModule>(ChatCommand::DisableChat, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::ChatDisabled {
            issued_by: alice.id()
        }
    );

    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::ChatDisabled {
            issued_by: alice.id()
        }
    );

    // Alice cannot send a global message
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Global,
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::Error(ChatError::ChatDisabled)
    );

    // Alice cannot send a private message
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Private(bob.id()),
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::Error(ChatError::ChatDisabled)
    );

    // Bob cannot send a private message
    bob.send_command::<ChatModule>(
        ChatCommand::SendMessage {
            content: "Hi there".to_string(),
            scope: Scope::Private(alice.id()),
        },
        None,
    )
    .await
    .unwrap();
    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::Error(ChatError::ChatDisabled)
    );
}

/// The chat should work after disabling and enabling it again.
#[test_log::test(tokio::test)]
async fn chat_works_after_enabling() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Disabling the chat should broadcast the ChatDisabled event
    alice
        .send_command::<ChatModule>(ChatCommand::DisableChat, None)
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::ChatDisabled {
            issued_by: alice.id()
        }
    );
    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::ChatDisabled {
            issued_by: alice.id()
        }
    );

    // Enabling the chat should broadcast the ChatEnabled event
    alice
        .send_command::<ChatModule>(ChatCommand::EnableChat, None)
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::ChatEnabled {
            issued_by: alice.id()
        }
    );
    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::ChatEnabled {
            issued_by: alice.id()
        }
    );

    // Alice can send a global message, bob and alice should receive the message
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Global,
            },
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Global,
        "Hi there",
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().payload,
    );
    assert_message_eq!(
        &Scope::Global,
        "Hi there",
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().payload,
    );

    // Alice can send a private message, bob and alice should receive the message
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Private(bob.id()),
            },
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Private(bob.id()),
        "Hi there",
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().payload,
    );
    assert_message_eq!(
        &Scope::Private(alice.id()),
        "Hi there",
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().payload,
    );
}

/// Private messages should not be received by participants that are not invited.
#[test_log::test(tokio::test)]
async fn private_messages_are_private() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Private messages should not be received by third parties
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "Hi there".to_string(),
                scope: Scope::Private(bob.id()),
            },
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Private(bob.id()),
        "Hi there",
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().payload,
    );
    assert_message_eq!(
        &Scope::Private(alice.id()),
        "Hi there",
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().payload,
    );
    assert!(charlie.received_nothing());
}

/// The global chat should be cleared, leaving private messages untouched.
///
/// 1. Alice and Bob join the room
/// 2. Alice sends a global message
/// 3. Alice sends a private message
/// 4. Alice clears the chat
/// 5. Bob leaves and rejoins, the JoinSuccess should only contain the private message
#[test_log::test(tokio::test)]
async fn global_chat_is_cleared() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    // Bob joins
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // populate the chat history both private and global

    // Alice can send a global message, bob and alice should receive the message
    let global_message = "Hi there";
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: global_message.to_string(),
                scope: Scope::Global,
            },
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Global,
        global_message,
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().payload,
    );
    assert_message_eq!(
        &Scope::Global,
        global_message,
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().payload,
    );

    // Alice can send a private message, bob and alice should receive the message
    let private_message = "Hi there from alice";
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: private_message.to_string(),
                scope: Scope::Private(bob.id()),
            },
            None,
        )
        .await
        .unwrap();
    assert_message_eq!(
        &Scope::Private(bob.id()),
        private_message,
        alice.id(),
        &alice.receive_event::<ChatModule>().await.unwrap().payload,
    );
    assert_message_eq!(
        &Scope::Private(alice.id()),
        private_message,
        alice.id(),
        &bob.receive_event::<ChatModule>().await.unwrap().payload,
    );

    // clear the chat
    alice
        .send_command::<ChatModule>(ChatCommand::ClearHistory, None)
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::HistoryCleared {
            issued_by: alice.id()
        }
    );
    assert_eq!(
        bob.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::HistoryCleared {
            issued_by: alice.id()
        }
    );

    // When bob reconnects, the join success should only contain the private message
    bob.disconnect().await.unwrap();
    alice.receive::<CoreEvent>().await.unwrap();

    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let chat_state = bob
        .join_success()
        .get_module::<<ChatModule as SignalingModule>::JoinInfo>()
        .expect("ChatState must be valid")
        .expect("ChatState must exist");

    assert!(chat_state.global_history.messages.is_empty());
    assert_eq!(
        chat_state
            .private_history
            .iter()
            .map(|h| h.correspondent)
            .collect::<Vec<_>>(),
        vec![alice.id()],
        "There should be one private chat with alice"
    );
    assert_eq!(
        chat_state.private_history[0]
            .history
            .messages
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>(),
        vec![private_message],
        "There should be one message from alice"
    );
}

/// Set the last seen timestamp
///
/// 1. Alice join the room
/// 2. Alice sets last seen timestamp for global and private chat
/// 3. Alice rejoins, the JoinSuccess should contain the last seen timestamps
#[test_log::test(tokio::test)]
async fn last_seen_timestamp_should_be_stored() {
    let timestamp = Timestamp::now();
    let other_participant = ParticipantId::generate();

    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    // Alice joins
    let mut alice = room.join_alice_moderator(0).await;

    // set global last seen timestamp
    alice
        .send_command::<ChatModule>(
            ChatCommand::SetLastSeenTimestamp {
                scope: Scope::Global,
                timestamp,
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::SetLastSeenTimestamp {
            scope: Scope::Global,
            timestamp,
        }
    );

    // set private last seen timestamp
    alice
        .send_command::<ChatModule>(
            ChatCommand::SetLastSeenTimestamp {
                scope: Scope::Private(other_participant),
                timestamp,
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<ChatModule>().await.unwrap().payload,
        ChatEvent::SetLastSeenTimestamp {
            scope: Scope::Private(other_participant),
            timestamp,
        }
    );

    alice.disconnect().await.unwrap();
    let alice = room.join_alice_moderator(0).await;
    let chat_state = alice
        .join_success()
        .get_module::<<ChatModule as SignalingModule>::JoinInfo>()
        .expect("ChatState must be valid")
        .expect("ChatState must exist");
    assert_eq!(chat_state.last_seen_timestamp_global, Some(timestamp));
    assert_eq!(
        chat_state
            .last_seen_timestamps_private
            .get(&other_participant)
            .copied(),
        Some(timestamp)
    );
}

#[test_log::test(tokio::test)]
async fn last_seen_timestamp_is_private() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let timestamp = Timestamp::now();
    alice
        .send_command::<ChatModule>(
            ChatCommand::SetLastSeenTimestamp {
                scope: Scope::Global,
                timestamp,
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the event
    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ChatEvent::SetLastSeenTimestamp {
            scope: Scope::Global,
            timestamp,
        }
    );

    // Bob does not receive the event
    assert!(bob.received_nothing());
}

/// Send a message in the breakout room scope
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Alice sends a message with the breakout room 1 scope
/// 2. Alice and Bob receive the message
/// 3. Charlie won't receive anything
#[test_log::test(tokio::test)]
async fn breakout_scope_messages() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    let breakout_message: String = "breakout_message1".into();
    let breakout_scope = Scope::Breakout(BreakoutId::from(1));

    // Alice sends a message to the breakout room
    breakout_alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: breakout_message.clone(),
                scope: breakout_scope.clone(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the message
    assert_message_eq!(
        &breakout_scope,
        breakout_message,
        breakout_alice.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    // Bob receives the message
    assert_message_eq!(
        &breakout_scope,
        breakout_message,
        breakout_alice.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    // Charlie is clueless
    assert!(main_room_charlie.received_nothing());
}

/// Send a message in the breakout room scope
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Charlie attempts to send a message to breakout room 1
/// 2. Charlie receives an error
#[test_log::test(tokio::test)]
async fn invalid_breakout_scope() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    let breakout_message: String = "breakout_message1".into();
    let breakout_scope = Scope::Breakout(BreakoutId::from(1));

    // Charlie sends a message to breakout room 1
    main_room_charlie
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: breakout_message.clone(),
                scope: breakout_scope.clone(),
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        main_room_charlie
            .receive::<ChatEvent>()
            .await
            .unwrap()
            .payload,
        ChatEvent::Error(ChatError::InvalidBreakoutScope)
    );

    assert!(breakout_alice.received_nothing());
    assert!(breakout_bob.received_nothing());
}

/// Send a message in global room scope from a breakout room
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Alice sends a message with global scope
/// 2. Alice, Bob and Charlie receive the message
#[test_log::test(tokio::test)]
async fn send_global_message_from_breakout_room() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    let global_message_from_breakout_room: String = "global message from breakout room".into();
    let global_scope = Scope::Global;

    // Alice sends a message to the global room
    breakout_alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: global_message_from_breakout_room.clone(),
                scope: global_scope.clone(),
            },
            None,
        )
        .await
        .unwrap();

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        breakout_alice.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        breakout_alice.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        breakout_alice.id(),
        &main_room_charlie
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );
}

/// Send a message in global room scope from the main room to a breakout room
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 3. Charlie sends a message with global scope
/// 4. Alice, Bob and Charlie receive the message
#[test_log::test(tokio::test)]
async fn send_global_message_to_breakout_room() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    let global_message_from_breakout_room: String = "global message from breakout room".into();
    let global_scope = Scope::Global;

    // Charlie sends a message to the global room
    main_room_charlie
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: global_message_from_breakout_room.clone(),
                scope: global_scope.clone(),
            },
            None,
        )
        .await
        .unwrap();

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        main_room_charlie.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        main_room_charlie.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    assert_message_eq!(
        &global_scope,
        global_message_from_breakout_room,
        main_room_charlie.id(),
        &main_room_charlie
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );
}

/// Send a message with private scope between breakout rooms
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Alice sends a private message to Bob (Breakout1 -> Breakout1)
/// 2. Alice and Bob receive the message
#[test_log::test(tokio::test)]
async fn send_private_message_breakout_to_breakout() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    // Alice sends a private message to Bob (Breakout1 -> Breakout1)
    let message: String = "private message from breakout to breakout room".into();
    breakout_alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: message.clone(),
                scope: Scope::Private(breakout_bob.id()),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives her own message
    assert_message_eq!(
        &Scope::Private(breakout_bob.id()),
        message,
        breakout_alice.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    // Bob receives the message from Alice
    assert_message_eq!(
        &Scope::Private(breakout_alice.id()),
        message,
        breakout_alice.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    // Charlie wasn't involved
    assert!(main_room_charlie.received_nothing());
}

/// Send a message with private scope between a breakout room and the main room
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Bob sends a private message to Charlie (Breakout1 -> Main)
/// 2. Bob and Charlie receive the message
#[test_log::test(tokio::test)]
async fn send_private_message_breakout_to_main() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    // Bob sends a private message to Charlie (Breakout1 -> Main)
    let message: String = "private message from breakout to main room".into();
    breakout_bob
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: message.clone(),
                scope: Scope::Private(main_room_charlie.id()),
            },
            None,
        )
        .await
        .unwrap();

    // Bob receives his own message
    assert_message_eq!(
        &Scope::Private(main_room_charlie.id()),
        message,
        breakout_bob.id(),
        &breakout_bob
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    // Charlie receives the message from Bob
    assert_message_eq!(
        &Scope::Private(breakout_bob.id()),
        message,
        breakout_bob.id(),
        &main_room_charlie
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    // Alice wasn't involved
    assert!(breakout_alice.received_nothing());
}

/// Send a message with private scope between the main and a breakout room
///
/// Uses the scenario from [start_breakout_scenario]
/// - Alice & Bob are in breakout room 1
/// - Charlie is in the main room
///
/// 1. Charlie sends a private message to Alice (Main -> Breakout1)
/// 2. Charlie and Alice receive the message
#[test_log::test(tokio::test)]
async fn send_private_message_main_to_breakout() {
    let breakout_scenario = start_breakout_scenario().await;

    let mut breakout_alice = breakout_scenario.alice;
    let mut breakout_bob = breakout_scenario.bob;
    let mut main_room_charlie = breakout_scenario.charlie;

    // Charlie sends a private message to Alice (Main -> Breakout1)
    let message: String = "private message from main to breakout room".into();
    main_room_charlie
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: message.clone(),
                scope: Scope::Private(breakout_alice.id()),
            },
            None,
        )
        .await
        .unwrap();

    // Charlie receives his own message
    assert_message_eq!(
        &Scope::Private(breakout_alice.id()),
        message,
        main_room_charlie.id(),
        &main_room_charlie
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    // Alice receives the message from charlie
    assert_message_eq!(
        &Scope::Private(main_room_charlie.id()),
        message,
        main_room_charlie.id(),
        &breakout_alice
            .receive_event::<ChatModule>()
            .await
            .unwrap()
            .payload,
    );

    // Bob wasn't involved
    assert!(breakout_bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn send_private_message_unknown_participant() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "hello".into(),
                scope: Scope::Private(ParticipantId::nil()),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(event, ChatEvent::Error(ChatError::UnknownParticipant));
}

/// Return type for [start_breakout_scenario]
struct BreakoutScenario<S> {
    _room: TestRoom,
    alice: MockParticipant<S>,
    bob: MockParticipant<S>,
    charlie: MockParticipant<S>,
}

/// Starts three breakout rooms with alice, bob and charlie
///
/// Alice and Bob will join the first breakout room but charlie will stay in the main room
async fn start_breakout_scenario() -> BreakoutScenario<JoinedState> {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();

    let mut alice = room.join_alice_moderator(1).await;
    let mut bob = room.join_bob(1).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(1).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Alice starts breakout rooms
    alice
        .start_breakout_rooms(
            &mut [&mut bob, &mut charlie],
            BreakoutConfig {
                rooms: vec![
                    BreakoutRoomConfig {
                        name: "Room 0".into(),
                        assignments: vec![],
                    },
                    BreakoutRoomConfig {
                        name: "Room 1".into(),
                        assignments: vec![],
                    },
                    BreakoutRoomConfig {
                        name: "Room 2".into(),
                        assignments: vec![],
                    },
                ],
                duration: None,
            },
        )
        .await;

    // Alice switches to breakout room 1
    alice
        .switch_breakout_room(
            &mut [&mut bob, &mut charlie],
            RoomKind::Breakout(BreakoutId::from(1)),
        )
        .await;

    // Bob switches to breakout room 1
    bob.switch_breakout_room(
        &mut [&mut alice, &mut charlie],
        RoomKind::Breakout(BreakoutId::from(1)),
    )
    .await;

    assert!(alice.received_nothing());
    assert!(bob.received_nothing());
    assert!(charlie.received_nothing());

    BreakoutScenario {
        _room: room,
        alice,
        bob,
        charlie,
    }
}

#[tokio::test]
async fn room_chat_history_chunks() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;

    // Chat history is empty
    let chat_state = alice
        .join_success()
        .get_module::<ChatState>()
        .expect("Did not receive chat state")
        .expect("Chat state must not be empty");
    assert!(chat_state.global_history.messages.is_empty());

    let message_count = (2 * CHAT_CHUNK_SIZE) + 1;
    fill_messages(&mut alice, &mut [], Scope::Global, "", message_count).await;

    alice.disconnect().await.unwrap();
    let mut alice = room.join_alice_moderator(1).await;

    let chunk = alice
        .join_success()
        .get_module::<ChatState>()
        .expect("Did not receive chat state")
        .expect("Chat state must not be empty")
        .global_history;

    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(message_count - CHAT_CHUNK_SIZE - 1));

    let chunk = get_chunk(&mut alice, Scope::Global, chunk.next_index.unwrap()).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(0));

    let chunk = get_chunk(&mut alice, Scope::Global, chunk.next_index.unwrap()).await;
    assert_eq!(chunk.messages.len(), 1);
    assert_eq!(chunk.next_index, None);

    // Out of bounds
    let chunk = get_chunk(&mut alice, Scope::Global, CHAT_CHUNK_SIZE * 100).await;
    assert_eq!(chunk, ChatChunk::default());
}

#[tokio::test]
async fn breakout_chat_history_chunks() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;

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

    let event = alice
        .switch_breakout_room(&mut [], RoomKind::Breakout(BreakoutId::from(0)))
        .await;
    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Received wrong breakout event");
    };
    let chunk = own_data
        .get::<ChatState>()
        .expect("Did not receive chat state")
        .expect("Chat state must not be empty")
        .breakout_room_history
        .expect("Breakout history must not be None");
    assert_eq!(chunk, ChatChunk::default());

    let breakout_id = BreakoutId::from(0);
    let scope = Scope::Breakout(breakout_id);
    let message_count = (2 * CHAT_CHUNK_SIZE) + 1;
    fill_messages(&mut alice, &mut [], scope.clone(), "", message_count).await;

    alice.switch_breakout_room(&mut [], RoomKind::Main).await;
    let event = alice
        .switch_breakout_room(&mut [], RoomKind::Breakout(breakout_id))
        .await;
    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Received wrong breakout event");
    };

    let chunk = own_data
        .get::<ChatState>()
        .expect("Did not receive chat state")
        .expect("Chat state must not be empty")
        .breakout_room_history
        .expect("Breakout history must not be None");
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(message_count - CHAT_CHUNK_SIZE - 1));

    let chunk = get_chunk(&mut alice, scope.clone(), chunk.next_index.unwrap()).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(0));

    let chunk = get_chunk(&mut alice, scope.clone(), chunk.next_index.unwrap()).await;
    assert_eq!(chunk.messages.len(), 1);
    assert_eq!(chunk.next_index, None);

    // Out of bounds
    let chunk = get_chunk(&mut alice, scope.clone(), CHAT_CHUNK_SIZE * 100).await;
    assert_eq!(chunk, ChatChunk::default());
}

#[tokio::test]
async fn private_chat_history_chunks() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Private chat history is empty
    let private_history = alice
        .join_success()
        .get_module::<ChatState>()
        .expect("Did not receive chat state")
        .expect("Chat state must not be empty")
        .private_history;
    assert!(private_history.is_empty());

    let scope = Scope::Private(bob.id());

    let message_count = (2 * CHAT_CHUNK_SIZE) + 1;
    fill_messages(
        &mut alice,
        &mut [&mut bob],
        scope.clone(),
        "",
        message_count,
    )
    .await;

    let mut alice = room.join_alice_moderator(2).await;

    let chunk = &alice
        .join_success()
        .get_module::<ChatState>()
        .expect("Did not receive chat state")
        .expect("Chat state must not be empty")
        .private_history[0]
        .history;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(message_count - CHAT_CHUNK_SIZE - 1));

    let scope = Scope::Private(bob.id());
    let chunk = get_chunk(&mut alice, scope.clone(), chunk.next_index.unwrap()).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(0));

    let chunk = get_chunk(&mut alice, scope.clone(), chunk.next_index.unwrap()).await;
    assert_eq!(chunk.messages.len(), 1);
    assert_eq!(chunk.next_index, None);

    // Out of bounds
    let chunk = get_chunk(&mut alice, scope, CHAT_CHUNK_SIZE * 100).await;
    assert_eq!(chunk, ChatChunk::default());
}

#[test_log::test(tokio::test)]
async fn other_breakout_room_history_chunk() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "room 0".into(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    // Bob is not allowed to see the messages of the breakout room when he isn't
    // inside it
    let scope = Scope::Breakout(BreakoutId::from(0));
    bob.send_command::<ChatModule>(
        ChatCommand::GetHistoryChunk {
            message_index: 0,
            scope: scope.clone(),
        },
        None,
    )
    .await
    .unwrap();

    let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(event, ChatEvent::Error(ChatError::InsufficientPermissions));

    // Alice is allowed to search the messages of the breakout room
    alice
        .send_command::<ChatModule>(
            ChatCommand::GetHistoryChunk {
                message_index: 0,
                scope: scope.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ChatEvent::BreakoutChatHistoryChunk(BreakoutHistory {
            history: ChatChunk::default(),
            breakout_id: BreakoutId::from(0),
        })
    );
}

async fn get_chunk(
    participant: &mut MockParticipantJoined,
    scope: Scope,
    message_index: u32,
) -> ChatChunk {
    participant
        .send_command::<ChatModule>(
            ChatCommand::GetHistoryChunk {
                message_index,
                scope: scope.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let event = participant
        .receive_event::<ChatModule>()
        .await
        .unwrap()
        .payload;

    match (scope, event) {
        (Scope::Global, ChatEvent::RoomChatHistoryChunk { history }) => history,
        (
            Scope::Breakout(..),
            ChatEvent::BreakoutChatHistoryChunk(BreakoutHistory { history, .. }),
        ) => history,
        (
            Scope::Private(..),
            ChatEvent::PrivateChatHistoryChunk(PrivateHistory { history, .. }),
        ) => history,
        _ => panic!("Received wrong event"),
    }
}

async fn fill_messages(
    sender: &mut MockParticipantJoined,
    receivers: &mut [&mut MockParticipantJoined],
    scope: Scope,
    content: &str,
    message_count: u32,
) {
    for i in 0..message_count {
        sender
            .send_command::<ChatModule>(
                ChatCommand::SendMessage {
                    content: format!("{i}_{content}"),
                    scope: scope.clone(),
                },
                None,
            )
            .await
            .unwrap();
        let event = sender.receive_event::<ChatModule>().await.unwrap().payload;
        assert_matches!(event, ChatEvent::MessageSent { .. });

        // Messages must be received because otherwise the channel fills up and
        // builds back pressure on the sender of the sending participant.
        for receiver in receivers.iter_mut() {
            receiver.receive_event::<ChatModule>().await.unwrap();
        }
    }
}

#[tokio::test]
async fn private_chat_history_on_join() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;
    let mut bob = room.join_bob(1).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(1).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Alice sends a private message to Bob
    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "hello Bob".into(),
                scope: Scope::Private(bob.id()),
            },
            None,
        )
        .await
        .unwrap();

    alice.receive::<ChatEvent>().await.unwrap();
    bob.receive::<ChatEvent>().await.unwrap();
    assert!(charlie.received_nothing());

    // Bob reconnects
    bob.disconnect().await.unwrap();
    alice.receive::<CoreEvent>().await.unwrap();
    charlie.receive::<CoreEvent>().await.unwrap();

    let bob = room.join_bob(1).await;
    flush_connected_events(&mut [&mut alice, &mut charlie]).await;

    // Bobs JoinSuccess contains the message from alice
    let mut chat_state = bob
        .join_success()
        .module_data
        .get::<ChatState>()
        .expect("Did not receive chat state")
        .expect("Chat state must not be empty");
    for private_history in chat_state.private_history.iter_mut() {
        for message in private_history.history.messages.iter_mut() {
            message.id = MessageId::nil();
            message.timestamp = Timestamp::unix_epoch();
        }
    }
    assert_eq!(
        chat_state.private_history,
        vec![PrivateHistory {
            correspondent: alice.id(),
            history: ChatChunk {
                messages: vec![StoredMessage {
                    id: MessageId::nil(),
                    source: alice.id(),
                    timestamp: Timestamp::unix_epoch(),
                    content: "hello Bob".into(),
                    scope: Scope::Private(bob.id())
                }],
                next_index: None
            }
        }]
    );

    // Charlie reconnects
    charlie.disconnect().await.unwrap();
    alice.receive::<CoreEvent>().await.unwrap();

    let charlie = room.join_charlie(1).await;

    // Charlies JoinSuccess does not contain any private chat messages
    let chat_state = charlie
        .join_success()
        .get_module::<ChatState>()
        .expect("Did not receive chat state")
        .expect("Chat state must not be empty");
    assert_eq!(chat_state.private_history, Vec::new());
}

#[test_log::test(tokio::test)]
async fn breakout_chat_history_on_join() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .start_breakout_rooms(
            &mut [],
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
        .switch_breakout_room(&mut [], RoomKind::Breakout(0.into()))
        .await;

    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "hello breakout room".into(),
                scope: Scope::Breakout(0.into()),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_matches!(event, ChatEvent::MessageSent { .. });

    let alice_id = alice.id();
    alice.disconnect().await.unwrap();

    let breakout_messages = room
        .join_alice_moderator(0)
        .await
        .join_success()
        .module_data
        .get::<ChatState>()
        .expect("ChatState must be valid")
        .expect("ChatState must exist")
        .breakout_room_history
        .unwrap();

    assert_eq!(breakout_messages.messages.len(), 1);
    let msg = &breakout_messages.messages[0];
    assert_eq!(msg.source, alice_id);
    assert_eq!(msg.content, "hello breakout room");
    assert_eq!(msg.scope, Scope::Breakout(0.into()));
}

#[test_log::test(tokio::test)]
async fn invalid_search_term_length() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;

    alice
        .send_command::<ChatModule>(
            ChatCommand::SearchHistory {
                term: "".into(),
                scope: Scope::Global,
                message_index: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_matches!(
        event,
        ChatEvent::Error(ChatError::InvalidSearchTermLength { .. })
    );
}

#[test_log::test(tokio::test)]
async fn search_room_chat_history() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;

    let message_count = (2 * CHAT_CHUNK_SIZE) + 1;
    let search_term = "hello";
    let scope = Scope::Global;
    fill_messages(
        &mut alice,
        &mut [],
        scope.clone(),
        search_term,
        message_count,
    )
    .await;
    fill_messages(&mut alice, &mut [], scope.clone(), "goodbye", message_count).await;

    // No matches
    let chunk = search(&mut alice, scope.clone(), None, "banana").await;
    assert_eq!(chunk, ChatChunk::default());

    let chunk = search(&mut alice, scope.clone(), None, search_term).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(message_count - CHAT_CHUNK_SIZE - 1));
    // Check for the correct order
    assert_eq!(
        chunk
            .messages
            .iter()
            .position(|m| m.content.contains(&(message_count - 1).to_string())),
        Some(CHAT_CHUNK_SIZE as usize - 1)
    );

    let chunk = search(&mut alice, scope.clone(), chunk.next_index, search_term).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(0));
    // Check for the correct order
    assert_eq!(
        chunk.messages.iter().position(|m| m
            .content
            .contains(&(message_count - 1 - CHAT_CHUNK_SIZE).to_string())),
        Some(CHAT_CHUNK_SIZE as usize - 1)
    );

    let chunk = search(&mut alice, scope.clone(), chunk.next_index, search_term).await;
    assert_eq!(chunk.messages.len(), 1);
    assert_eq!(chunk.next_index, None);

    // Out of bounds
    let chunk = search(&mut alice, scope, Some(CHAT_CHUNK_SIZE * 1000), search_term).await;
    assert_eq!(chunk, ChatChunk::default());
}

#[test_log::test(tokio::test)]
async fn search_breakout_chat_history() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;

    alice
        .start_breakout_rooms(
            &mut [],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "breakout_room".into(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    let breakout_id = BreakoutId::from(0);
    alice
        .switch_breakout_room(&mut [], RoomKind::Breakout(breakout_id))
        .await;

    let message_count = (2 * CHAT_CHUNK_SIZE) + 1;
    let search_term = "hello";
    let scope = Scope::Breakout(breakout_id);
    fill_messages(
        &mut alice,
        &mut [],
        scope.clone(),
        search_term,
        message_count,
    )
    .await;
    fill_messages(&mut alice, &mut [], scope.clone(), "goodbye", message_count).await;

    // No matches
    let chunk = search(&mut alice, scope.clone(), None, "banana").await;
    assert_eq!(chunk, ChatChunk::default());

    let chunk = search(&mut alice, scope.clone(), None, search_term).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(message_count - CHAT_CHUNK_SIZE - 1));
    // Check for the correct order
    assert_eq!(
        chunk
            .messages
            .iter()
            .position(|m| m.content.contains(&(message_count - 1).to_string())),
        Some(CHAT_CHUNK_SIZE as usize - 1)
    );

    let chunk = search(&mut alice, scope.clone(), chunk.next_index, search_term).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(0));
    // Check for the correct order
    assert_eq!(
        chunk.messages.iter().position(|m| m
            .content
            .contains(&(message_count - 1 - CHAT_CHUNK_SIZE).to_string())),
        Some(CHAT_CHUNK_SIZE as usize - 1)
    );

    let chunk = search(&mut alice, scope.clone(), chunk.next_index, search_term).await;
    assert_eq!(chunk.messages.len(), 1);
    assert_eq!(chunk.next_index, None);

    // Out of bounds
    let chunk = search(
        &mut alice,
        scope.clone(),
        Some(CHAT_CHUNK_SIZE * 1000),
        search_term,
    )
    .await;
    assert_eq!(chunk, ChatChunk::default());
}

#[test_log::test(tokio::test)]
async fn search_private_chat_history() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let message_count = (2 * CHAT_CHUNK_SIZE) + 1;
    let search_term = "hello";
    let scope = Scope::Private(bob.id());
    fill_messages(
        &mut alice,
        &mut [&mut bob],
        scope.clone(),
        search_term,
        message_count,
    )
    .await;
    fill_messages(
        &mut alice,
        &mut [&mut bob],
        scope.clone(),
        "goodbye",
        message_count,
    )
    .await;

    // No matches
    let chunk = search(&mut alice, scope.clone(), None, "banana").await;
    assert_eq!(chunk, ChatChunk::default());

    let chunk = search(&mut alice, scope.clone(), None, search_term).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(message_count - CHAT_CHUNK_SIZE - 1));
    // Check for the correct order
    assert_eq!(
        chunk
            .messages
            .iter()
            .position(|m| m.content.contains(&(message_count - 1).to_string())),
        Some(CHAT_CHUNK_SIZE as usize - 1)
    );

    let chunk = search(&mut alice, scope.clone(), chunk.next_index, search_term).await;
    assert_eq!(chunk.messages.len() as u32, CHAT_CHUNK_SIZE);
    assert_eq!(chunk.next_index, Some(0));
    // Check for the correct order
    assert_eq!(
        chunk.messages.iter().position(|m| m
            .content
            .contains(&(message_count - 1 - CHAT_CHUNK_SIZE).to_string())),
        Some(CHAT_CHUNK_SIZE as usize - 1)
    );

    let chunk = search(&mut alice, scope.clone(), chunk.next_index, search_term).await;
    assert_eq!(chunk.messages.len(), 1);
    assert_eq!(chunk.next_index, None);

    // Out of bounds
    let chunk = search(
        &mut alice,
        scope.clone(),
        Some(CHAT_CHUNK_SIZE * 1000),
        search_term,
    )
    .await;
    assert_eq!(chunk, ChatChunk::default());
}

async fn search(
    participant: &mut MockParticipantJoined,
    search_scope: Scope,
    message_index: Option<u32>,
    term: &str,
) -> ChatChunk {
    participant
        .send_command::<ChatModule>(
            ChatCommand::SearchHistory {
                term: term.into(),
                scope: search_scope.clone(),
                message_index,
            },
            None,
        )
        .await
        .unwrap();

    let event = participant
        .receive_event::<ChatModule>()
        .await
        .unwrap()
        .payload;

    match event {
        ChatEvent::SearchResults { matches, scope } => {
            assert_eq!(scope, search_scope);
            for message in &matches.messages {
                message.content.contains(term);
            }
            matches
        }
        _ => panic!("Received wrong event"),
    }
}

#[test_log::test(tokio::test)]
async fn search_other_breakout_room() {
    let mut room = TestRoom::builder().register_module::<ChatModule>().spawn();
    let mut alice = room.join_alice_moderator(1).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "room 0".into(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    // Bob is not allowed to see the messages of the breakout room when he isn't
    // inside it
    let scope = Scope::Breakout(BreakoutId::from(0));
    bob.send_command::<ChatModule>(
        ChatCommand::SearchHistory {
            scope: scope.clone(),
            term: "hello".into(),
            message_index: None,
        },
        None,
    )
    .await
    .unwrap();

    let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(event, ChatEvent::Error(ChatError::InsufficientPermissions));

    // Alice is allowed to search the messages of the breakout room
    alice
        .send_command::<ChatModule>(
            ChatCommand::SearchHistory {
                scope: scope.clone(),
                term: "hello".into(),
                message_index: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ChatEvent::SearchResults {
            matches: ChatChunk::default(),
            scope
        }
    );
}

#[test_log::test(tokio::test)]
async fn rate_limit_slow_down() {
    let mut room = TestRoom::builder()
        .add_init_module_data(&ChatSettings {
            rate_limit: Some(RateLimitSettings {
                tokens_per_second: NonZero::new(1).unwrap(),
                // 11 messages in 1 second will trigger the too many requests error
                token_bucket_size: NonZero::new(10).unwrap(),
                // 6 messages in 1 second will trigger the slow down event
                slow_down_threshold: 0.5,
            }),
        })
        .unwrap()
        .register_module::<ChatModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    for _ in 0..5 {
        bob.send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "hello".into(),
                scope: Scope::Global,
            },
            None,
        )
        .await
        .unwrap();
        let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
        assert_matches!(event, ChatEvent::MessageSent { .. });
    }

    // The 6th message triggers the slow down event
    bob.send_command::<ChatModule>(
        ChatCommand::SendMessage {
            content: "hello".into(),
            scope: Scope::Global,
        },
        None,
    )
    .await
    .unwrap();
    let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
    assert_matches!(event, ChatEvent::SlowDown { .. });

    // The message is still sent
    let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
    assert_matches!(event, ChatEvent::MessageSent { .. });

    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn rate_limit_reached() {
    let mut room = TestRoom::builder()
        .add_init_module_data(&ChatSettings {
            rate_limit: Some(RateLimitSettings {
                tokens_per_second: NonZero::new(1).unwrap(),
                // 11 messages in 1 second will trigger the too many requests error
                token_bucket_size: NonZero::new(10).unwrap(),
                // 6 messages in 1 second will trigger the slow down event
                slow_down_threshold: 0.5,
            }),
        })
        .unwrap()
        .register_module::<ChatModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    for _ in 0..5 {
        bob.send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "hello".into(),
                scope: Scope::Global,
            },
            None,
        )
        .await
        .unwrap();
        let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
        assert_message_eq!(Scope::Global, "hello", bob.id(), &event);
    }

    // Starting with the 5th message the slow down event is triggered
    for _ in 0..5 {
        bob.send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "hello".into(),
                scope: Scope::Global,
            },
            None,
        )
        .await
        .unwrap();
        let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
        assert_eq!(
            event,
            ChatEvent::SlowDown {
                recommended_wait_ms: 1000
            }
        );

        let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
        assert_message_eq!(Scope::Global, "hello", bob.id(), &event);
    }

    // The 11th message triggers the too many requests error
    bob.send_command::<ChatModule>(
        ChatCommand::SendMessage {
            content: "hello".into(),
            scope: Scope::Global,
        },
        None,
    )
    .await
    .unwrap();
    let event = bob.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ChatEvent::Error(ChatError::TooManyRequests {
            retry_after_ms: 5000
        })
    );

    // The message is not sent
    assert!(bob.received_nothing());

    // Alice receives 10 chat messages from bob
    for _ in 0..10 {
        let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
        assert_message_eq!(Scope::Global, "hello", bob.id(), &event);
    }

    // Alice receives the rate limited event for bob
    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ChatEvent::ParticipantRateLimited {
            participant_id: bob.id()
        }
    );
}

#[test_log::test(tokio::test)]
async fn moderators_dont_receive_own_participant_rate_limit_reached() {
    let mut room = TestRoom::builder()
        .register_module::<ChatModule>()
        .add_init_module_data(&ChatSettings {
            rate_limit: Some(RateLimitSettings {
                tokens_per_second: NonZero::new(1).unwrap(),
                token_bucket_size: NonZero::new(1).unwrap(),
                slow_down_threshold: 1.0,
            }),
        })
        .unwrap()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "hello".to_string(),
                scope: Scope::Global,
            },
            None,
        )
        .await
        .unwrap();
    // The first message is sent successfully
    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_message_eq!(Scope::Global, "hello", alice.id(), &event);
    // Alice is asked to slow down

    alice
        .send_command::<ChatModule>(
            ChatCommand::SendMessage {
                content: "hello".to_string(),
                scope: Scope::Global,
            },
            None,
        )
        .await
        .unwrap();
    // The second message triggers the rate limit
    let event = alice.receive_event::<ChatModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ChatEvent::Error(ChatError::TooManyRequests {
            retry_after_ms: 1000
        })
    );

    // Alice does not receive the participant rate limited event for herself
    assert!(alice.received_nothing());
}
