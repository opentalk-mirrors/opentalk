// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    assert_matches,
    collections::{BTreeSet, HashSet},
};

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::{
    participant::{
        MockParticipantJoined, MockParticipantJoining, MockParticipantWaiting,
        bob_public_user_profile, charlie_public_user_profile,
    },
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    core::{CoreCommand, CoreError, CoreEvent, LeftWaitingRoom},
    disconnect_reason::DisconnectReason,
    room_parameters::WaitingRoom,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
    state::{ModerationState, WaitingParticipantPeerData},
};
use opentalk_types_signaling::{ParticipantId, Role};

#[test_log::test(tokio::test)]
async fn join_info() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let bob_0 = room.waiting_room_bob(0).await;
    let bob_1 = room.waiting_room_bob(1).await;
    let alice = room.join_alice_moderator(0).await;

    let moderator_data = alice
        .join_success()
        .module_data
        .get::<ModerationState>()
        .expect("Module data must be present")
        .expect("Module data must not be none")
        .moderator_data
        .expect("Moderator data must be present");
    assert_eq!(moderator_data.waiting_room, WaitingRoom::ForEveryone);

    let waiting_bob = &moderator_data.waiting_room_participants[0];
    assert_matches!(
       waiting_bob,
       WaitingParticipantPeerData { participant_id, accepted, .. }
            if *participant_id == bob_0.id() && !accepted
    );

    assert_eq!(
        waiting_bob
            .connections
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from_iter([bob_0.connection_id(), bob_1.connection_id()])
    );
}

