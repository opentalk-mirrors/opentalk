// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashSet;

use opentalk_roomserver_module_reaction::ReactionModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_reaction::{
    Reaction, ReactionCommand, ReactionEvent, ReactionState, event::ReactionError,
    state::ReactionRestrictions,
};
use opentalk_types_signaling::ParticipantId;
use pretty_assertions::assert_eq;

async fn enable_restrictions(
    sender: &mut MockParticipantJoined,
    others: &mut [&mut MockParticipantJoined],
    unrestricted_participants: HashSet<ParticipantId>,
) {
    sender
        .send_command::<ReactionModule>(
            ReactionCommand::EnableRestrictions {
                unrestricted_participants: unrestricted_participants.clone(),
            },
            None,
        )
        .await
        .unwrap();

    let event = sender
        .receive_event::<ReactionModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ReactionEvent::RestrictionsEnabled {
            unrestricted_participants: unrestricted_participants.clone(),
        }
    );

    for other in others {
        let event = other
            .receive_event::<ReactionModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(
            event,
            ReactionEvent::RestrictionsEnabled {
                unrestricted_participants: unrestricted_participants.clone(),
            }
        );
    }
}

#[test_log::test(tokio::test)]
async fn join_success() {
    let mut room = TestRoom::builder()
        .register_module::<ReactionModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let join_success = alice
        .join_success()
        .get_module::<ReactionState>()
        .expect("Reaction module data must be serializable")
        .expect("Reaction module data must be present");
    assert_eq!(
        join_success,
        ReactionState {
            restrictions: ReactionRestrictions::Disabled
        }
    );

    let unrestricted_participants = HashSet::from_iter([alice.id()]);
    enable_restrictions(&mut alice, &mut [], unrestricted_participants.clone()).await;

    let join_success = room
        .join_bob(0)
        .await
        .join_success()
        .get_module::<ReactionState>()
        .expect("Reaction module data must be serializable")
        .expect("Reaction module data must be present");
    assert_eq!(
        join_success,
        ReactionState {
            restrictions: ReactionRestrictions::Enabled {
                unrestricted_participants,
            }
        }
    );
}

#[test_log::test(tokio::test)]
async fn react() {
    let mut room = TestRoom::builder()
        .register_module::<ReactionModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ReactionModule>(
            ReactionCommand::React {
                reaction: Reaction::ThumbsUp,
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives her own reaction as an event
    let event = alice
        .receive_event::<ReactionModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ReactionEvent::Reacted {
            participant_id: alice.id(),
            reaction: Reaction::ThumbsUp
        }
    );

    // Bob receives Alice's reaction
    let event = bob.receive_event::<ReactionModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ReactionEvent::Reacted {
            participant_id: alice.id(),
            reaction: Reaction::ThumbsUp
        }
    );
}

#[test_log::test(tokio::test)]
async fn enable_restrictions_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ReactionModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<ReactionModule>(
        ReactionCommand::EnableRestrictions {
            unrestricted_participants: vec![bob.id()].into_iter().collect(),
        },
        None,
    )
    .await
    .unwrap();

    let event = bob.receive_event::<ReactionModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ReactionEvent::Error(ReactionError::InsufficientPermissions),
    );
}

#[test_log::test(tokio::test)]
async fn disable_restrictions_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ReactionModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<ReactionModule>(ReactionCommand::DisableRestrictions, None)
        .await
        .unwrap();

    let event = bob.receive_event::<ReactionModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ReactionEvent::Error(ReactionError::InsufficientPermissions),
    );
}

#[test_log::test(tokio::test)]
async fn restrictions() {
    let mut room = TestRoom::builder()
        .register_module::<ReactionModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice enables reaction restrictions, allowing only Bob to react
    let unrestricted_participants = HashSet::from_iter([bob.id()]);
    enable_restrictions(
        &mut alice,
        &mut [&mut bob],
        unrestricted_participants.clone(),
    )
    .await;

    // Bob can still react successfully
    bob.send_command::<ReactionModule>(
        ReactionCommand::React {
            reaction: Reaction::ThumbsDown,
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives his own reaction as an event
    let event = bob.receive_event::<ReactionModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ReactionEvent::Reacted {
            participant_id: bob.id(),
            reaction: Reaction::ThumbsDown
        }
    );

    // Alice receives Bob's reaction
    let event = alice
        .receive_event::<ReactionModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ReactionEvent::Reacted {
            participant_id: bob.id(),
            reaction: Reaction::ThumbsDown
        }
    );

    // Alice can no longer react
    alice
        .send_command::<ReactionModule>(
            ReactionCommand::React {
                reaction: Reaction::Heart,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<ReactionModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ReactionEvent::Error(ReactionError::Restricted),);

    assert!(bob.received_nothing());

    // Alice disables restrictions again
    alice
        .send_command::<ReactionModule>(ReactionCommand::DisableRestrictions, None)
        .await
        .unwrap();

    // Alice receives the restrictions disabled event
    let event = alice
        .receive_event::<ReactionModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, ReactionEvent::RestrictionsDisabled);

    // Bob receives the restrictions disabled event
    let event = bob.receive_event::<ReactionModule>().await.unwrap().payload;
    assert_eq!(event, ReactionEvent::RestrictionsDisabled);

    // Alice can react again
    alice
        .send_command::<ReactionModule>(
            ReactionCommand::React {
                reaction: Reaction::Joy,
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives her own reaction as an event
    let event = alice
        .receive_event::<ReactionModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ReactionEvent::Reacted {
            participant_id: alice.id(),
            reaction: Reaction::Joy
        }
    );

    // Bob receives Alice's reaction as an event
    let event = bob.receive_event::<ReactionModule>().await.unwrap().payload;
    assert_eq!(
        event,
        ReactionEvent::Reacted {
            participant_id: alice.id(),
            reaction: Reaction::Joy
        }
    );
}

#[test_log::test(tokio::test)]
async fn alice_in_breakout_bob_in_main() {
    let mut room = TestRoom::builder()
        .register_module::<ReactionModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_owned(),
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
        .send_command::<ReactionModule>(
            ReactionCommand::React {
                reaction: Reaction::ThumbsUp,
            },
            None,
        )
        .await
        .unwrap();
    alice.receive_event::<ReactionModule>().await.unwrap();

    alice
        .send_command::<ReactionModule>(
            ReactionCommand::EnableRestrictions {
                unrestricted_participants: HashSet::new(),
            },
            None,
        )
        .await
        .unwrap();
    alice.receive_event::<ReactionModule>().await.unwrap();

    alice
        .send_command::<ReactionModule>(ReactionCommand::DisableRestrictions, None)
        .await
        .unwrap();
    alice.receive_event::<ReactionModule>().await.unwrap();

    assert!(bob.received_nothing());
}
