// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use futures::{Sink, SinkExt, Stream};
use opentalk_roomserver_types::signaling::websocket::{
    Error as SignalingError, SignalingSink, SignalingSocketItem, SignalingSocketMessage,
    SignalingStream,
};
use tokio::sync::mpsc;
use tokio_util::sync::PollSender;

#[derive(Debug)]
pub struct MockSocket {
    receiver: mpsc::Receiver<Result<SignalingSocketItem, SignalingError>>,
    sender: PollSender<SignalingSocketMessage>,
}

impl MockSocket {
    #[must_use]
    pub fn new(
        receiver: mpsc::Receiver<Result<SignalingSocketItem, SignalingError>>,
        sender: mpsc::Sender<SignalingSocketMessage>,
    ) -> MockSocket {
        MockSocket {
            receiver,
            sender: PollSender::new(sender),
        }
    }
}

impl Stream for MockSocket {
    type Item = Result<SignalingSocketItem, SignalingError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

impl Sink<SignalingSocketMessage> for MockSocket {
    type Error = SignalingError;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.sender
            .poll_ready_unpin(cx)
            .map_err(SignalingError::from)
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: SignalingSocketMessage,
    ) -> Result<(), Self::Error> {
        self.sender
            .start_send_unpin(item)
            .map_err(SignalingError::from)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.sender
            .poll_flush_unpin(cx)
            .map_err(SignalingError::from)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.sender
            .poll_close_unpin(cx)
            .map_err(SignalingError::from)
    }
}

impl SignalingSink for MockSocket {}
impl SignalingStream for MockSocket {}