async fn accept_participant(
    moderator: &mut MockParticipantJoined,
    joinee: MockParticipantWaiting,
) -> MockParticipantJoined {
    moderator
        .send_command::<ModerationModule>(
            ModerationCommand::Accept {
                target: joinee.id(),
            },
            None,
        )
        .await
        .unwrap();
    let event = moderator
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::ParticipantAccepted {
            participant_id: joinee.id()
        }
    );

    let mut joinee = joinee.enter_room(None).await.unwrap();

    let event = moderator.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::LeftWaitingRoom(
            LeftWaitingRoom { id, connection_ids }
        ) if joinee.id() == id && HashSet::from_iter([joinee.connection_id()]) == connection_ids
    );
    let event = moderator.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::ParticipantConnected {
            participant_id,
            connection_id,
            ..
            } if participant_id == joinee.id() && connection_id == joinee.connection_id()
    );
    assert!(moderator.received_nothing());
    assert!(joinee.received_nothing());
    joinee
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Charlie joins via the waiting room (Test participant to verify events are send out correctly)
/// 4. Bob joins twice and is put in the waiting room (multiple connections)
/// 5. Alice accepts bob
/// 6. Both of Bobs devices join the room
/// 7. Moderators receive LeftWaitingRoom event; Normal users receive Joined event
#[test_log::test(tokio::test)]
async fn join_via_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let charlie = room.waiting_room_charlie(0).await;

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::JoinedWaitingRoom { participant_id, display_name, .. }
            if participant_id == charlie.id()
                && display_name == charlie_public_user_profile().user_info.display_name
    );
    let mut charlie = accept_participant(&mut alice, charlie).await;

    let mut bob_0 = room.waiting_room_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    let CoreEvent::JoinedWaitingRoom {
        participant_id,
        connection_ids,
        display_name,
        ..
    } = event.payload
    else {
        panic!("Expected JoinedWaitingRoom event, got: {:?}", event.payload);
    };
    assert_eq!(participant_id, bob_0.id());
    assert_eq!(connection_ids, vec![bob_0.connection_id()]);
    assert_eq!(
        display_name,
        bob_public_user_profile().user_info.display_name
    );

    let mut bob_1 = room.waiting_room_bob(1).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    let CoreEvent::JoinedWaitingRoom {
        participant_id,
        connection_ids,
        display_name,
        ..
    } = event.payload
    else {
        panic!("Expected JoinedWaitingRoom event, got: {:?}", event.payload);
    };

    assert_eq!(participant_id, bob_0.id());
    assert_eq!(
        connection_ids.into_iter().collect::<BTreeSet<_>>(),
        BTreeSet::from_iter([bob_0.connection_id(), bob_1.connection_id()])
    );
    assert_eq!(
        display_name,
        bob_public_user_profile().user_info.display_name
    );

    assert!(bob_0.received_nothing());
    assert!(bob_1.received_nothing());
    alice
        .send_command::<ModerationModule>(ModerationCommand::Accept { target: bob_0.id() }, None)
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
            participant_id: bob_0.id()
        }
    );

    let event = bob_0
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Accepted);

    let event = bob_1
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::Accepted);

    bob_0
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let bob_0 = bob_0.join_success().await.unwrap();
    let bob_1 = bob_1.join_success().await.unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::LeftWaitingRoom(LeftWaitingRoom { id, connection_ids })
            if bob_0.id() == id
                && BTreeSet::from_iter([&bob_0.connection_id(), &bob_1.connection_id()])
                    == BTreeSet::from_iter(connection_ids.iter())
    );
    // charlie should only receive the JoinedEvent, which will be checked next.

    async fn receive_joined_events(
        participant: &mut MockParticipantJoined,
        expected: BTreeSet<(ParticipantId, ConnectionId)>,
    ) {
        let mut joined_connections = BTreeSet::new();
        for _ in 0..2 {
            let event = participant.receive::<CoreEvent>().await.unwrap();
            match event.payload {
                CoreEvent::ParticipantConnected {
                    participant_id,
                    connection_id,
                    ..
                } => {
                    joined_connections.insert((participant_id, connection_id));
                }
                other => {
                    panic!("Unexpected CoreEvent: {other:?}");
                }
            }
        }
        assert_eq!(
            joined_connections,
            expected,
            "Participant {} didn't receive all joined events",
            participant.display_name()
        );
    }
    for p in [&mut alice, &mut charlie] {
        receive_joined_events(
            p,
            BTreeSet::from([
                (bob_0.id(), bob_0.connection_id()),
                (bob_1.id(), bob_1.connection_id()),
            ]),
        )
        .await;
    }

    // No additional messages, bob_0 or bob_1 might receive a ParticipantJoined event depending on
    // the order of joining
    assert!(alice.received_nothing());
    assert!(charlie.received_nothing());
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Alice accepts bob
#[test_log::test(tokio::test)]
async fn accept_unknown_participant() {
    let mut room = TestRoom::builder()
        .waiting_room(WaitingRoom::ForEveryone)
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Accept {
                target: ParticipantId::from_u128(12),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive::<ModerationEvent>().await.unwrap();
    assert!(
        matches!(
            event.payload,
            ModerationEvent::Error(ModerationError::NotWaiting)
        ),
        "Expected moderation error, got: {:?}",
        event.payload
    );
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
#[test_log::test(tokio::test)]
async fn moderators_skip_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    room.join_alice_moderator(0).await;
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Bob joins and is accepted
/// 4. Bob leaves
/// 5. When Bob joins again, he skips the waiting room
#[test_log::test(tokio::test)]
async fn registered_users_once_accepted_always_skip() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let bob = room.waiting_room_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::JoinedWaitingRoom { participant_id, .. } if participant_id == bob.id()
    );
    let bob = accept_participant(&mut alice, bob).await;

    bob.disconnect().await.unwrap();
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::ParticipantDisconnected { .. });
    room.join_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::ParticipantConnected { .. });
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Gustav joins and is accepted
/// 4. Gustav leaves
/// 5. When Gustav joins again, he skips the waiting room
#[test_log::test(tokio::test)]
async fn guest_users_once_accepted_always_skip() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForGuests)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let gustav = room.waiting_room_gustav_guest().await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::JoinedWaitingRoom { participant_id, .. } if participant_id == gustav.id()
    );
    let gustav = accept_participant(&mut alice, gustav).await;

    gustav.disconnect().await.unwrap();
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::ParticipantDisconnected { .. });
    room.join_gustav_guest().await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::ParticipantConnected { .. });
}

