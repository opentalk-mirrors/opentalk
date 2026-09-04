// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_room::mocking::room::{TestRoom, flush_connected_events};
use opentalk_roomserver_test_util_livekit as test_util;
use opentalk_roomserver_types::{
    livekit_proxy::{LiveKitProxyRequest, LiveKitProxyTarget},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::LiveKitState;
use opentalk_roomserver_types_moderation::command::ModerationCommand;
use opentalk_types_common::rooms::RoomId;
use tokio::sync::oneshot;

use crate::common::MockClientSocket;

mod common;

/// Establishes a proxied downstream socket for the given request and returns a receiver that
/// resolves (with an error) once the proxy for that socket shuts down.
async fn connect_proxied_socket(
    room: &TestRoom,
    request: LiveKitProxyRequest,
) -> oneshot::Receiver<()> {
    let upstream = room
        .room_handle
        .prepare_proxy_socket(request.clone())
        .await
        .expect("failed to prepare upstream proxy socket");

    let (alive_tx, alive_rx) = oneshot::channel();
    room.room_handle
        .accept_livekit_socket(
            request,
            upstream,
            Box::new(MockClientSocket { _alive: alive_tx }),
        )
        .await
        .expect("failed to accept downstream proxy socket");

    alive_rx
}

#[test_log::test(tokio::test)]
// The `livekit_` prefix ensures that tests that require the livekit server can be grouped by name
async fn livekit_moving_to_waiting_room_disconnects_proxy() {
    let (_container, settings) = test_util::create_livekit_container().await;

    let mut room = TestRoom::builder()
        .room_id(RoomId::generate())
        .register_module::<LiveKitModule>()
        .register_module::<ModerationModule>()
        .add_init_module_data(&settings)
        .unwrap()
        .spawn();

    let mut alice = room.join_alice_moderator(0).await;
    let bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let participant_id = bob.id();
    let connection_id = bob.connection_id();

    // Media token issued to Bob on join.
    let bob_token = bob
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present")
        .credentials
        .token
        .clone();

    let request = LiveKitProxyRequest {
        raw_query: Some(format!("access_token={bob_token}")),
        headers: Default::default(),
        room_id: room.id(),
        proxy_target: LiveKitProxyTarget::LiveKit {
            room_kind: RoomKind::Main,
        },
        participant_id,
        connection_id,
    };

    // Establish Bob's media proxy while he is in the conference.
    let mut bob_alive = connect_proxied_socket(&room, request).await;

    // The proxy must stay alive as long as Bob remains in the conference.
    let still_alive = tokio::time::timeout(Duration::from_millis(500), &mut bob_alive).await;
    assert!(
        still_alive.is_err(),
        "the media proxy was shut down before Bob was moved to the waiting room"
    );

    // A moderator moves Bob to the waiting room.
    alice
        .send_command::<ModerationModule>(
            ModerationCommand::SendToWaitingRoom {
                target: participant_id,
            },
            None,
        )
        .await
        .unwrap();

    // Being moved to the waiting room disconnects Bob's conference connection, which must tear down
    // his media proxy. When the proxy shuts down, its `MockClientSocket` is dropped and `bob_alive`
    // resolves. If the proxy stayed up, `bob_alive` stays pending and the timeout elapses.
    let result = tokio::time::timeout(Duration::from_secs(5), &mut bob_alive).await;
    assert!(
        result.is_ok(),
        "the media proxy was not shut down when Bob was moved to the waiting room"
    );
}
