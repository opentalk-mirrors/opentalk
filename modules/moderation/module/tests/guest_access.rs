// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::assert_matches;

use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::{
    participant::MockParticipantJoined,
    room::{TestRoom, flush_connected_events},
};
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    core::CoreEvent,
    disconnect_reason::DisconnectReason,
    kick_reason::KickReason,
    room_kind::RoomKind,
    signaling::websocket::CloseFrame,
};
use opentalk_roomserver_types_moderation::{
    command::ModerationCommand,
    event::{ModerationError, ModerationEvent},
};

async fn disable_guest_access(
    sender: &mut MockParticipantJoined,
    registered: &mut [&mut MockParticipantJoined],
) {
    // Send the guest access disabled command
    sender
        .send_command::<ModerationModule>(ModerationCommand::DisableGuestAccess, None)
        .await
        .unwrap();

    // Receive the guest access disabled event
    let event = sender
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(ModerationEvent::GuestAccessDisabled, event);

    for participant in registered {
        let event = participant
            .receive_event::<ModerationModule>()
            .await
            .unwrap()
            .payload;
        assert_eq!(ModerationEvent::GuestAccessDisabled, event);
    }
}

#[test_log::test(tokio::test)]
async fn enable_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<ModerationModule>(ModerationCommand::EnableGuestAccess, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        ModerationEvent::Error(ModerationError::InsufficientPermissions),
        event
    );
}

#[test_log::test(tokio::test)]
async fn disable_insufficient_permissions() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut bob = room.join_bob(0).await;

    bob.send_command::<ModerationModule>(ModerationCommand::DisableGuestAccess, None)
        .await
        .unwrap();

    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        ModerationEvent::Error(ModerationError::InsufficientPermissions),
        event
    );
}

#[test_log::test(tokio::test)]
async fn disable() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    // Alice disables the guest access
    disable_guest_access(&mut alice, &mut [&mut bob]).await;

    // Gustav receives the kicked event
    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        ModerationEvent::Kicked {
            reason: KickReason::GuestAccessDisabled
        },
        event
    );

    // Gustav is removed from the meeting
    assert_eq!(
        gustav.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_owned(),
        })
    );

    // Alice receives the disconnect event for gustav
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason
        } if participant_id == gustav.id()
         && connection_id == gustav.connection_id()
         && reason == DisconnectReason::GuestAccessDisabled
    );

    // Bob receives the disconnect event for gustav
    let event = bob.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason
        } if participant_id == gustav.id()
         && connection_id == gustav.connection_id()
         && reason == DisconnectReason::GuestAccessDisabled
    );
}

#[test_log::test(tokio::test)]
async fn enable() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice disables the guest access
    disable_guest_access(&mut alice, &mut [&mut bob]).await;

    // Alice enables the guest access again
    alice
        .send_command::<ModerationModule>(ModerationCommand::EnableGuestAccess, None)
        .await
        .unwrap();

    // Alice receives the guest access enabled event
    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(ModerationEvent::GuestAccessEnabled, event);

    // Bob receives the guest access enabled event
    let event = bob
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(ModerationEvent::GuestAccessEnabled, event);
}

#[test_log::test(tokio::test)]
async fn affects_breakout_room() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;
    let mut gustav = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice, &mut bob]).await;

    let bob_id = bob.id();
    let gustav_id = gustav.id();
    alice
        .start_breakout_rooms(
            &mut [&mut bob, &mut gustav],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_owned(),
                    assignments: vec![bob_id, gustav_id],
                }],
                duration: None,
            },
        )
        .await;

    // Bob and Gustav switch to the breakout room
    bob.switch_breakout_room(&mut [&mut alice, &mut gustav], RoomKind::Breakout(0.into()))
        .await;
    gustav
        .switch_breakout_room(&mut [&mut alice, &mut bob], RoomKind::Breakout(0.into()))
        .await;

    // Alice disables guest access
    disable_guest_access(&mut alice, &mut [&mut bob]).await;

    // Gustav is kicked
    let event = gustav
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        ModerationEvent::Kicked {
            reason: KickReason::GuestAccessDisabled
        },
        event
    );
    assert_eq!(
        gustav.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_owned(),
        })
    );
}