/// Registered users that are not invited have the guest role and must wait in a waiting room that
/// is enabled for guests.
///
/// 1. Spawn room with a waiting room for guests
/// 2. Alice joins as a moderator
/// 3. Bob joins as a registered user with the guest role and has to enter the waiting room
/// 4. Alice accepts Bob, who enters the room as a guest
#[test_log::test(tokio::test)]
async fn registered_guests_wait_in_waiting_room_for_guests() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForGuests)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let bob = MockParticipantJoining::bob(0)
        .guest()
        .enter_waiting_room(&mut room)
        .await
        .unwrap();
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(
        event.payload,
        CoreEvent::JoinedWaitingRoom { participant_id, .. } if participant_id == bob.id()
    );

    let bob = accept_participant(&mut alice, bob).await;
    assert_eq!(bob.join_success().role, Role::Guest);
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins the room as a moderator
/// 3. Bob enters the waiting room (Alice receives notification)
/// 4. Bob leaves the waiting room (Alice receives notification)
#[test_log::test(tokio::test)]
async fn event_when_leaving_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let bob = room.waiting_room_bob(0).await;
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. });

    bob.disconnect().await.unwrap();
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::LeftWaitingRoom(..));
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Bob enters the waiting room (Alice is notified)
/// 4. Bob sends `EnterWaitingRoom` again
/// 5. Bob is told again that he is in the waiting room, but the moderators are not notified twice
#[test_log::test(tokio::test)]
async fn entering_waiting_room_twice_is_idempotent() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.waiting_room_bob(0).await;

    // Alice is notified that Bob entered the waiting room.
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::JoinedWaitingRoom { participant_id, connection_ids, .. }
            if participant_id == bob.id()
                && connection_ids == vec![bob.connection_id()]
    );

    // Bob enters the waiting room a second time.
    bob.send_core_command(CoreCommand::EnterWaitingRoom { display_name: None }, None)
        .await
        .unwrap();

    // Bob is told again that he is in the waiting room ...
    let event = bob.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::InWaitingRoom);

    // ... but the moderators are not notified a second time.
    assert!(alice.received_nothing());
}

/// 1. Alice joins the conference as a moderator
/// 2. Alice sends `EnterWaitingRoom` and receives an `AlreadyInRoom` error
#[test_log::test(tokio::test)]
async fn entering_waiting_room_while_in_conference_fails() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    // Alice can skip the waiting room because she's a moderator.
    let mut alice = room.join_alice_moderator(0).await;

    // Alice is already in the conference and cannot move into the waiting room.
    alice
        .send_core_command(CoreCommand::EnterWaitingRoom { display_name: None }, None)
        .await
        .unwrap();

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::Error(CoreError::AlreadyInRoom));
}

/// 1. Spawn room with activated waiting room
/// 2. Alice connects two devices, both land in the lobby and may skip the waiting room
/// 3. Device 0 enters the conference; device 1 is not dragged along and stays in the lobby
/// 4. Device 1 enters the conference on its own and device 0 is notified about it
#[test_log::test(tokio::test)]
async fn lobby_devices_enter_conference_independently() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();

    // As a moderator Alice may skip the waiting room, so both of her devices land in the lobby
    // with `can_enter == true` instead of being placed into the waiting room.
    let mut alice_0 = room.alice_moderator_lobby(0).await;
    let mut alice_1 = room.alice_moderator_lobby(1).await;
    assert!(alice_0.can_enter());
    assert!(alice_1.can_enter());

    // Device 0 enters the conference.
    alice_0
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = alice_0.receive::<CoreEvent>().await.unwrap().payload;
    let CoreEvent::JoinSuccess(join_success) = event else {
        panic!("Expected JoinSuccess, got {event:#?}");
    };

    // Alice receives the DisplayNameAssigned event on device 1 because her first device joined the
    // room
    let event = alice_1
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameAssigned {
            new_name: Some(join_success.display_name),
        }
    );

    // Device 1 is not dragged into the conference; it stays in the lobby.
    assert!(alice_1.received_nothing());

    // Device 1 can then enter the conference independently.
    alice_1
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = alice_1.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::JoinSuccess(..));

    // The already-connected device 0 is notified about the new connection.
    let event = alice_0.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantConnected { connection_id, .. } if connection_id == alice_1.connection_id()
    );
}

