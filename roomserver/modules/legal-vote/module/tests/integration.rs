// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{assert_matches, collections::HashMap, str::FromStr};

use insta::{assert_json_snapshot, assert_snapshot};
use opentalk_roomserver_module_legal_vote::LegalVoteModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_signaling::storage::module_resources::provider::ModuleResourceProvider;
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    core::CoreEvent,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_legal_vote::{
    LEGAL_VOTE_MODULE_ID, LegalVoteCommand, LegalVoteEvent,
    cancel::{CancelReason, CustomCancelReason},
    event::{FinalResults, LegalVoteError, Results, StopKind, VotingRecord},
    issue::{Issue, TechnicalIssueKind},
    parameters::Parameters,
    tally::Tally,
    token::Token,
    user_parameters::{AllowedParticipants, Name, UserParameters},
    vote::{LegalVoteId, VoteOption},
};
use opentalk_types_api_internal::module_resources::ModuleResourceFilter;
use opentalk_types_common::assets::AssetId;
use opentalk_types_signaling::ParticipantId;

#[test_log::test(tokio::test)]
async fn start_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<LegalVoteModule>(
        LegalVoteCommand::Start(UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([bob.id()]).unwrap(),
            enable_abstain: true,
            auto_close: true,
            duration: None,
            create_pdf: false,
            timezone: None,
        }),
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Error(LegalVoteError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn start_invalid_parameters() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<LegalVoteModule>(
            LegalVoteCommand::Start(UserParameters {
                pseudonymous: true,
                live: true,
                name: Name::from_str("Vote").unwrap(),
                subtitle: None,
                topic: None,
                allowed_participants: AllowedParticipants::try_from([alice.id()]).unwrap(),
                enable_abstain: true,
                auto_close: true,
                duration: None,
                create_pdf: false,
                timezone: None,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Error(LegalVoteError::InvalidParameters)
    );
}

#[test_log::test(tokio::test)]
async fn start_already_active() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let parameters = UserParameters {
        pseudonymous: true,
        live: false,
        name: Name::from_str("Vote").unwrap(),
        subtitle: None,
        topic: None,
        allowed_participants: AllowedParticipants::try_from([alice.id()]).unwrap(),
        enable_abstain: true,
        auto_close: true,
        duration: None,
        create_pdf: false,
        timezone: None,
    };
    start_vote(&mut alice, parameters.clone(), &mut []).await;

    // Alice tries to start the vote again
    alice
        .send_command::<LegalVoteModule>(LegalVoteCommand::Start(parameters), None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Error(LegalVoteError::VoteAlreadyActive)
    );
}

#[test_log::test(tokio::test)]
async fn ineligible() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id]).unwrap(),
            enable_abstain: true,
            auto_close: true,
            duration: None,
            create_pdf: false,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Alice received a token
    assert!(tokens[&alice.id()].is_some());
    // Bob did not receive a token
    assert!(tokens[&bob.id()].is_none());

    // Bob tries to vote anyway with a wrong token
    bob.send_command::<LegalVoteModule>(
        LegalVoteCommand::Vote {
            legal_vote_id,
            option: VoteOption::Yes,
            token: Token::generate(),
        },
        None,
    )
    .await
    .unwrap();

    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, LegalVoteEvent::Error(LegalVoteError::InvalidToken));
}

#[test_log::test(tokio::test)]
async fn ineligible_participants() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut charlie = room.join_charlie(0).await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob, &mut charlie]).await;

    let bob_id = bob.id();
    alice
        .start_breakout_rooms(
            &mut [&mut bob, &mut charlie, &mut gustav],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".into(),
                    assignments: vec![bob_id],
                }],
                duration: None,
            },
        )
        .await;

    // Bob switches to the breakout room
    bob.switch_breakout_room(
        &mut [&mut alice, &mut charlie, &mut gustav],
        RoomKind::Breakout(0.into()),
    )
    .await;

    alice
        .send_command::<LegalVoteModule>(
            LegalVoteCommand::Start(UserParameters {
                pseudonymous: true,
                live: false,
                name: Name::from_str("Vote").unwrap(),
                subtitle: None,
                topic: None,
                allowed_participants: [alice.id(), bob.id(), charlie.id(), gustav.id()]
                    .try_into()
                    .unwrap(),
                enable_abstain: false,
                auto_close: false,
                duration: None,
                create_pdf: false,
                timezone: None,
            }),
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    let LegalVoteEvent::Error(LegalVoteError::IneligibleParticipants { participants }) = event
    else {
        panic!("Expected `AllowlistContainsIneligibleParticipants` error");
    };
    assert_eq!(participants.len(), 2);
    // Gustav is ineligible because he is a guest
    assert!(participants.contains(&gustav.id()));
    // Bob is ineligible because he is in another room
    assert!(participants.contains(&bob.id()));
}

