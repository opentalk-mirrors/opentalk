// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::assert_matches;

use opentalk_roomserver_module_timer::TimerModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        command::BreakoutCommand,
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};
use opentalk_roomserver_types_timer::{
    StopKind, TIMER_MODULE_ID, TimerCommand, TimerConfig, TimerError, TimerEvent, command::Kind,
    event::Stopped, peer_state::TimerPeerState, state::TimerState,
};

async fn start_timer(
    user: &mut MockParticipantJoined,
    kind: Kind,
    style: Option<String>,
    title: Option<String>,
    enable_ready_check: bool,
) {
    user.send_command::<TimerModule>(
        TimerCommand::Start {
            kind,
            style,
            title,
            enable_ready_check,
        },
        None,
    )
    .await
    .unwrap();

    assert_matches!(
        user.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Started { .. }
    );
}

/// if ready state is enabled, it should be part of the join success information
#[test_log::test(tokio::test)]
async fn ready_state_is_part_of_join_success() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    start_timer(&mut alice, Kind::Stopwatch, None, None, true).await;

    let charlie = room.join_charlie(0).await;
    let join_success = charlie.join_success();
    let state = join_success
        .module_data
        .get::<TimerState>()
        .expect("deserialization must work")
        .expect("state must be set");
    assert_matches!(
        state,
        TimerState {
            config: TimerConfig {
                kind: opentalk_roomserver_types_timer::Kind::Stopwatch,
                ready_check_enabled: true,
                ..
            },
            ready_status: Some(false),
            ..
        }
    );

    let bob = room.join_bob(0).await;
    let join_success = bob.join_success();
    let state = join_success
        .module_data
        .get::<TimerState>()
        .expect("deserialization must work")
        .expect("state must be set");
    assert_matches!(
        state,
        TimerState {
            config: TimerConfig {
                kind: opentalk_roomserver_types_timer::Kind::Stopwatch,
                ready_check_enabled: true,
                ..
            },
            ready_status: Some(false),
            ..
        }
    );

    let charlie_state_for_bob = join_success
        .participants
        .iter()
        .find(|p| p.id == charlie.id())
        .unwrap()
        .get_module::<TimerPeerState>()
        .expect("deserialization must work")
        .expect("state must be set");
    assert_eq!(
        charlie_state_for_bob,
        TimerPeerState {
            ready_status: false,
        }
    );
}

#[test_log::test(tokio::test)]
async fn can_not_start_second_timer() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    start_timer(&mut alice, Kind::Stopwatch, None, None, false).await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Start {
                kind: Kind::Stopwatch,
                style: None,
                title: None,
                enable_ready_check: false,
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Error(TimerError::TimerAlreadyRunning)
    );
}

#[test_log::test(tokio::test)]
async fn non_moderator_cant_start_timer() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<TimerModule>(
        TimerCommand::Start {
            kind: Kind::Countdown { duration: 0 },
            style: None,
            title: None,
            enable_ready_check: false,
        },
        None,
    )
    .await
    .unwrap();

    assert_eq!(
        bob.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Error(TimerError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn all_participants_receive_timer_events() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    start_timer(&mut alice, Kind::Stopwatch, None, None, false).await;

    assert_matches!(
        bob.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Started { .. }
    );

    alice
        .send_command::<TimerModule>(TimerCommand::Stop { reason: None }, None)
        .await
        .unwrap();

    assert_matches!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Stopped(..)
    );

    assert_matches!(
        bob.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Stopped(..)
    );
}

#[test_log::test(tokio::test)]
async fn exceed_timer() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    start_timer(
        &mut alice,
        Kind::Countdown { duration: 0 },
        None,
        None,
        false,
    )
    .await;

    assert_matches!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Stopped(Stopped {
            kind: StopKind::Expired,
            ..
        })
    );
}

#[test_log::test(tokio::test)]
async fn stop_timer() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    start_timer(&mut alice, Kind::Stopwatch, None, None, false).await;

    alice
        .send_command::<TimerModule>(
            TimerCommand::Stop {
                reason: Some("test".into()),
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Stopped(Stopped {
            kind: StopKind::ByModerator(alice.id()),
            reason: Some("test".into())
        })
    );
}

#[test_log::test(tokio::test)]
async fn can_not_update_ready_status_when_timer_not_running() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Error(TimerError::TimerNotRunning),
    );
}

#[test_log::test(tokio::test)]
async fn can_not_update_ready_status_when_disabled() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    start_timer(&mut alice, Kind::Stopwatch, None, None, false).await;

    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Error(TimerError::ReadyCheckNotEnabled),
    );
}

#[test_log::test(tokio::test)]
async fn update_ready_status() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    start_timer(&mut alice, Kind::Stopwatch, None, None, true).await;

    assert_matches!(
        bob.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Started { .. }
    );

    bob.send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::UpdatedReadyStatus {
            participant_id: bob.id(),
            status: true,
        },
    );

    assert_eq!(
        bob.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::UpdatedReadyStatus {
            participant_id: bob.id(),
            status: true,
        },
    );
}