#[test_log::test(tokio::test)]
async fn disable_kicks_guests_from_conference_and_lobby() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();

    // Alice joins the conference
    let mut alice = room.join_alice_moderator(0).await;

    // Gustav joins the conference with device 0
    let mut gustav_conference = room.join_gustav_guest().await;
    flush_connected_events(&mut [&mut alice]).await;

    // Gustav joins the lobby with device 1
    let mut gustav_lobby = room.lobby_gustav_guest().await;

    // Alice disables guest access
    disable_guest_access(&mut alice, &mut []).await;

    // Gustav receives the kicked event in the conference
    let event = gustav_conference
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        ModerationEvent::Kicked {
            reason: KickReason::GuestAccessDisabled
        },
        event
    );

    // Gustav receives the kicked event in the lobby
    let event = gustav_lobby
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        ModerationEvent::Kicked {
            reason: KickReason::GuestAccessDisabled
        },
        event
    );

    // Gustav's conference connection is closed
    assert_eq!(
        gustav_conference.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_owned(),
        })
    );

    // Gustav's lobby connection is closed
    assert_eq!(
        gustav_lobby.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_owned(),
        })
    );

    // Alice is only notified about Gustav disconnecting from the conference
    let event = alice.receive::<CoreEvent>().await.unwrap().payload;
    assert_matches!(
        event,
        CoreEvent::ParticipantDisconnected {
            participant_id,
            connection_id,
            reason
        } if participant_id == gustav_conference.id()
         && connection_id == gustav_conference.connection_id()
         && reason == DisconnectReason::GuestAccessDisabled
    );
    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn disable_kicks_lobby_only_guest() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();

    // Alice joins the conference
    let mut alice = room.join_alice_moderator(0).await;

    // Gustav joins only the lobby, without a conference connection
    let mut gustav_lobby = room.lobby_gustav_guest().await;

    // Alice disables guest access
    disable_guest_access(&mut alice, &mut []).await;

    // Gustav receives the kicked event in the lobby
    let event = gustav_lobby
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;
    assert_eq!(
        ModerationEvent::Kicked {
            reason: KickReason::GuestAccessDisabled
        },
        event
    );

    // Gustav's lobby connection is closed
    assert_eq!(
        gustav_lobby.receive_close_frame().await.unwrap(),
        Some(CloseFrame {
            code: 1000,
            reason: "closed by server".to_owned(),
        })
    );

    // The conference is not notified about a lobby-only participant disconnecting
    assert!(alice.received_nothing());
}

#[test_log::test(tokio::test)]
async fn services_are_exempt() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;
    let mut recorder = room.join_recorder(RoomKind::Main, 0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Alice disables guest access
    disable_guest_access(&mut alice, &mut [&mut recorder]).await;

    // The recorder is not kicked
    assert!(recorder.received_nothing());
}

#[test_log::test(tokio::test)]
async fn enable_already_enabled() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .guest_access(true)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::EnableGuestAccess, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;

    assert_eq!(ModerationEvent::GuestAccessEnabled, event);
}

#[test_log::test(tokio::test)]
async fn disable_already_disabled() {
    let mut room = TestRoom::builder()
        .register_module::<ModerationModule>()
        .guest_access(false)
        .spawn();
    let mut alice = room.join_alice_moderator(0).await;

    alice
        .send_command::<ModerationModule>(ModerationCommand::DisableGuestAccess, None)
        .await
        .unwrap();

    let event = alice
        .receive_event::<ModerationModule>()
        .await
        .unwrap()
        .payload;

    assert_eq!(ModerationEvent::GuestAccessDisabled, event);
}