#[test_log::test(tokio::test)]
async fn invalid_option() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: false,
            timezone: None,
        },
        &mut [],
    )
    .await;

    // Alice tries to abstain
    alice
        .send_command::<LegalVoteModule>(
            LegalVoteCommand::Vote {
                legal_vote_id,
                option: VoteOption::Abstain,
                token: tokens[&alice_id].unwrap(),
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, LegalVoteEvent::Error(LegalVoteError::InvalidOption));
}

#[test_log::test(tokio::test)]
async fn double_vote() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id]).unwrap(),
            enable_abstain: false,
            auto_close: false,
            duration: None,
            create_pdf: false,
            timezone: None,
        },
        &mut [],
    )
    .await;

    // Alice votes
    let token = tokens[&alice_id].unwrap();
    vote(&mut alice, legal_vote_id, VoteOption::Yes, token).await;

    // Alice tries to vote again with the same token
    alice
        .send_command::<LegalVoteModule>(
            LegalVoteCommand::Vote {
                legal_vote_id,
                option: VoteOption::Yes,
                token,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, LegalVoteEvent::Error(LegalVoteError::InvalidToken));

    // The results only contain a single vote
    let results = stop_vote(&mut alice, legal_vote_id, &mut []).await;
    assert_eq!(
        results,
        FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 0,
                abstain: None
            },
            voting_record: VotingRecord::TokenVotes(HashMap::from_iter([(token, VoteOption::Yes)]))
        })
    );
}

