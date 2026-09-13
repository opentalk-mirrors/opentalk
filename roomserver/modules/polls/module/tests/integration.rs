// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{assert_matches, collections::BTreeSet, iter, time::Duration};

use opentalk_roomserver_module_polls::PollsModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        breakout_id::BreakoutId,
        command::BreakoutCommand,
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};
use opentalk_roomserver_types_polls::{
    Choice, ChoiceId, Item, PollId, Results,
    command::{Choices, PollsCommand, Vote},
    event::{Error, PollsEvent},
    state::PollsState,
};

#[test_log::test(tokio::test)]
async fn can_not_start_second_poll() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts a new poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Poll 0".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Poll is started successfully
    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );

    // Alice tries to start a second poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Poll 1".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // An error is returned
    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::StillRunning)
    );
}

#[test_log::test(tokio::test)]
async fn non_moderator_cant_start_poll() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut bob = room.join_bob(0).await;

    // Bob tries to start a poll
    bob.send_command::<PollsModule>(
        PollsCommand::Start {
            topic: "Test".into(),
            live: false,
            multiple_choice: false,
            choices: vec!["a".into(), "b".into()],
            duration: Duration::from_mins(5),
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives an error message because he is not a moderator
    assert_eq!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn can_not_start_poll_with_invalid_duration() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to start a poll with a very long duration
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_hours(24 * 30), // 30 days
            },
            None,
        )
        .await
        .unwrap();

    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidDuration { .. })
    );
}

#[test_log::test(tokio::test)]
async fn can_not_start_poll_with_invalid_topic_length() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to start a poll with a topic name that is too long
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: iter::repeat_n("a", 101).collect(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidTopicLength { .. })
    );
}

#[test_log::test(tokio::test)]
async fn can_not_start_polls_with_invalid_choice_count() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to start a poll with too few choices
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Poll 0".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidChoiceCount { .. })
    );

    // Alice tries to start a poll with too many choices
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Poll 0".into(),
                live: false,
                multiple_choice: false,
                choices: iter::repeat_n("a".into(), 65).collect(),
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidChoiceCount { .. })
    );
}

#[test_log::test(tokio::test)]
async fn can_not_start_poll_with_invalid_option_length() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to start a poll with a topic name that is too long
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec![iter::repeat_n("a", 101).collect(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidChoiceDescriptionLength { .. })
    );
}

#[test_log::test(tokio::test)]
async fn can_not_vote_on_wrong_poll() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice tries to vote, but no poll is running
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: PollId::generate(),
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidPollId)
    );

    // Alice starts a poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Poll started successfully
    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );

    // Alice tries to vote on a different poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: PollId::generate(),
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidPollId)
    );
}

#[test_log::test(tokio::test)]
async fn can_not_give_multiple_choices_on_single_choice_poll() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts a poll without multiple choice
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    // Alice tries to vote on multiple choices
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from([ChoiceId::from_u32(0), ChoiceId::from_u32(1)]),
                },
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::MultipleChoicesNotAllowed)
    );
}

#[test_log::test(tokio::test)]
async fn can_not_vote_on_invalid_choice() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts a poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    // Alice tries to vote on a different poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(2),
                },
            }),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidChoiceId)
    );
}

#[test_log::test(tokio::test)]
async fn non_moderator_can_not_finish_poll() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    let event = bob.receive_event::<PollsModule>().await.unwrap().payload;
    // Bob receives the started event
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    // Bob tries to stop the poll
    bob.send_command::<PollsModule>(PollsCommand::Finish { id }, None)
        .await
        .unwrap();

    assert_eq!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn can_not_finish_poll_with_invalid_id() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts a poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Poll is started successfully
    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );

    alice
        .send_command::<PollsModule>(
            PollsCommand::Finish {
                id: PollId::generate(),
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Error(Error::InvalidPollId)
    );
}

#[test_log::test(tokio::test)]
async fn receiving_live_update_when_enabled() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a poll with live update
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: true,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Poll is started successfully
    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    assert_matches!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );

    // Alice votes
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            }),
            None,
        )
        .await
        .unwrap();

    // Alice and Bob receive an update
    let results = vec![
        Item {
            id: ChoiceId::from_u32(0),
            count: 1,
        },
        Item {
            id: ChoiceId::from_u32(1),
            count: 0,
        },
    ];
    let event = PollsEvent::LiveUpdate(Results { id, results });
    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );
    assert_eq!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );
}

#[test_log::test(tokio::test)]
async fn not_receiving_live_update_when_disabled() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a poll without live update
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Poll is started successfully
    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    assert_matches!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );

    // Alice votes
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            }),
            None,
        )
        .await
        .unwrap();

    // Alice and Bob do not receive anything
    assert!(alice.received_nothing());
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn single_choice_poll() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Poll is started successfully
    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    assert_matches!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );

    // Alice votes
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            }),
            None,
        )
        .await
        .unwrap();

    // Bob votes
    bob.send_command::<PollsModule>(
        PollsCommand::Vote(Vote {
            poll_id: id,
            choices: Choices::Single {
                choice_id: ChoiceId::from_u32(0),
            },
        }),
        None,
    )
    .await
    .unwrap();

    // Alice ends the poll
    alice
        .send_command::<PollsModule>(PollsCommand::Finish { id }, None)
        .await
        .unwrap();

    // Alice and Bob receive the results
    let event = PollsEvent::Done(Results {
        id,
        results: vec![
            Item {
                id: ChoiceId::from_u32(0),
                count: 2,
            },
            Item {
                id: ChoiceId::from_u32(1),
                count: 0,
            },
        ],
    });
    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );
    assert_eq!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );
}

