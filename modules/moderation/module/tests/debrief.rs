// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::{
    participant::{MockParticipantJoined, ReceiveError},
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_signaling::signaling_event::SignalingEvent;
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    core::CoreEvent,
    disconnect_reason::DisconnectReason,
    kick_reason::KickReason,
    room_parameters::WaitingRoom,
    signaling::websocket::{CloseFrame, SignalingSocketMessage},
};
use opentalk_roomserver_types_moderation::{
    KickScope,
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
};
use opentalk_types_signaling::ParticipantId;

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to kick charlie
    bob.send_command::<ModerationModule>(ModerationCommand::Debrief(KickScope::All), None)
        .await
        .unwrap();

    // Bob receives an error because he is not a moderator
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn debrief_all() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::Debrief(KickScope::All), None)
        .await
        .unwrap();

    // Everyone receives the waiting room enabled event
    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    let event = gustav.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    // Everyone gets kicked
    let expected = BTreeSet::from_iter([
        (bob.id(), bob.connection_id()),
        (gustav.id(), gustav.connection_id()),
    ]);
    verify_disconnects(&mut alice, expected, true).await;

    let expected = BTreeSet::from_iter([
        (alice.id(), alice.connection_id()),
        (gustav.id(), gustav.connection_id()),
    ]);
    verify_disconnects(&mut bob, expected, true).await;

    let expected = BTreeSet::from_iter([
        (alice.id(), alice.connection_id()),
        (bob.id(), bob.connection_id()),
    ]);
    verify_disconnects(&mut gustav, expected, true).await;
}

#[test_log::test(tokio::test)]
async fn debrief_users_and_guests() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::Debrief(KickScope::UsersAndGuests),
            None,
        )
        .await
        .unwrap();
    // Alice receives the debriefing started event
    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::DebriefingStarted {
            issued_by: alice.id()
        }
    );

    // Everyone receives the waiting room enabled event
    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    let event = gustav.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForEveryone
        }
    );

    // Alice does not get kicked, because she is a moderator
    let expected = BTreeSet::from_iter([
        (bob.id(), bob.connection_id()),
        (gustav.id(), gustav.connection_id()),
    ]);
    verify_disconnects(&mut alice, expected, false).await;

    // Bob gets kicked
    let expected = BTreeSet::from_iter([(gustav.id(), gustav.connection_id())]);
    verify_disconnects(&mut bob, expected, true).await;

    // Gustav gets kicked
    let expected = BTreeSet::from_iter([(bob.id(), bob.connection_id())]);
    verify_disconnects(&mut gustav, expected, true).await;
}

#[test_log::test(tokio::test)]
async fn debrief_guests() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::Debrief(KickScope::Guests), None)
        .await
        .unwrap();

    // Alice receives the debriefing started event
    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::DebriefingStarted {
            issued_by: alice.id()
        }
    );

    // Bob receives the debriefing started event
    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::DebriefingStarted {
            issued_by: alice.id()
        }
    );

    // Everyone receives the waiting room enabled event
    let event = alice.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForGuests
        }
    );

    let event = bob.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForGuests
        }
    );

    let event = gustav.receive_event::<ModerationModule>().await.unwrap();
    assert_eq!(
        event.payload,
        ModerationEvent::WaitingRoomUpdated {
            new_state: WaitingRoom::ForGuests
        }
    );

    // Alice does not get kicked, because she isn't a guest
    let expected = BTreeSet::from_iter([(gustav.id(), gustav.connection_id())]);
    verify_disconnects(&mut alice, expected, false).await;

    // Bob does not get kicked, because he isn't a guest
    let expected = BTreeSet::from_iter([(gustav.id(), gustav.connection_id())]);
    verify_disconnects(&mut bob, expected, false).await;

    // Gustav gets kicked
    let expected = BTreeSet::from_iter([]);
    verify_disconnects(&mut gustav, expected, true).await;
}

async fn verify_disconnects(
    participant: &mut MockParticipantJoined,
    expected: BTreeSet<(ParticipantId, ConnectionId)>,
    expect_self_kick: bool,
) {
    let mut expected_messages = expected.len();
    if expect_self_kick {
        expected_messages += 2;
    }
    let mut received_kick = false;
    let mut received_close = false;
    for _ in 0..expected_messages {
        match participant.receive::<CoreEvent>().await {
            Ok(SignalingEvent { payload, .. }) => {
                let CoreEvent::ParticipantDisconnected {
                    participant_id,
                    connection_id,
                    reason,
                } = payload
                else {
                    panic!("Received unexpected CoreEvent");
                };

                assert_eq!(DisconnectReason::Debriefed, reason);
                assert!(expected.contains(&(participant_id, connection_id)));
            }
            Err(ReceiveError::InvalidJson { message, .. }) if expect_self_kick => {
                let SignalingSocketMessage::Text(text) = message else {
                    panic!("Received unexpected SignalingSocketMessage: {message:?}");
                };
                assert_eq!(
                    ModerationEvent::Kicked {
                        reason: KickReason::Debriefed
                    },
                    serde_json::from_str::<SignalingEvent<ModerationEvent>>(&text)
                        .unwrap()
                        .payload
                );
                received_kick = true;
            }
            Err(ReceiveError::UnexpectedMessage(SignalingSocketMessage::Close(Some(
                close_frame,
            )))) if expect_self_kick => {
                assert_eq!(
                    CloseFrame {
                        code: 1000,
                        reason: "closed by server".to_string()
                    },
                    close_frame
                );
                received_close = true;
                break;
            }
            Err(e) => panic!("Received error: {e}"),
        }
    }

    assert_eq!(received_kick, expect_self_kick);
    assert_eq!(received_close, expect_self_kick);
    assert!(
        participant.received_nothing(),
        "Received additional messages",
    );
}