#[test_log::test(tokio::test)]
async fn pseudonymous() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Alice votes
    let alice_token = tokens[&alice_id].unwrap();
    vote(&mut alice, legal_vote_id, VoteOption::Yes, alice_token).await;

    // Bob does not get notified about Alice's vote
    assert!(bob.received_nothing());

    // Bob votes
    let bob_token = tokens[&bob.id()].unwrap();
    vote(&mut bob, legal_vote_id, VoteOption::No, bob_token).await;

    // The vote closes because auto_close is enabled
    let (stop_kind, results) = receive_stop_event(&mut alice, legal_vote_id, &mut [&mut bob]).await;
    assert_eq!(stop_kind, StopKind::Auto);
    assert_eq!(
        results,
        FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None
            },
            voting_record: VotingRecord::TokenVotes(HashMap::from([
                (alice_token, VoteOption::Yes),
                (bob_token, VoteOption::No),
            ])),
        })
    );

    // The protocol is stored as a module resource
    let filter =
        ModuleResourceFilter::new(room.id(), LEGAL_VOTE_MODULE_ID).id(*legal_vote_id.inner());
    let resources = room
        .downcast_module_resource_storage()
        .get(filter)
        .await
        .unwrap();

    assert_eq!(resources.len(), 1);
    assert_json_snapshot!(resources[0].data, {
        ".entries[].timestamp" => "[timestamp]",
        ".entries[].event.parameters.legal_vote_id" => "[legal_vote_id]",
        ".entries[].event.parameters.start_time" => "[timestamp]",
        ".entries[].event.parameters.allowed_participants" => "[vec]",
        ".entries[].event.parameters.allowed_users" => "[vec]",
        ".entries[].event.token" => "[token]",
    }, @r#"
    {
      "entries": [
        {
          "event": {
            "event": "start",
            "issuer": "00000000-0000-0000-0000-0000000a11ce",
            "parameters": {
              "allowed_participants": "[vec]",
              "allowed_users": "[vec]",
              "auto_close": true,
              "create_pdf": true,
              "enable_abstain": false,
              "initiator_id": "00000000-0000-0000-0000-0000000a11ce",
              "legal_vote_id": "[legal_vote_id]",
              "live": false,
              "max_votes": 2,
              "name": "Vote",
              "pseudonymous": true,
              "start_time": "[timestamp]"
            }
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "vote",
            "option": "yes",
            "token": "[token]"
          }
        },
        {
          "event": {
            "event": "vote",
            "option": "no",
            "token": "[token]"
          }
        },
        {
          "event": {
            "auto": null,
            "event": "stop"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "final_results",
            "no": 1,
            "results": "valid",
            "yes": 1
          },
          "timestamp": "[timestamp]"
        }
      ],
      "version": 1
    }
    "#);

    // Alice is notified that a pdf has been created
    let asset_id = receive_pdf(&mut alice).await;

    assert_eq!(room.file_count().await, 1);

    let pdf = room.stored_asset(asset_id).await.unwrap();
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    insta::with_settings!({filters => vec![
        (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}", "[timestamp]"),
        (r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "[legal_vote_id]"),
        (&format!("{alice_token}|{bob_token}"), "[token]"),
    ]}, {
        assert_snapshot!(content, @r"


        OpenTalk Vote Report
         Title : Vote

        Pseudonymous : Yes

        Referendum leader : Alice Aal

        Vote id : [legal_vote_id]

        Start : [timestamp]

        End : [timestamp]

        Report timezone : Europe/Berlin

        Participant count : 2

        Scheduled duration : Unlimited

        Abstention : Disallowed

        Automatic close : Enabled

        Vote ended due to : All users voted

        Number of votes : 2

        Results
         Vote Count

        Approval 1

        Disapproval 1

        Recorded votes
         Name Token Vote Timestamp

        Hidden [token] Approval —

        Hidden [token] Disapproval —

        Event log
         Name Timestamp Event
        ");
    });

    // Bob is not notified about the pdf
    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn public() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a vote
    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: false,
            live: true,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Alice votes
    let alice_token = tokens[&alice_id].unwrap();
    vote(&mut alice, legal_vote_id, VoteOption::Yes, alice_token).await;

    // Alice is notified about her own vote
    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    let updated = LegalVoteEvent::Updated {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 1,
                no: 0,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(HashMap::from_iter([(
                alice.id(),
                VoteOption::Yes,
            )])),
        },
    };
    assert_eq!(event, updated);

    // Bob is notified about Alice's vote
    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, updated);

    // Bob votes
    let bob_token = tokens[&bob.id()].unwrap();
    vote(&mut bob, legal_vote_id, VoteOption::No, bob_token).await;

    // Alice is notified about Bob's vote
    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    let updated = LegalVoteEvent::Updated {
        legal_vote_id,
        results: Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None,
            },
            voting_record: VotingRecord::UserVotes(HashMap::from_iter([
                (alice.id(), VoteOption::Yes),
                (bob.id(), VoteOption::No),
            ])),
        },
    };
    assert_eq!(event, updated);

    // Bob is notified about his own vote
    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, updated);

    // The vote closes because auto_close is enabled
    let (stop_kind, results) = receive_stop_event(&mut alice, legal_vote_id, &mut [&mut bob]).await;
    assert_eq!(stop_kind, StopKind::Auto);
    assert_eq!(
        results,
        FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 1,
                abstain: None
            },
            voting_record: VotingRecord::UserVotes(HashMap::from([
                (alice.id(), VoteOption::Yes),
                (bob.id(), VoteOption::No),
            ])),
        })
    );

    // The protocol is stored as a module resource
    let filter =
        ModuleResourceFilter::new(room.id(), LEGAL_VOTE_MODULE_ID).id(*legal_vote_id.inner());
    let resources = room
        .downcast_module_resource_storage()
        .get(filter)
        .await
        .unwrap();
    assert_eq!(resources.len(), 1);
    assert_json_snapshot!(resources[0].data, {
        ".entries[].timestamp" => "[timestamp]",
        ".entries[].event.parameters.legal_vote_id" => "[legal_vote_id]",
        ".entries[].event.parameters.start_time" => "[timestamp]",
        ".entries[].event.parameters.allowed_participants" => "[vec]",
        ".entries[].event.parameters.allowed_users" => "[vec]",
        ".entries[].event.token" => "[token]",
    }, @r#"
    {
      "entries": [
        {
          "event": {
            "event": "start",
            "issuer": "00000000-0000-0000-0000-0000000a11ce",
            "parameters": {
              "allowed_participants": "[vec]",
              "allowed_users": "[vec]",
              "auto_close": true,
              "create_pdf": true,
              "enable_abstain": false,
              "initiator_id": "00000000-0000-0000-0000-0000000a11ce",
              "legal_vote_id": "[legal_vote_id]",
              "live": true,
              "max_votes": 2,
              "name": "Vote",
              "pseudonymous": false,
              "start_time": "[timestamp]"
            }
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "vote",
            "issuer": "00000000-0000-0000-0000-0000000a11ce",
            "option": "yes",
            "participant_id": "00000000-0000-0000-0000-0000000a11ce",
            "token": "[token]"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "vote",
            "issuer": "00000000-0000-0000-0000-000000000b0b",
            "option": "no",
            "participant_id": "00000000-0000-0000-0000-000000000b0b",
            "token": "[token]"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "auto": null,
            "event": "stop"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "final_results",
            "no": 1,
            "results": "valid",
            "yes": 1
          },
          "timestamp": "[timestamp]"
        }
      ],
      "version": 1
    }
    "#);

    // Alice is notified that a pdf has been created
    let asset_id = receive_pdf(&mut alice).await;

    assert_eq!(room.file_count().await, 1);
    let pdf = room.stored_asset(asset_id).await.unwrap();
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    insta::with_settings!({filters => vec![
        (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}", "[timestamp]"),
        (r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "[legal_vote_id]"),
        (&format!("{alice_token}|{bob_token}"), "[token]"),
    ]}, {
        assert_snapshot!(content, @r"


        OpenTalk Vote Report
         Title : Vote

        Pseudonymous : No

        Referendum leader : Alice Aal

        Vote id : [legal_vote_id]

        Start : [timestamp]

        End : [timestamp]

        Report timezone : Europe/Berlin

        Participant count : 2

        Scheduled duration : Unlimited

        Abstention : Disallowed

        Automatic close : Enabled

        Vote ended due to : All users voted

        Number of votes : 2

        Results
         Vote Count

        Approval 1

        Disapproval 1

        Recorded votes
         Name Token Vote Timestamp

        Alice Aal [token] Approval [timestamp]

        Bob Barsch [token] Disapproval [timestamp]

        Event log
         Name Timestamp Event
        ");
    });
}

#[test_log::test(tokio::test)]
async fn cancel_initiator_left() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let alice_id = alice.id();
    start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Alice disconnects
    alice.disconnect().await.unwrap();

    // Bob is notified that the vote was cancelled
    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert!(
        matches!(event, LegalVoteEvent::Canceled { reason, .. } if reason == CancelReason::InitiatorLeft)
    );

    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn cancel_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let alice_id = alice.id();
    let (legal_vote_id, _) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Bob tries to cancel the vote
    bob.send_command::<LegalVoteModule>(
        LegalVoteCommand::Cancel {
            legal_vote_id,
            reason: CustomCancelReason::from_str("Take this, Alice!").unwrap(),
        },
        None,
    )
    .await
    .unwrap();

    // Bob is not allowed to cancel the vote because he is isn't a moderator
    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Error(LegalVoteError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn cancel() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts the vote
    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [],
    )
    .await;

    // Alice cancels the vote
    let reason = CustomCancelReason::from_str("never mind...").unwrap();
    alice
        .send_command::<LegalVoteModule>(
            LegalVoteCommand::Cancel {
                legal_vote_id,
                reason: reason.clone(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the cancelled event
    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_matches!(
        event,
        LegalVoteEvent::Canceled {
            legal_vote_id: produced_id,
            reason: produced_reason,
            end_time: _,
        } if produced_id == legal_vote_id && produced_reason == CancelReason::Custom(reason)
    );

    // The protocol is stored as a module resource
    let filter =
        ModuleResourceFilter::new(room.id(), LEGAL_VOTE_MODULE_ID).id(*legal_vote_id.inner());
    let resources = room
        .downcast_module_resource_storage()
        .get(filter)
        .await
        .unwrap();
    assert_eq!(resources.len(), 1);
    assert_json_snapshot!(resources[0].data, {
        ".entries[].timestamp" => "[timestamp]",
        ".entries[].event.parameters.legal_vote_id" => "[legal_vote_id]",
        ".entries[].event.parameters.start_time" => "[timestamp]",
        ".entries[].event.token" => "[token]",
    }, @r#"
    {
      "entries": [
        {
          "event": {
            "event": "start",
            "issuer": "00000000-0000-0000-0000-0000000a11ce",
            "parameters": {
              "allowed_participants": [
                "00000000-0000-0000-0000-0000000a11ce"
              ],
              "allowed_users": [
                "00000000-0000-0000-0000-0000000a11ce"
              ],
              "auto_close": true,
              "create_pdf": true,
              "enable_abstain": false,
              "initiator_id": "00000000-0000-0000-0000-0000000a11ce",
              "legal_vote_id": "[legal_vote_id]",
              "live": false,
              "max_votes": 1,
              "name": "Vote",
              "pseudonymous": true,
              "start_time": "[timestamp]"
            }
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "custom": "never mind...",
            "event": "cancel",
            "issuer": "00000000-0000-0000-0000-0000000a11ce",
            "reason": "custom"
          },
          "timestamp": "[timestamp]"
        }
      ],
      "version": 1
    }
    "#);

    // Alice is notified that a pdf has been created
    let asset_id = receive_pdf(&mut alice).await;

    assert_eq!(room.file_count().await, 1);
    let pdf = room.stored_asset(asset_id).await.unwrap();
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    let alice_token = tokens[&alice.id()].unwrap();
    insta::with_settings!({filters => vec![
        (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}", "[timestamp]"),
        (r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "[legal_vote_id]"),
        (&format!("{alice_token}"), "[token]"),
    ]}, {
        assert_snapshot!(content, @"


        OpenTalk Vote Report
         Title : Vote

        Pseudonymous : Yes

        Referendum leader : Alice Aal

        Vote id : [legal_vote_id]

        Start : [timestamp]

        End : [timestamp]

        Report timezone : Europe/Berlin

        Participant count : 1

        Scheduled duration : Unlimited

        Abstention : Disallowed

        Automatic close : Enabled

        Vote ended due to : Aborted for custom reason:   never mind...

        Number of votes : 0

        Recorded votes
         Name Token Vote Timestamp

        Event log
         Name Timestamp Event
        ");
    });

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn stop_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a vote
    let alice_id = alice.id();
    let (legal_vote_id, _) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Bob tries to stop the vote
    bob.send_command::<LegalVoteModule>(LegalVoteCommand::Stop { legal_vote_id }, None)
        .await
        .unwrap();

    // Bob is not allowed to stop the vote because he is isn't a moderator
    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Error(LegalVoteError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn stop() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a vote
    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Alice votes
    let alice_token = tokens[&alice_id].unwrap();
    vote(&mut alice, legal_vote_id, VoteOption::Yes, alice_token).await;

    // Alice stops the vote
    let results = stop_vote(&mut alice, legal_vote_id, &mut [&mut bob]).await;
    assert_eq!(
        results,
        FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 0,
                abstain: None
            },
            voting_record: VotingRecord::TokenVotes(HashMap::from_iter([(
                alice_token,
                VoteOption::Yes
            )])),
        })
    );

    // The protocol is stored as a module resource
    let filter =
        ModuleResourceFilter::new(room.id(), LEGAL_VOTE_MODULE_ID).id(*legal_vote_id.inner());
    let resources = room
        .downcast_module_resource_storage()
        .get(filter)
        .await
        .unwrap();
    assert_eq!(resources.len(), 1);
    assert_json_snapshot!(resources[0].data, {
        ".entries[].timestamp" => "[timestamp]",
        ".entries[].event.parameters.legal_vote_id" => "[legal_vote_id]",
        ".entries[].event.parameters.start_time" => "[timestamp]",
        ".entries[].event.parameters.allowed_participants" => "[vec]",
        ".entries[].event.parameters.allowed_users" => "[vec]",
        ".entries[].event.token" => "[token]",
    }, @r#"
    {
      "entries": [
        {
          "event": {
            "event": "start",
            "issuer": "00000000-0000-0000-0000-0000000a11ce",
            "parameters": {
              "allowed_participants": "[vec]",
              "allowed_users": "[vec]",
              "auto_close": true,
              "create_pdf": true,
              "enable_abstain": false,
              "initiator_id": "00000000-0000-0000-0000-0000000a11ce",
              "legal_vote_id": "[legal_vote_id]",
              "live": false,
              "max_votes": 2,
              "name": "Vote",
              "pseudonymous": true,
              "start_time": "[timestamp]"
            }
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "vote",
            "option": "yes",
            "token": "[token]"
          }
        },
        {
          "event": {
            "by_user": "00000000-0000-0000-0000-0000000a11ce",
            "event": "stop"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "final_results",
            "no": 0,
            "results": "valid",
            "yes": 1
          },
          "timestamp": "[timestamp]"
        }
      ],
      "version": 1
    }
    "#);

    // Alice is notified that a pdf has been created
    let asset_id = receive_pdf(&mut alice).await;

    let pdf = room.stored_asset(asset_id).await.unwrap();
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    insta::with_settings!({filters => vec![
        (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}", "[timestamp]"),
        (r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "[legal_vote_id]"),
        (&alice_token.to_string(), "[token]"),
    ]}, {
        assert_snapshot!(content);
    });

    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn report_issue_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a vote
    let alice_id = alice.id();
    let (legal_vote_id, _) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Bob tries to report an issue
    bob.send_command::<LegalVoteModule>(
        LegalVoteCommand::ReportIssue {
            legal_vote_id,
            issue: Issue::Technical {
                kind: TechnicalIssueKind::Audio,
                description: None,
            },
        },
        None,
    )
    .await
    .unwrap();

    // Bob is not allowed to report an issue because he isn't allowed to vote
    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Error(LegalVoteError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn report_issue() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a vote
    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Alice votes
    let alice_token = tokens[&alice_id].unwrap();
    vote(&mut alice, legal_vote_id, VoteOption::Yes, alice_token).await;

    // Bob reports an issue
    bob.send_command::<LegalVoteModule>(
        LegalVoteCommand::ReportIssue {
            legal_vote_id,
            issue: Issue::Technical {
                kind: TechnicalIssueKind::Audio,
                description: None,
            },
        },
        None,
    )
    .await
    .unwrap();

    // Bob and Alice get notified about the reported issue
    let issue_alice = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        issue_alice,
        LegalVoteEvent::ReportedIssue {
            legal_vote_id,
            participant_id: None, // None because the vote is pseudonymous
            issue: Issue::Technical {
                kind: TechnicalIssueKind::Audio,
                description: None
            }
        }
    );

    let issue_bob = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(issue_alice, issue_bob);

    // Alice stops the vote
    let results = stop_vote(&mut alice, legal_vote_id, &mut [&mut bob]).await;
    assert_eq!(
        results,
        FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 0,
                abstain: None
            },
            voting_record: VotingRecord::TokenVotes(HashMap::from_iter([(
                alice_token,
                VoteOption::Yes
            )])),
        })
    );

    // The protocol is stored as a module resource
    let filter =
        ModuleResourceFilter::new(room.id(), LEGAL_VOTE_MODULE_ID).id(*legal_vote_id.inner());
    let resources = room
        .downcast_module_resource_storage()
        .get(filter)
        .await
        .unwrap();

    assert_eq!(resources.len(), 1);
    assert_json_snapshot!(resources[0].data, {
        ".entries[].timestamp" => "[timestamp]",
        ".entries[].event.parameters.legal_vote_id" => "[legal_vote_id]",
        ".entries[].event.parameters.start_time" => "[timestamp]",
        ".entries[].event.parameters.allowed_participants" => "[vec]",
        ".entries[].event.parameters.allowed_users" => "[vec]",
        ".entries[].event.token" => "[token]",
    }, @r#"
    {
      "entries": [
        {
          "event": {
            "event": "start",
            "issuer": "00000000-0000-0000-0000-0000000a11ce",
            "parameters": {
              "allowed_participants": "[vec]",
              "allowed_users": "[vec]",
              "auto_close": true,
              "create_pdf": true,
              "enable_abstain": false,
              "initiator_id": "00000000-0000-0000-0000-0000000a11ce",
              "legal_vote_id": "[legal_vote_id]",
              "live": false,
              "max_votes": 2,
              "name": "Vote",
              "pseudonymous": true,
              "start_time": "[timestamp]"
            }
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "vote",
            "option": "yes",
            "token": "[token]"
          }
        },
        {
          "event": {
            "event": "issue",
            "kind": "audio"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "by_user": "00000000-0000-0000-0000-0000000a11ce",
            "event": "stop"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "final_results",
            "no": 0,
            "results": "valid",
            "yes": 1
          },
          "timestamp": "[timestamp]"
        }
      ],
      "version": 1
    }
    "#);

    // Alice is notified that a pdf has been created
    let asset_id = receive_pdf(&mut alice).await;

    let pdf = room.stored_asset(asset_id).await.unwrap();
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    insta::with_settings!({filters => vec![
        (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}", "[timestamp]"),
        (r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "[legal_vote_id]"),
        (&alice_token.to_string(), "[token]"),
    ]}, {
        assert_snapshot!(content);
    });
}

#[test_log::test(tokio::test)]
async fn generate_pdf_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a vote
    let alice_id = alice.id();
    let (legal_vote_id, _) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: false,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Alice stops the vote
    stop_vote(&mut alice, legal_vote_id, &mut [&mut bob]).await;

    // Bob tries to generate a pdf
    bob.send_command::<LegalVoteModule>(
        LegalVoteCommand::GeneratePdf {
            legal_vote_id,
            timezone: None,
        },
        None,
    )
    .await
    .unwrap();

    // Bob is not allowed to generate a pdf because he isn't a moderator
    let event = bob
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Error(LegalVoteError::InsufficientPermissions)
    );
}