/// 1. Spawn room with activated waiting room
/// 2. Bob lands in the lobby and is not allowed to skip the waiting room
/// 3. Bob sends `EnterRoom` without entering the waiting room and being accepted
/// 4. Bob receives a `NotAccepted` error
#[test_log::test(tokio::test)]
async fn entering_conference_from_lobby_without_acceptance_fails() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();

    let mut bob = MockParticipantJoining::bob(0)
        .enter_lobby(&mut room)
        .await
        .unwrap();
    assert!(!bob.can_enter());

    // Trying to enter the conference directly, without being accepted, is rejected.
    bob.send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();

    let event = bob.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::Error(CoreError::NotAccepted));
}

/// 1. Spawn room with activated waiting room
/// 2. Alice joins as a moderator
/// 3. Bob connects two devices, both land in the lobby and cannot skip the waiting room
/// 4. Only device 0 enters the waiting room; device 1 stays in the lobby
/// 5. Alice accepts Bob
/// 6. Bob sends `EnterRoom` from the device that is in the waiting room
/// 7. Only that device moves into the conference; device 1 stays in the lobby and can enter later
#[test_log::test(tokio::test)]
async fn entering_conference_from_waiting_room_leaves_lobby_devices_behind() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Bob connects two devices. As a regular user with the waiting room enabled, neither device
    // may skip it.
    let mut bob_0 = MockParticipantJoining::bob(0)
        .enter_lobby(&mut room)
        .await
        .unwrap();
    let mut bob_1 = MockParticipantJoining::bob(1)
        .enter_lobby(&mut room)
        .await
        .unwrap();
    assert!(!bob_0.can_enter());
    assert!(!bob_1.can_enter());

    // Only device 0 enters the waiting room.
    bob_0
        .send_core_command(CoreCommand::EnterWaitingRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = bob_0.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::InWaitingRoom);

    // The moderator is notified about the single waiting connection; device 1 stays in the lobby.
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::JoinedWaitingRoom {
            participant_id,
            connection_ids,
            ..
        } if participant_id == bob_0.id() && connection_ids == vec![bob_0.connection_id()]
    );

    // Bob's other device is still in the lobby and does not get notified.
    assert!(bob_1.received_nothing());

    // Alice accepts Bob.
    alice
        .send_command::<ModerationModule>(ModerationCommand::Accept { target: bob_0.id() }, None)
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
            participant_id: bob_0.id()
        }
    );

    // Both devices are told that Bob was accepted.
    assert_eq!(
        bob_0
            .receive_event::<ModerationModule>()
            .await
            .unwrap()
            .payload,
        ModerationEvent::Accepted
    );
    assert_eq!(
        bob_1
            .receive_event::<ModerationModule>()
            .await
            .unwrap()
            .payload,
        ModerationEvent::Accepted
    );

    // Bob enters the conference from the device that is in the waiting room.
    bob_0
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = bob_0.receive::<CoreEvent>().await.unwrap().payload;
    let CoreEvent::JoinSuccess(join_success) = event else {
        panic!("Expected JoinSuccess, got {event:#?}");
    };

    // Bob receives the DisplayNameAssigned event on device 1 because her first device joined the
    // room
    let event = bob_1
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameAssigned {
            new_name: Some(join_success.display_name),
        }
    );

    // Device 1 is not dragged along; it stays in the lobby.
    assert!(bob_1.received_nothing());

    // The moderator sees only the waiting-room connection leave the waiting room and join.
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::LeftWaitingRoom(LeftWaitingRoom { id, connection_ids })
            if id == bob_0.id() && connection_ids == HashSet::from_iter([bob_0.connection_id()])
    );
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantConnected { participant_id, connection_id, .. }
            if participant_id == bob_0.id() && connection_id == bob_0.connection_id()
    );

    // Device 1 remained an accepted lobby participant and can enter the conference on its own.
    bob_1
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = bob_1.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::JoinSuccess(..));

    // Both the moderator and the already-connected device 0 are notified about device 1.
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantConnected { participant_id, connection_id, .. }
            if participant_id == bob_1.id() && connection_id == bob_1.connection_id()
    );
    let event = bob_0.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantConnected { participant_id, connection_id, .. }
            if participant_id == bob_1.id() && connection_id == bob_1.connection_id()
    );
}

