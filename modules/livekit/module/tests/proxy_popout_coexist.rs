// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::time::Duration;

use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::flush_connected_events;
use opentalk_roomserver_types::{
    livekit_proxy::{LiveKitProxyRequest, LiveKitProxyTarget},
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::{LiveKitCommand, LiveKitEvent, LiveKitState};
use tokio::sync::oneshot;

use crate::common::MockClientSocket;

mod common;

/// Establishes a proxied downstream socket for the given token and returns a receiver that resolves
/// (with an error) once the proxy for that socket shuts down.
async fn connect_proxied_socket(
    room: &opentalk_roomserver_room::mocking::room::TestRoom,
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
async fn livekit_popout_proxy_does_not_disconnect_main_proxy() {
    let (_container, mut room, _public_url) = common::build_room().await;

    let mut alice = room.join_alice_moderator(0).await;
    let mut bob = room.join_bob(0).await;
    flush_connected_events(&mut [&mut alice]).await;

    let participant_id = alice.id();
    let connection_id = alice.connection_id();

    // Main media token issued on join.
    let normal_token = alice
        .join_success()
        .get_module::<LiveKitState>()
        .expect("LiveKit state must be deserializable")
        .expect("LiveKit state must be present")
        .credentials
        .token
        .clone();

    // Request a popout stream token.
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

    let build_request = |token: &str| LiveKitProxyRequest {
        raw_query: Some(format!("access_token={token}")),
        headers: Default::default(),
        room_id: room.id(),
        proxy_target: LiveKitProxyTarget::LiveKit {
            room_kind: RoomKind::Main,
        },
        participant_id,
        connection_id,
    };

    // Establish the main media proxy first.
    let mut main_alive = connect_proxied_socket(&room, build_request(&normal_token)).await;

    // Establishing the popout proxy uses the same (participant, connection) key because the
    // web-api strips the `-popout` suffix from the token identity. This must not tear down the
    // already running main media proxy.
    let _popout_alive = connect_proxied_socket(&room, build_request(&popout_token)).await;

    // If the main proxy is still alive, its `MockClientSocket` is not dropped, so `main_alive`
    // stays pending and the timeout elapses. If the popout proxy evicted the main proxy, the
    // main socket is dropped and `main_alive` resolves immediately.
    let result = tokio::time::timeout(Duration::from_secs(5), &mut main_alive).await;
    assert!(
        result.is_err(),
        "the main media proxy was shut down when the popout stream proxy connected"
    );
}