#[test_log::test(tokio::test)]
async fn generate_pdf_invalid_id() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<LegalVoteModule>(
            LegalVoteCommand::GeneratePdf {
                legal_vote_id: LegalVoteId::generate(),
                timezone: None,
            },
            None,
        )
        .await
        .unwrap();

    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, LegalVoteEvent::Error(LegalVoteError::InvalidVoteId));
}

#[test_log::test(tokio::test)]
async fn generate_pdf() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice starts a vote
    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: false,
            timezone: None,
        },
        &mut [],
    )
    .await;

    let alice_token = tokens[&alice_id].unwrap();
    // Alice stops the vote
    stop_vote(&mut alice, legal_vote_id, &mut []).await;

    // Alice generates a pdf
    alice
        .send_command::<LegalVoteModule>(
            LegalVoteCommand::GeneratePdf {
                legal_vote_id,
                timezone: None,
            },
            None,
        )
        .await
        .unwrap();

    let asset_id = receive_pdf(&mut alice).await;

    let pdf = room.stored_asset(asset_id).await.unwrap();
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    insta::with_settings!({filters => vec![
        (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}", "[timestamp]"),
        (r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "[legal_vote_id]"),
        (&alice_token.to_string(), "[token]"),
    ]}, {
        assert_snapshot!(content);
    });
}