/// The lobby join carries a display name only when the participant is already present in the
/// conference with another connection.
#[test_log::test(tokio::test)]
async fn lobby_join_display_name_depends_on_conference_presence() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();

    // Bob's first device has no other connection in the conference, so the lobby join carries no
    // display name.
    let mut bob_0 = MockParticipantJoining::bob(0)
        .enter_lobby(&mut room)
        .await
        .unwrap();
    assert_eq!(bob_0.display_name(), None);

    // Bob's first device enters the conference.
    bob_0
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = bob_0.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::JoinSuccess(..));

    // A second device of Bob joins the lobby and is given Bob's display name, because Bob is now
    // already present in the conference.
    let bob_1 = MockParticipantJoining::bob(1)
        .enter_lobby(&mut room)
        .await
        .unwrap();
    assert_eq!(
        bob_1.display_name(),
        Some(&bob_public_user_profile().user_info.display_name)
    );
}

/// A lobby-only participant must stay invisible to moderators until they enter the waiting room.
///
/// 1. Spawn room with activated waiting room
/// 2. Bob lands in the lobby but does not enter the waiting room
/// 3. Alice joins as a moderator and her join snapshot shows an empty waiting room
/// 4. Alice receives no `JoinedWaitingRoom` event while Bob lingers in the lobby
/// 5. Bob enters the waiting room and only now becomes visible to the moderator
#[test_log::test(tokio::test)]
async fn lobby_only_participant_is_invisible_until_entering_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();

    // Bob lands in the lobby but does not enter the waiting room yet.
    let mut bob = MockParticipantJoining::bob(0)
        .enter_lobby(&mut room)
        .await
        .unwrap();

    // Alice joins as a moderator. Because Bob is only in the lobby, her JoinSuccess lists no
    // waiting participants.
    let mut alice = room.join_alice_moderator(0).await;
    let waiting_participants = alice
        .join_success()
        .module_data
        .get::<ModerationState>()
        .expect("Module data must be present")
        .expect("Module data must not be none")
        .moderator_data
        .expect("Moderator data must be present")
        .waiting_room_participants;
    assert!(
        waiting_participants.is_empty(),
        "Lobby-only participant must not appear in the waiting room: {waiting_participants:?}"
    );

    // Bob is not notified about Alice joining
    assert!(bob.received_nothing());

    // Bob enters the waiting room and only now becomes visible to the moderator.
    bob.send_core_command(CoreCommand::EnterWaitingRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = bob.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::InWaitingRoom);

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::JoinedWaitingRoom { participant_id, connection_ids, .. }
            if participant_id == bob.id() && connection_ids == vec![bob.connection_id()]
    );
}

#[test_log::test(tokio::test)]
async fn enable_waiting_room_for_everyone() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::ForEveryone,
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
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    // Sending the command again produces the same event
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::ForEveryone,
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
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );
}

