// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Sink, Stream};
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_room::mocking::room::TestRoom;
use opentalk_roomserver_test_util_livekit as test_util;
use opentalk_roomserver_types::livekit_proxy::websocket::{
    Error, LiveKitSink, LiveKitSocketMessage, LiveKitStream,
};
use opentalk_types_common::rooms::RoomId;
use tokio::sync::oneshot;

#[allow(unused)]
pub async fn build_room() -> (test_util::ContainerGuard, TestRoom, String) {
    let (container, settings) = test_util::create_livekit_container().await;

    let room = TestRoom::builder()
        .room_id(RoomId::generate())
        .register_module::<LiveKitModule>()
        .add_init_module_data(&settings)
        .unwrap()
        .spawn();

    (container, room, settings.public_url)
}

/// A minimal downstream socket used to drive the proxy from tests.
///
/// It never yields inbound messages and silently discards outbound ones. Dropping it (which
/// happens when the proxy task shuts down) closes `_alive`, allowing the test to observe whether
/// the associated proxy is still running.
#[derive(Debug)]
#[allow(unused)]
pub struct MockClientSocket {
    pub _alive: oneshot::Sender<()>,
}

impl Stream for MockClientSocket {
    type Item = Result<LiveKitSocketMessage, Error>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

impl Sink<LiveKitSocketMessage> for MockClientSocket {
    type Error = Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, _item: LiveKitSocketMessage) -> Result<(), Self::Error> {
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl LiveKitStream for MockClientSocket {}
impl LiveKitSink for MockClientSocket {}