#[test_log::test(tokio::test)]
async fn reconnect_during_vote() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice starts a vote
    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: true,
            live: false,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id, bob.id()]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: true,
            timezone: None,
        },
        &mut [&mut bob],
    )
    .await;

    // Bob disconnects
    bob.disconnect().await.unwrap();

    // Alice receives Bobs disconnect event
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::ParticipantDisconnected { .. });

    // Bob reconnects
    let mut bob = room.join_bob(0).await;

    // Alice receives Bobs reconnect event
    let event = alice.receive::<CoreEvent>().await.unwrap();
    assert_matches!(event.payload, CoreEvent::ParticipantConnected { .. });

    // Bob can still vote
    let bob_token = tokens[&bob.id()].unwrap();
    vote(&mut bob, legal_vote_id, VoteOption::No, bob_token).await;

    // Alice stops the vote
    let results = stop_vote(&mut alice, legal_vote_id, &mut [&mut bob]).await;
    assert_eq!(
        results,
        FinalResults::Valid(Results {
            tally: Tally {
                yes: 0,
                no: 1,
                abstain: None
            },
            voting_record: VotingRecord::TokenVotes(HashMap::from_iter([(
                bob_token,
                VoteOption::No
            )])),
        })
    );

    // The module resource contains Bobs disconnect and reconnect events
    let filter =
        ModuleResourceFilter::new(room.id(), LEGAL_VOTE_MODULE_ID).id(*legal_vote_id.inner());
    let resources = room
        .downcast_module_resource_storage()
        .get(filter)
        .await
        .unwrap();
    assert_eq!(resources.len(), 1);
    assert_json_snapshot!(resources[0].data, {
        ".entries[].timestamp" => "[timestamp]",
        ".entries[].event.parameters.legal_vote_id" => "[legal_vote_id]",
        ".entries[].event.parameters.start_time" => "[timestamp]",
        ".entries[].event.parameters.allowed_participants" => "[vec]",
        ".entries[].event.parameters.allowed_users" => "[vec]",
        ".entries[].event.token" => "[token]",
    }, @r#"
    {
      "entries": [
        {
          "event": {
            "event": "start",
            "issuer": "00000000-0000-0000-0000-0000000a11ce",
            "parameters": {
              "allowed_participants": "[vec]",
              "allowed_users": "[vec]",
              "auto_close": true,
              "create_pdf": true,
              "enable_abstain": false,
              "initiator_id": "00000000-0000-0000-0000-0000000a11ce",
              "legal_vote_id": "[legal_vote_id]",
              "live": false,
              "max_votes": 2,
              "name": "Vote",
              "pseudonymous": true,
              "start_time": "[timestamp]"
            }
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "user_left"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "user_joined"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "vote",
            "option": "no",
            "token": "[token]"
          }
        },
        {
          "event": {
            "by_user": "00000000-0000-0000-0000-0000000a11ce",
            "event": "stop"
          },
          "timestamp": "[timestamp]"
        },
        {
          "event": {
            "event": "final_results",
            "no": 1,
            "results": "valid",
            "yes": 0
          },
          "timestamp": "[timestamp]"
        }
      ],
      "version": 1
    }
    "#);

    // The pdf contains Bobs disconnect and reconnect events
    let asset_id = receive_pdf(&mut alice).await;

    let pdf = room.stored_asset(asset_id).await.unwrap();
    let content = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    let alice_token = tokens[&alice_id].unwrap();
    insta::with_settings!({filters => vec![
        (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}", "[timestamp]"),
        (r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}", "[legal_vote_id]"),
        (&format!("{alice_token}|{bob_token}"), "[token]"),
    ]}, {
        assert_snapshot!(content);
    });
}