#[test_log::test(tokio::test)]
async fn enable_waiting_room_for_guests() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::ForGuests,
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
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForGuests
        }
    );

    // Sending the command again produces the same event
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::ForGuests,
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        event,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForGuests
        }
    );
}

#[test_log::test(tokio::test)]
async fn waiting_room_already_in_state() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::Disabled)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice disables the waiting room, even tough it already is disabled
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::Disabled,
            },
            None,
        )
        .await
        .unwrap();

    // Alice is notified that the waiting room has the desired state
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::Disabled
        }
    );

    // Bob does not get notified because the waiting room state did not actually change
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn enable_waiting_room_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to enable the waiting room
    bob.send_command::<ModerationModule>(
        ModerationCommand::ChangeWaitingRoomState {
            new_state: WaitingRoom::ForEveryone,
        },
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;

    // Bob is not allowed to enable the waiting room because he isn't a moderator
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn disable_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::Disabled,
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
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::Disabled
        }
    );
}

#[test_log::test(tokio::test)]
async fn disable_waiting_room_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice enables the waiting room
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::ForEveryone,
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the waiting room enabled event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    // Bob receives the waiting room enabled event
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    // Bob tries to disable the waiting room
    bob.send_command::<ModerationModule>(
        ModerationCommand::ChangeWaitingRoomState {
            new_state: WaitingRoom::Disabled,
        },
        None,
    )
    .await
    .unwrap();
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;

    // Bob is not allowed to disable the waiting room because he isn't a moderator
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn send_to_waiting_room_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::ForEveryone,
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
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    bob.send_command::<ModerationModule>(
        ModerationCommand::SendToWaitingRoom { target: alice.id() },
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn cannot_send_owner_to_waiting_room() {
    let mut room = TestRoom::builder()
        .owner(bob_public_user_profile())
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::SendToWaitingRoom { target: bob.id() },
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
        ModerationEvent::Error(ModerationError::CannotSendRoomOwnerToWaitingRoom)
    );
}

#[test_log::test(tokio::test)]
async fn send_to_waiting_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::Disabled) // waiting room is enabled automatically by move
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice sends Bob to the waiting room
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::SendToWaitingRoom { target: bob.id() },
            None,
        )
        .await
        .unwrap();

    // Alice and bob receive the waiting room enabled event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    // Alice receives the ParticipantDisconnected and JoinedWaitingRoom events
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason,
        } if participant_id == bob.id() && connection_id == bob.connection_id() && reason == DisconnectReason::SentToWaitingRoom
    );

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    let CoreEvent::JoinedWaitingRoom {
        participant_id,
        connection_ids,
        ..
    } = event
    else {
        panic!("Expected JoinedWaitingRoom event, got: {:?}", event);
    };
    assert_eq!(participant_id, bob.id());
    assert_eq!(connection_ids, vec![bob.connection_id()]);

    // Bob receives the SentToWaitingRoom event
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::SentToWaitingRoom);

    // Bob does not receive the JoinedWaitingRoom event
    assert!(bob.received_nothing());

    let bob = bob.in_waiting_room();

    // Alice accepts bob
    let mut bob_0 = accept_participant(&mut alice, bob).await;

    // Bob can now join directly with a second device again
    let bob_1 = room.join_bob(1).await;

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantConnected {
            participant_id,
            connection_id,
            ..
        } if participant_id == bob_1.id() && connection_id == bob_1.connection_id()
    );

    let event = bob_0.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantConnected {
            participant_id,
            connection_id,
            ..
        } if participant_id == bob_1.id() && connection_id == bob_1.connection_id()
    );
}

