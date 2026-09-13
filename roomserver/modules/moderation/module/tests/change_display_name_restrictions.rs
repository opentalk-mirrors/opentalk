// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashSet;

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
};
use opentalk_types_common::users::DisplayName;

#[test_log::test(tokio::test)]
async fn restricted_participants_cant_change_display_name() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice enables display name change restrictions
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableDisplayNameChangeRestrictions {
                unrestricted_participants: HashSet::new(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice and Gustav receive the display name restrictions enabled event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    // Gustav tries to change his display name
    gustav
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Gus Gunslinger"),
                target: gustav.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Gustav receives an error because he is restricted
    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );

    // Alice doesn't receive any event
    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn moderators_can_always_change_display_names() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice enables display name change restrictions
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableDisplayNameChangeRestrictions {
                unrestricted_participants: HashSet::new(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice and Gustav receive the display name restrictions enabled event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    // Alice changes Gustav's display name
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Gus Gunslinger"),
                target: gustav.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice and Gustav receive the display name changed event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChanged {
            target: gustav.id(),
            issued_by: alice.id(),
            old_name: DisplayName::from_str_lossy("Gustav the great"),
            new_name: DisplayName::from_str_lossy("Gus Gunslinger")
        }
    );

    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChanged {
            target: gustav.id(),
            issued_by: alice.id(),
            old_name: DisplayName::from_str_lossy("Gustav the great"),
            new_name: DisplayName::from_str_lossy("Gus Gunslinger")
        }
    );
}

#[test_log::test(tokio::test)]
async fn unrestricted_participants_can_change_display_name() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice enables display name change restrictions, but exempts Gustav
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableDisplayNameChangeRestrictions {
                unrestricted_participants: HashSet::from_iter([gustav.id()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice and Gustav receive the display name restrictions enabled event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::from_iter([gustav.id()])
        }
    );

    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::from_iter([gustav.id()])
        }
    );

    // Gustav changes his display name
    gustav
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Gus Gunslinger"),
                target: gustav.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Gustav is allowed to change his display name because he is exempted
    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChanged {
            target: gustav.id(),
            issued_by: gustav.id(),
            old_name: DisplayName::from_str_lossy("Gustav the great"),
            new_name: DisplayName::from_str_lossy("Gus Gunslinger")
        }
    );

    // Alice also receives the display name changed event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;

    assert_eq!(
        event,
        ModerationEvent::DisplayNameChanged {
            target: gustav.id(),
            issued_by: gustav.id(),
            old_name: DisplayName::from_str_lossy("Gustav the great"),
            new_name: DisplayName::from_str_lossy("Gus Gunslinger")
        }
    );
}

#[test_log::test(tokio::test)]
async fn exempt_unknown_participant() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    // Alice enables display name change restrictions exempting an unknown participant
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableDisplayNameChangeRestrictions {
                unrestricted_participants: HashSet::from_iter([0xdeadbeef.into()]),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives a successful response. Exempting an unknown participant is allowed, as they
    // might join later.
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::from_iter([0xdeadbeef.into()])
        }
    );
}

#[test_log::test(tokio::test)]
async fn enable_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Bob tries to enable display name change restrictions
    bob.send_command::<ModerationModule>(
        ModerationCommand::EnableDisplayNameChangeRestrictions {
            unrestricted_participants: HashSet::new(),
        },
        None,
    )
    .await
    .unwrap();

    // Bob receives an error because he isn't a moderator
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
async fn disable_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice enables display name change restrictions
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableDisplayNameChangeRestrictions {
                unrestricted_participants: HashSet::new(),
            },
            None,
        )
        .await
        .unwrap();

    // Bob gets notified that the display name change has been restricted
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    // Bob tries to disable display name change restrictions
    bob.send_command::<ModerationModule>(ModerationCommand::DisableMicrophoneRestrictions, None)
        .await
        .unwrap();

    // Bob receives an error because he isn't a moderator
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
async fn alice_in_breakout_gustav_in_main() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut gustav],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_string(),
                    assignments: vec![alice.id()],
                }],
                duration: None,
            },
        )
        .await;

    // Alice switches to the breakout room while Gustav remains in the main room
    alice
        .switch_breakout_room(&mut [&mut gustav], RoomKind::Breakout(0.into()))
        .await;

    // Alice enables display name change restrictions
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::EnableDisplayNameChangeRestrictions {
                unrestricted_participants: HashSet::new(),
            },
            None,
        )
        .await
        .unwrap();

    // Alice receives the display name restrictions enabled event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    // Gustav also receives the display name restrictions enabled event because the setting is
    // global
    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::DisplayNameChangeRestrictionsEnabled {
            unrestricted_participants: HashSet::new()
        }
    );

    // Gustav tries to change his display name
    gustav
        .send_command::<ModerationModule>(
            ModerationCommand::ChangeDisplayName {
                new_name: DisplayName::from_str_lossy("Gus Gunslinger"),
                target: gustav.id(),
            },
            None,
        )
        .await
        .unwrap();

    // Gustav receives an error because he is restricted
    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        event,
        ModerationEvent::Error(ModerationError::InsufficientPermissions)
    );

    // Alice doesn't receive any event
    assert!(alice.received_nothing());
}