#[test_log::test(tokio::test)]
async fn breakout_room() {
    let mut room = TestRoom::builder()
        .register_module::<LegalVoteModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
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
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice starts a vote in the breakout room
    let alice_id = alice.id();
    let (legal_vote_id, tokens) = start_vote(
        &mut alice,
        UserParameters {
            pseudonymous: false,
            live: true,
            name: Name::from_str("Vote").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([alice_id]).unwrap(),
            enable_abstain: false,
            auto_close: true,
            duration: None,
            create_pdf: false,
            timezone: None,
        },
        &mut [],
    )
    .await;

    // Bob deso not receive the started event because he is in the main room
    assert!(bob.received_nothing());

    // Alice votes
    let alice_token = tokens[&alice_id].unwrap();
    vote(&mut alice, legal_vote_id, VoteOption::Yes, alice_token).await;

    // Alice receives the updated event
    let event = alice
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Updated {
            legal_vote_id,
            results: Results {
                tally: Tally {
                    yes: 1,
                    no: 0,
                    abstain: None
                },
                voting_record: VotingRecord::UserVotes(HashMap::from_iter([(
                    alice.id(),
                    VoteOption::Yes
                )]))
            }
        }
    );

    // Bob does not receive the updated event because he is in the main room
    assert!(bob.received_nothing());

    // The vote automatically closes because auto_close is enabled
    let (stop_kind, results) = receive_stop_event(&mut alice, legal_vote_id, &mut []).await;
    assert_eq!(stop_kind, StopKind::Auto);
    assert_eq!(
        results,
        FinalResults::Valid(Results {
            tally: Tally {
                yes: 1,
                no: 0,
                abstain: None
            },
            voting_record: VotingRecord::UserVotes(HashMap::from_iter([(
                alice.id(),
                VoteOption::Yes
            )])),
        })
    );

    // Bob does not receive the stopped event because he is in the main room
    assert!(bob.received_nothing());
}

