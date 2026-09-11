// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Regression tests ensuring that a disconnected participant that is still
//! part of the room does not stop timer broadcasts from reaching the remaining
//! connected participants.

use std::assert_matches;

use opentalk_roomserver_module_timer::TimerModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events, flush_disconnected_events},
};
use opentalk_roomserver_types_timer::{TimerCommand, TimerEvent, command};

/// Builds a room in which `alice` and `bob` stay connected while `charlie`
/// disconnects but remains part of the room. `alice` is a moderator and is used
/// to send timer commands.
///
/// Returns the room together with the two connected participants, with `alice`
/// as the first element.
async fn setup_room_with_disconnected_participant()
-> (TestRoom, MockParticipantJoined, MockParticipantJoined) {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();

    // Connected participants. `alice` is the moderator/sender.
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Participant that disconnects but remains in the room. Its outgoing queue
    // must be drained while connected, otherwise the bounded room channels block
    // subsequent broadcasts.
    let charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // The disconnected state stays in the room, so timer broadcasts targeting
    // the whole room still iterate over it.
    charlie.disconnect().await.unwrap();
    flush_disconnected_events(&mut [&mut alice, &mut bob]).await;

    (room, alice, bob)
}

/// Stopping a timer must notify every connected participant even when the room
/// still contains disconnected participants.
#[test_log::test(tokio::test)]
async fn timer_reaches_all_connected_participants_despite_disconnected() {
    let (_room, mut alice, mut bob) = setup_room_with_disconnected_participant().await;

    // Start a stopwatch. Stopping it broadcasts to every participant in the
    // room, including the disconnected ones.
    alice
        .send_command::<TimerModule>(
            TimerCommand::Start {
                kind: command::Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            },
            None,
        )
        .await
        .unwrap();

    for participant in [&mut alice, &mut bob] {
        let event = participant.receive_event::<TimerModule>().await.unwrap();
        let display_name = participant.display_name();
        assert_matches!(
            event.payload,
            TimerEvent::Started { .. },
            "`{display_name}` did not receive TimerEvent::Started"
        );
    }

    alice
        .send_command::<TimerModule>(TimerCommand::Stop { reason: None }, None)
        .await
        .unwrap();

    for participant in [&mut alice, &mut bob] {
        let event = participant.receive_event::<TimerModule>().await.unwrap();
        let display_name = participant.display_name();
        assert_matches!(
            event.payload,
            TimerEvent::Stopped(_),
            "`{display_name}` did not receive TimerEvent::Stopped"
        );
    }
}
