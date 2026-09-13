// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{assert_matches, collections::BTreeSet, time::Duration};

use livekit::{RoomEvent, RoomOptions};
use livekit_api::services::room::RoomClient;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_test_util_livekit::{LIVEKIT_KEY, LIVEKIT_SECRET};
use opentalk_roomserver_types::{
    breakout::{
        breakout_config::{BreakoutConfig, BreakoutRoomConfig},
        event::BreakoutEvent,
    },
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::LiveKitState;
use opentalk_types_common::rooms::RoomId;
use tokio::time::sleep;

mod common;

/// Test that the JoinSuccess contains the access token for the LiveKit room.
#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_rooms_lifecycle() {
    let (_container, mut room, public_url) = common::build_room().await;
    let room_id = room.id();
    let livekit_client = RoomClient::with_api_key(&public_url, LIVEKIT_KEY, LIVEKIT_SECRET);

    let mut alice = room.join_alice_moderator(0).await;

    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let bob_livekit_state = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    // join livekit to ensure participant exists and can be muted.
    let (_bob_room, mut room_events) = livekit::Room::connect(
        &public_url,
        &bob_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();
    let connected = room_events.recv().await;
    assert_matches!(connected, Some(RoomEvent::Connected { .. }));

    // start breakout rooms
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

    let event = alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Breakout(0.into()))
        .await;

    let BreakoutEvent::SwitchedRoom { own_data, .. } = event else {
        panic!("Expected SwitchedRoom, got: {event:?}")
    };

    let alice_livekit_state = own_data
        .get::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present");

    // join livekit to ensure participant exists and can be muted.
    let (_bob_room, mut room_events) = livekit::Room::connect(
        &public_url,
        &alice_livekit_state.credentials.token,
        RoomOptions::default(),
    )
    .await
    .unwrap();
    let connected = room_events.recv().await;
    assert_matches!(connected, Some(RoomEvent::Connected { .. }));

    // livekit rooms should be created
    let room_list = get_rooms(&livekit_client, room_id).await;
    assert_eq!(
        room_list,
        BTreeSet::from([format!("{room_id}:main"), format!("{room_id}:0")])
    );

    // Stop breakout rooms, livekit rooms should be cleaned up
    alice
        .switch_breakout_room(&mut [&mut bob], RoomKind::Main)
        .await;
    alice.stop_breakout_rooms(&mut [&mut bob]).await;

    // wait until room is closed
    sleep(Duration::from_millis(100)).await;

    let room_list = get_rooms(&livekit_client, room_id).await;
    assert_eq!(room_list, BTreeSet::from([format!("{room_id}:main")]));

    alice.disconnect().await.unwrap();
    bob.disconnect().await.unwrap();

    // there is no way to wait for the room to be destroyed
}

/// Get rooms on the livekit server containing the `room_id`.
///
/// The rooms are filtered by `room_id` so that the tests can be executed concurrently and are more
/// robust against leaked rooms.
async fn get_rooms(livekit_client: &RoomClient, room_id: RoomId) -> BTreeSet<String> {
    let room_id_str = room_id.to_string();
    livekit_client
        .list_rooms(vec![])
        .await
        .unwrap()
        .into_iter()
        // ensure that this test can be executed on a shared livekit server by filtering other rooms
        // that were created by other tests
        .filter_map(|room| {
            if room.name.contains(&room_id_str) {
                Some(room.name)
            } else {
                None
            }
        })
        .collect()
}