#[test_log::test(tokio::test)]
async fn ready_state_persists() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    start_timer(&mut alice, Kind::Stopwatch, None, None, true).await;

    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::UpdatedReadyStatus {
            participant_id: alice.id(),
            status: true,
        },
    );

    alice.disconnect().await.unwrap();

    let alice = room.join_alice_moderator(0).await;

    let timer_state = alice
        .join_success()
        .module_data
        .get::<TimerState>()
        .unwrap()
        .unwrap();

    assert!(timer_state.ready_status.unwrap());
}

#[test_log::test(tokio::test)]
async fn breakout_room_scope() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts breakout rooms with separate rooms for Alice and Bob
    let alice_room = BreakoutRoomConfig {
        name: "alice_room".into(),
        assignments: vec![alice.id()],
    };
    let bob_room = BreakoutRoomConfig {
        name: "bob_room".into(),
        assignments: vec![bob.id()],
    };
    alice
        .send_breakout_command(
            BreakoutCommand::Start(BreakoutConfig {
                rooms: vec![alice_room, bob_room],
                duration: None,
            }),
            None,
        )
        .await
        .unwrap();

    // Alice and Bob receive the BreakoutStarted event
    alice.receive::<BreakoutEvent>().await.unwrap();
    bob.receive::<BreakoutEvent>().await.unwrap();

    // Alice switches to room 0
    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    // Alice starts a timer in her room
    start_timer(
        &mut alice,
        Kind::Countdown { duration: 0 },
        None,
        None,
        false,
    )
    .await;

    assert_matches!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::Stopped(Stopped {
            kind: StopKind::Expired,
            ..
        })
    );

    // Bob doesn't
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn breakout_room_ready_state() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts breakout rooms with separate rooms for Alice and Bob
    let alice_room = BreakoutRoomConfig {
        name: "alice_room".into(),
        assignments: vec![alice.id()],
    };
    let bob_room = BreakoutRoomConfig {
        name: "bob_room".into(),
        assignments: vec![bob.id()],
    };
    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![alice_room, bob_room],
                duration: None,
            },
        )
        .await;

    // Alice switches to room 0
    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    // Alice starts a timer in room 0
    start_timer(&mut alice, Kind::Stopwatch, None, None, true).await;

    // Alice updates her ready state in room 0
    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();
    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::UpdatedReadyStatus {
            participant_id: alice.id(),
            status: true,
        },
    );

    // Alice switches to room 1
    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(BreakoutId::from(1)))
        .await;

    // Alice switches back to room 0
    let event = alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    // Alice is still ready in room 0
    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        unreachable!("Received wrong event");
    };

    let state = own_data.get::<TimerState>().unwrap();
    assert_matches!(
        state,
        Some(TimerState {
            ready_status: Some(true),
            ..
        })
    );
}

/// Ensure the [`BreakoutEvent::SwitchedRoom`] event contains the ready state.
///
/// 1. Alice and Bob join
/// 2. Alice creates two breakout rooms
/// 3. Alice and Bob join separate breakout rooms
/// 4. Alice starts the timer and sets a ready state
/// 5. Bob joins Alice's breakout room and must receive Alice's ready state
#[test_log::test(tokio::test)]
async fn breakout_room_switch_contains_peer_ready_state() {
    let mut room = TestRoom::builder().register_module::<TimerModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let alice_room = BreakoutRoomConfig {
        name: "alice_room".into(),
        assignments: vec![alice.id()],
    };
    let bob_room = BreakoutRoomConfig {
        name: "bob_room".into(),
        assignments: vec![bob.id()],
    };
    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![alice_room, bob_room],
                duration: None,
            },
        )
        .await;

    let alice_room = RoomKind::Breakout(BreakoutId::from(0));
    let bob_room = RoomKind::Breakout(BreakoutId::from(1));

    alice
        .switch_breakout_room(&mut [&mut bob], alice_room)
        .await;
    bob.switch_breakout_room(&mut [&mut alice], bob_room).await;

    start_timer(&mut alice, Kind::Stopwatch, None, None, true).await;

    alice
        .send_command::<TimerModule>(TimerCommand::UpdateReadyStatus { status: true }, None)
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<TimerModule>().await.unwrap().payload,
        TimerEvent::UpdatedReadyStatus {
            participant_id: alice.id(),
            status: true,
        },
    );

    let event = bob
        .switch_breakout_room(&mut [&mut alice], alice_room)
        .await;

    let BreakoutEvent::SwitchedRoom { peer_data, .. } = event else {
        unreachable!("Received wrong event");
    };

    let alice_timer_data = peer_data
        .get(&alice.id())
        .expect("Alice must be part of the peer data")
        .get(&TIMER_MODULE_ID)
        .expect("Timer peer data must be present");
    let bob_state = alice_timer_data.clone_inner();

    assert_eq!(
        bob_state
            .get("ready_status")
            .and_then(|ready_status| ready_status.as_bool()),
        Some(true),
    );
}
