// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use livekit::{DataPacket, RoomOptions};
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types::{
    breakout::breakout_config::{BreakoutConfig, BreakoutRoomConfig},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::{LiveKitCommand, LiveKitEvent, LiveKitState};

mod common;

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_request_access_token() {
    let (_container, mut room, public_url) = common::build_room().await;

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .send_command::<LiveKitModule>(LiveKitCommand::RequestPopoutStreamAccessToken, None)
        .await
        .unwrap();
    let token_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert!(bob.received_nothing());

    if let LiveKitEvent::PopoutStreamAccessToken { token } = token_event.payload {
        // join livekit to ensure we got the token for the correct room
        let (alice_room, _room_events) =
            livekit::Room::connect(&public_url, &token, RoomOptions::default())
                .await
                .unwrap();
        assert_eq!(alice_room.name(), format!("{}:main", room.id()));
    } else {
        panic!("Expected PopoutStreamAccessToken event, got: {token_event:?}");
    }
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_alice_in_breakout_bob_in_main() {
    let (_container, mut room, public_url) = common::build_room().await;

    // Alice and Bob join the meeting
    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    alice
        .start_breakout_rooms(
            &mut [&mut bob],
            BreakoutConfig {
                rooms: vec![BreakoutRoomConfig {
                    name: "Room 0".to_string(),
                    assignments: vec![],
                }],
                duration: None,
            },
        )
        .await;

    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    alice
        .send_command::<LiveKitModule>(LiveKitCommand::RequestPopoutStreamAccessToken, None)
        .await
        .unwrap();
    let token_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert!(bob.received_nothing());

    if let LiveKitEvent::PopoutStreamAccessToken { token } = token_event.payload {
        // join livekit to ensure we got the token for the correct room
        let (alice_room, _room_events) =
            livekit::Room::connect(&public_url, &token, RoomOptions::default())
                .await
                .unwrap();
        assert_eq!(alice_room.name(), format!("{}:0", room.id()));
    } else {
        panic!("Expected PopoutStreamAccessToken event, got: {token_event:?}");
    }
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_popout_and_normal_connection_coexist() {
    let (_container, mut room, public_url) = common::build_room().await;

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    // Get the normal credentials that were issued on join
    let livekit_state = alice
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");
    let normal_token = livekit_state.credentials.token.clone();

    alice
        .send_command::<LiveKitModule>(LiveKitCommand::RequestPopoutStreamAccessToken, None)
        .await
        .unwrap();
    let token_event = alice.receive_event::<LiveKitModule>().await.unwrap();
    assert!(bob.received_nothing());

    let LiveKitEvent::PopoutStreamAccessToken {
        token: popout_token,
    } = token_event.payload
    else {
        panic!("Expected PopoutStreamAccessToken event, got: {token_event:?}");
    };

    let expected_room_name = format!("{}:main", room.id());

    // Establish both connections simultaneously
    let (normal_room, _normal_events) =
        livekit::Room::connect(&public_url, &normal_token, RoomOptions::default())
            .await
            .unwrap();
    let (popout_room, _popout_events) =
        livekit::Room::connect(&public_url, &popout_token, RoomOptions::default())
            .await
            .unwrap();

    // Both connections must be in the same LiveKit room
    assert_eq!(normal_room.name(), expected_room_name);
    assert_eq!(popout_room.name(), expected_room_name);

    // Sending data confirms both connections are alive (not just cached state)
    let probe = DataPacket {
        reliable: true,
        ..DataPacket::default()
    };
    normal_room
        .local_participant()
        .publish_data(probe.clone())
        .await
        .unwrap();
    popout_room
        .local_participant()
        .publish_data(probe)
        .await
        .unwrap();
}