#[test_log::test(tokio::test)]
async fn multiple_choice_poll() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a poll with multiple choice
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: true,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Poll is started successfully
    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    assert_matches!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );

    // Alice votes for both options
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Multiple {
                    choice_ids: BTreeSet::from([ChoiceId::from_u32(0), ChoiceId::from_u32(1)]),
                },
            }),
            None,
        )
        .await
        .unwrap();

    // Bob votes for a single option
    bob.send_command::<PollsModule>(
        PollsCommand::Vote(Vote {
            poll_id: id,
            choices: Choices::Single {
                choice_id: ChoiceId::from_u32(0),
            },
        }),
        None,
    )
    .await
    .unwrap();

    // Alice ends the poll
    alice
        .send_command::<PollsModule>(PollsCommand::Finish { id }, None)
        .await
        .unwrap();

    // Alice and Bob receive the results
    let event = PollsEvent::Done(Results {
        id,
        results: vec![
            Item {
                id: ChoiceId::from_u32(0),
                count: 2,
            },
            Item {
                id: ChoiceId::from_u32(1),
                count: 1,
            },
        ],
    });
    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );
    assert_eq!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );
}

#[test_log::test(tokio::test)]
async fn can_update_vote() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts a poll with multiple choice
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: true,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Poll is started successfully
    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    // Alice votes for option "a"
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            }),
            None,
        )
        .await
        .unwrap();

    // Alice reconsiders and votes for option "b"
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(1),
                },
            }),
            None,
        )
        .await
        .unwrap();

    // Alice ends the poll
    alice
        .send_command::<PollsModule>(PollsCommand::Finish { id }, None)
        .await
        .unwrap();

    // The results contain a single vote for option "b"
    let event = PollsEvent::Done(Results {
        id,
        results: vec![
            Item {
                id: ChoiceId::from_u32(0),
                count: 0,
            },
            Item {
                id: ChoiceId::from_u32(1),
                count: 1,
            },
        ],
    });
    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );
}

#[test_log::test(tokio::test)]
async fn polls_expire() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a poll
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_secs(0),
            },
            None,
        )
        .await
        .unwrap();

    // Alice and Bob receive the started event
    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );
    assert_matches!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Started { .. }
    );

    // Alice and Bob receive the done event
    assert_matches!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Done(..)
    );
    assert_matches!(
        bob.receive_event::<PollsModule>().await.unwrap().payload,
        PollsEvent::Done(..)
    );
}

#[test_log::test(tokio::test)]
async fn breakout_scopes() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts breakout rooms
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

    // Alice starts a poll with live updates in her room
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: true,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the started event
    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    // Bob doesn't
    assert!(bob.received_nothing());

    // Alice votes
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            }),
            None,
        )
        .await
        .unwrap();

    // Alice receives live updates
    let results = vec![
        Item {
            id: ChoiceId::from_u32(0),
            count: 1,
        },
        Item {
            id: ChoiceId::from_u32(1),
            count: 0,
        },
    ];
    let event = PollsEvent::LiveUpdate(Results { id, results });
    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );

    // Bob doesn't
    assert!(bob.received_nothing());

    // Alice ends the poll
    alice
        .send_command::<PollsModule>(PollsCommand::Finish { id }, None)
        .await
        .unwrap();

    // Alice receives the results
    let event = PollsEvent::Done(Results {
        id,
        results: vec![
            Item {
                id: ChoiceId::from_u32(0),
                count: 1,
            },
            Item {
                id: ChoiceId::from_u32(1),
                count: 0,
            },
        ],
    });
    assert_eq!(
        alice.receive_event::<PollsModule>().await.unwrap().payload,
        event
    );

    // Bob doesn't
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn switch_breakout_room_join_module_data() {
    let mut room = TestRoom::builder().register_module::<PollsModule>().spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts breakout rooms
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

    // Alice starts a poll her room
    alice
        .send_command::<PollsModule>(
            PollsCommand::Start {
                topic: "Test".into(),
                live: false,
                multiple_choice: false,
                choices: vec!["a".into(), "b".into()],
                duration: Duration::from_mins(5),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the started event
    let event = alice.receive_event::<PollsModule>().await.unwrap().payload;
    let PollsEvent::Started { id, .. } = event else {
        unreachable!("Poll did not start");
    };

    // Alice votes
    alice
        .send_command::<PollsModule>(
            PollsCommand::Vote(Vote {
                poll_id: id,
                choices: Choices::Single {
                    choice_id: ChoiceId::from_u32(0),
                },
            }),
            None,
        )
        .await
        .unwrap();

    // Alice switches to the main room
    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Main)
        .await;
    // And back to breakout room 0
    let event = alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(BreakoutId::from(0)))
        .await;

    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        unreachable!("Received wrong event");
    };
    let state = own_data
        .get::<PollsState>()
        .expect("PollsState should be present in ModuleData")
        .expect("PollsState should not be None");

    // Alice received the correct module data
    assert_eq!(
        state,
        PollsState {
            id,
            topic: "Test".into(),
            live: false,
            multiple_choice: false,
            choices: vec![
                Choice {
                    id: ChoiceId::from_u32(0),
                    content: "a".into()
                },
                Choice {
                    id: ChoiceId::from_u32(1),
                    content: "b".into()
                }
            ],
            started: state.started,
            duration: Duration::from_mins(5),
        }
    );
}