async fn start_vote(
    initiator: &mut MockParticipantJoined,
    parameters: UserParameters,
    others: &mut [&mut MockParticipantJoined],
) -> (LegalVoteId, HashMap<ParticipantId, Option<Token>>) {
    let expected_max_votes = parameters.allowed_participants.len() as u32;

    initiator
        .send_command::<LegalVoteModule>(LegalVoteCommand::Start(parameters.clone()), None)
        .await
        .unwrap();

    let initiator_start_event = initiator
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    let LegalVoteEvent::Started(Parameters {
        legal_vote_id,
        token,
        initiator_id,
        max_votes,
        inner,
        ..
    }) = &initiator_start_event
    else {
        panic!("Expected LegalVoteEvent::Started, got {initiator_start_event:?}");
    };
    assert_eq!(*initiator_id, initiator.id());
    assert_eq!(*max_votes, expected_max_votes);
    assert_eq!(*inner, parameters);

    let mut tokens = HashMap::from_iter([(initiator.id(), *token)]);
    for participant in others.iter_mut() {
        let event = participant
            .receive_event::<LegalVoteModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(*initiator_id, initiator.id());
        assert_eq!(*max_votes, expected_max_votes);
        assert_eq!(*inner, parameters);

        let LegalVoteEvent::Started(Parameters {
            legal_vote_id: id,
            token,
            ..
        }) = &event
        else {
            panic!("Expected LegalVoteEvent::Started, got {event:?}");
        };
        assert_eq!(id, legal_vote_id);

        tokens.insert(participant.id(), *token);
    }

    (*legal_vote_id, tokens)
}

