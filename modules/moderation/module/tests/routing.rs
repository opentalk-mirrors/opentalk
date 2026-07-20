// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::assert_matches;

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::{
    mock_module::{MockCommand, MockEvent, MockModule},
    room::TestRoom,
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        command::BreakoutCommand,
        event::BreakoutEvent,
    },
    core::{CoreCommand, CoreEvent},
    room_parameters::WaitingRoom,
};
use opentalk_roomserver_types_moderation::{command::ModerationCommand, event::ModerationEvent};

#[test_log::test(tokio::test)]
async fn waiting_participants_dont_receive_messages() {
    let mut room = TestRoom::builder()
        .register_module::<MockModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.waiting_room_bob(0).await;

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. });

    // Chat message is used as an arbitrary command that should not be sent to
    // waiting participants
    alice
        .send_command::<MockModule>(MockCommand::Valid, None)
        .await
        .unwrap();
    let event = alice.receive_event::<MockModule>().await.unwrap();
    assert_matches!(event.payload, MockEvent::Success);

    // Bob should not receive the event
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn waiting_participants_dont_receive_broadcasts() {
    let mut room = TestRoom::builder()
        .register_module::<MockModule>()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.waiting_room_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. });

    let mut charlie = room.waiting_room_charlie(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. });

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Accept {
                target: charlie.id(),
            },
            None,
        )
        .await
        .unwrap();
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::ParticipantAccepted {
            participant_id: charlie.id()
        }
    );

    let event = charlie
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Accepted);

    charlie
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let mut charlie = charlie.join_success().await.unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::LeftWaitingRoom { .. });

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::ParticipantConnected { .. });

    assert!(bob.received_nothing());

    // Alice sends a command that triggers a broadcast
    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: Vec::from_iter([BreakoutRoomConfig {
                    name: "Room 1".into(),
                    assignments: Vec::new(),
                }]),
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<BreakoutEvent>().await.unwrap();
    assert_matches!(event.payload, BreakoutEvent::Started { .. });

    let event = charlie.receive::<BreakoutEvent>().await.unwrap();
    assert_matches!(event.payload, BreakoutEvent::Started { .. });

    assert!(bob.received_nothing());

    charlie.disconnect().await.unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::ParticipantDisconnected { .. });

    assert!(bob.received_nothing());
}