/// A guest that is sent to the waiting room keeps the guest role.
///
/// 1. Spawn room with deactivated waiting room
/// 2. Alice joins as a moderator, Gustav joins as a guest
/// 3. Alice sends Gustav to the waiting room, which enables the waiting room for guests
/// 4. Alice accepts Gustav, who enters the room as a guest again
#[test_log::test(tokio::test)]
async fn guest_sent_to_waiting_room_keeps_guest_role() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::Disabled)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::SendToWaitingRoom {
                target: gustav.id(),
            },
            None,
        )
        .await
        .unwrap();

    for participant in [&mut alice, &mut gustav] {
        let event = participant
            .receive_event::<ModerationModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(
            event,
            ModerationEvent::WaitingRoomUpdated {
                new_state: WaitingRoom::ForGuests
            }
        );
    }

    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantDisconnected { participant_id, .. } if participant_id == gustav.id()
    );
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::JoinedWaitingRoom { participant_id, .. } if participant_id == gustav.id()
    );

    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::SentToWaitingRoom);

    let gustav = accept_participant(&mut alice, gustav.in_waiting_room()).await;
    assert_eq!(gustav.join_success().role, Role::Guest);
}

/// A moderator that is sent to the waiting room has their moderator role revoked and can no longer
/// enter the room without being accepted first.
///
/// 1. Spawn room with activated waiting room
/// 2. Alice and Frank join as moderators
/// 3. Frank connects a second device, it lands in the lobby and may skip the waiting room
/// 4. Alice sends Frank to the waiting room
/// 5. Frank's lobby device is told that he can no longer enter
/// 6. Neither of Frank's devices can enter the conference without being accepted
/// 7. Alice accepts Frank into the room and he can enter, but stays a regular user without
///    moderator rights
#[test_log::test(tokio::test)]
async fn moderator_sent_to_waiting_room_cannot_enter_without_acceptance() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let frank_0 = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let mut frank_1 = room.lobby_frank(1).await;
    assert!(frank_1.can_enter());

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::SendToWaitingRoom {
                target: frank_0.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the ParticipantDisconnected and JoinedWaitingRoom events
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason,
        } if participant_id == frank_0.id()
         && connection_id == frank_0.connection_id()
         && reason == DisconnectReason::SentToWaitingRoom
    );
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::JoinedWaitingRoom {
            participant_id,
            connection_ids,
            ..
        } if participant_id == frank_0.id()
           && connection_ids == vec![frank_0.connection_id()]
    );

    let mut frank_0 = frank_0.in_waiting_room();
    let event = frank_0
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::SentToWaitingRoom);

    // Frank lost his moderator rights, so his lobby device can no longer skip the waiting room
    let event = frank_1
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::EntryPermissionChanged { can_enter: false }
    );

    // Frank's last conference connection left, so the lobby device loses the display name
    let event = frank_1
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameAssigned { new_name: None }
    );

    // Frank cannot enter the conference from the waiting room...
    frank_0
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = frank_0.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::Error(CoreError::NotAccepted));

    // ...nor from the lobby.
    frank_1
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let event = frank_1.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(event, CoreEvent::Error(CoreError::NotAccepted));

    // Alice accepts frank from the waiting room
    let role = accept_participant(&mut alice, frank_0)
        .await
        .join_success()
        .role;

    assert_eq!(role, Role::User);
}