async fn vote(
    participant: &mut MockParticipantJoined,
    legal_vote_id: LegalVoteId,
    option: VoteOption,
    token: Token,
) {
    participant
        .send_command::<LegalVoteModule>(
            LegalVoteCommand::Vote {
                legal_vote_id,
                option,
                token,
            },
            None,
        )
        .await
        .unwrap();

    let event = participant
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        LegalVoteEvent::Voted {
            legal_vote_id,
            vote_option: option,
            issuer: participant.id(),
            consumed_token: token,
        }
    );
}

async fn stop_vote(
    participant: &mut MockParticipantJoined,
    legal_vote_id: LegalVoteId,
    others: &mut [&mut MockParticipantJoined],
) -> FinalResults {
    participant
        .send_command::<LegalVoteModule>(LegalVoteCommand::Stop { legal_vote_id }, None)
        .await
        .unwrap();

    let (stop_kind, results) = receive_stop_event(participant, legal_vote_id, others).await;
    assert_eq!(stop_kind, StopKind::ByParticipant(participant.id()));

    results
}

async fn receive_stop_event(
    participant: &mut MockParticipantJoined,
    legal_vote_id: LegalVoteId,
    others: &mut [&mut MockParticipantJoined],
) -> (StopKind, FinalResults) {
    let event = participant
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;

    for other in others {
        let other_event = other
            .receive_event::<LegalVoteModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(event, other_event);
    }

    let LegalVoteEvent::Stopped {
        legal_vote_id: produced_id,
        kind,
        results,
        ..
    } = event
    else {
        panic!("Expected LegalVoteEvent::Stopped, got {event:?}");
    };

    assert_eq!(produced_id, legal_vote_id);

    (kind, results)
}

async fn receive_pdf(participant: &mut MockParticipantJoined) -> AssetId {
    let event = participant
        .receive_event::<LegalVoteModule>()
        .await
        .unwrap()
        .payload;
    let LegalVoteEvent::ReportGenerated { asset_id, .. } = event else {
        panic!("Expected LegalVoteEvent::PdfGenerated, got {event:?}");
    };
    asset_id
}
