// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::ParticipantId;

#[test_log::test(tokio::test)]
async fn insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut bob]).await;

    // Bob can't change Alice's display name
    bob.send_command::<ModerationModule>(
        ModerationCommand::ChangeDisplayName {
            new_name: DisplayName::from_str_lossy("Charlie"),
            target: gustav.id(),
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
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );

    assert!(gustav.received_nothing());
}

#[test_log::test(tokio::test)]
async fn display_name_too_short() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy(""),
                target: gustav.id(),
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
        ModerationEvent::Error(ModerationError::InvalidDisplayName)
    );

    assert!(gustav.received_nothing());
}

#[test_log::test(tokio::test)]
async fn display_name_too_long() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy(&str::repeat("x", 256)),
                target: gustav.id(),
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
        ModerationEvent::Error(ModerationError::InvalidDisplayName)
    );

    assert!(gustav.received_nothing());
}

#[test_log::test(tokio::test)]
async fn unknown_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Bob"),
                target: ParticipantId::nil(),
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
        ModerationEvent::Error(ModerationError::UnknownParticipant)
    );
}

#[test_log::test(tokio::test)]
async fn cannot_change_name_of_registered_users() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Bob can't change Alice's display name
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Charlie"),
                target: bob.id(),
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
        ModerationEvent::Error(ModerationError::CannotChangeNameOfRegisteredUsers)
    );

    assert!(bob.received_nothing());
}

#[test_log::test(tokio::test)]
async fn change_display_name() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    // Bob can't change Alice's display name
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Bob"),
                target: gustav.id(),
            },
            None,
        )
        .await
        .unwrap();

    let expected = ModerationEvent::DisplayNameChanged {
        target: gustav.id(),
        issued_by: alice.id(),
        old_name: DisplayName::from_str_lossy("Gustav the great"),
        new_name: DisplayName::from_str_lossy("Bob"),
    };
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, expected.clone());

    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(event, expected);
}

#[test_log::test(tokio::test)]
async fn lobby_participant_receive_display_name_change_event() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut gustav_lobby = room.lobby_gustav_guest().await;

    // Gustav joins the conference with another connection
    let mut gustav_conference = room.join_gustav_guest().await;

    // Gustav's lobby connection is notified that the display name was set
    let event = gustav_lobby
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameAssigned {
            new_name: Some(gustav_conference.display_name().clone())
        }
    );

    // Gustav changes his display name in the conference
    let old_name = gustav_conference.display_name().clone();
    let new_name = DisplayName::from_str_lossy("Gus");
    gustav_conference
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: new_name.clone(),
                target: gustav_conference.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Gustav's conference connection is receives the display name changed event
    let event = gustav_conference
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChanged {
            target: gustav_conference.id(),
            issued_by: gustav_conference.id(),
            old_name,
            new_name: new_name.clone(),
        }
    );

    // Gustav's conference connection received no further events
    assert!(gustav_conference.received_nothing());

    // Gustav's lobby connection is also notified that the display name changed
    let event = gustav_lobby
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameAssigned {
            new_name: Some(new_name)
        }
    );

    // Gustav disconnects from the conference
    gustav_conference.disconnect().await.unwrap();

    // Gustav's lobby connecetion is notified that the display name was cleared
    let event = gustav_lobby
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameAssigned { new_name: None }
    );

    // Gustav can now choose a new display name when entering from the lobby
    let display_name = DisplayName::from_str_lossy("Gustavo");
    let gustav = gustav_lobby
        .enter_room(Some(display_name.clone()))
        .await
        .unwrap();
    assert_eq!(gustav.join_success().display_name, display_name);
}