/// A participant that is granted moderator right while in the lobby may skip the waiting room and
/// enter the conference without being accepted.
///
/// 1. Spawn room with activated waiting room
/// 2. Alice and Frank join as moderators
/// 3. Alice sends Frank to the waiting room, which removes his moderator rights
/// 4. Alice promotes Frank to moderator
/// 5. Frank is told that he can enter and enters the conference without being accepted
#[test_log::test(tokio::test)]
async fn participant_promoted_to_moderator_in_waiting_room_can_enter() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let frank = room.join_frank_moderator(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::SendToWaitingRoom { target: frank.id() },
            None,
        )
        .await
        .unwrap();

    // Alice receives the ParticipantDisconnected and JoinedWaitingRoom events
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason,
        } if participant_id == frank.id()
         && connection_id == frank.connection_id()
         && reason == DisconnectReason::SentToWaitingRoom
    );
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::JoinedWaitingRoom {
            participant_id,
            connection_ids,
            ..
        } if participant_id == frank.id()
           && connection_ids == vec![frank.connection_id()]
    );

    let mut frank = frank.in_waiting_room();
    let event = frank
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ModerationEvent::SentToWaitingRoom);

    // Alice promotes Frank to moderator
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::UpdateRole {
                participant_id: frank.id(),
                new_role: Role::Moderator,
            },
            None,
        )
        .await
        .unwrap();

    // As a moderator, Frank may skip the waiting room again
    let event = frank
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::EntryPermissionChanged { can_enter: true }
    );

    frank
        .send_core_command(CoreCommand::EnterRoom { display_name: None }, None)
        .await
        .unwrap();
    let frank = frank.join_success().await.unwrap();
    assert_eq!(frank.join_success().role, Role::Moderator);
}

/// Changing the waiting room state notifies participants waiting in the lobby whether they may
/// still enter the conference.
///
/// 1. Spawn a room with the waiting room disabled
/// 2. Bob lands in the lobby and may enter the conference
/// 3. Alice joins as a moderator and enables the waiting room for everyone
/// 4. Bob is told that he can no longer enter
#[test_log::test(tokio::test)]
async fn enabling_waiting_room_notifies_lobby_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();

    // Bob lands in the lobby and, with the waiting room disabled, may enter the conference.
    let mut bob = MockParticipantJoining::bob(0)
        .enter_lobby(&mut room)
        .await
        .unwrap();
    assert!(bob.can_enter());

    let mut alice = room.join_alice_moderator(0).await;

    // Alice enables the waiting room for everyone.
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeWaitingRoomState {
                new_state: WaitingRoom::ForEveryone,
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
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    // Bob is told that he can no longer enter.
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::EntryPermissionChanged { can_enter: false }
    );

    assert!(bob.received_nothing());
}

/// Regression test for leaked waiting-room participants.
///
/// A waiting-room participant whose connection is lost through the congestion
/// path (a failed lobby send that moves the connection into `lobby.disconnects`,
/// rather than a clean close-frame handshake) must still be removed from the
/// waiting room, and the moderators must be notified with `LeftWaitingRoom`.
///
/// Here Bob's connection dies without a close frame at the same moment the
/// moderator accepts him. Delivering `Accepted` to Bob's now-dead lobby
/// connection fails, which routes his disconnect through the congestion path.
/// If that path is mishandled, moderators never learn that Bob left and his
/// entry is leaked in the waiting room.
#[test_log::test(tokio::test)]
async fn lost_waiting_room_connection_notifies_moderators() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .waiting_room(WaitingRoom::ForEveryone)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let bob = room.waiting_room_bob(0).await;
    let bob_id = bob.id();

    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::JoinedWaitingRoom { .. });

    // Trigger the lobby disconnect on connection loss. We need a failed attempt
    // to send an event to Bob. Bob's connection dies without a close frame while
    // the moderator accepts him, so the room's `Accepted` message is sent to a
    // dead lobby connection.
    drop(bob);
    alice
        .send_command::<ModerationModule>(ModerationCommand::Accept { target: bob_id }, None)
        .await
        .unwrap();

    // Alice receives the accept response for Bob ...
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::ParticipantAccepted {
            participant_id: bob_id
        }
    );

    // ... and must then be told that Bob left the waiting room, even though his
    // connection was lost through the congestion path. Otherwise he is leaked in
    // the waiting room.
    let event = alice
        .receive::<CoreEvent>()
        .await
        .expect("moderators must be notified that the waiting-room participant left")
        .payload;
    assert_matches!(
        event,
        CoreEvent::LeftWaitingRoom(LeftWaitingRoom { id, .. }) if id == bob_id
    );
}
