// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use actix_ws::{Message, ProtocolError};
use futures::{Sink, SinkExt, Stream};
use opentalk_roomserver_types::signaling::{
    continuation_buffer::ContinuationBuffer,
    websocket::{
        self, SignalingSink, SignalingSocketItem, SignalingSocketMessage, SignalingStream,
    },
};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::PollSender;

#[derive(Debug)]
pub struct WebSocketAdapter {
    incoming: Receiver<Result<Message, ProtocolError>>,
    outgoing: PollSender<SignalingSocketMessage>,
    continuation_buffer: ContinuationBuffer,
}

impl WebSocketAdapter {
    pub fn new(
        incoming: Receiver<Result<Message, ProtocolError>>,
        outgoing: Sender<SignalingSocketMessage>,
    ) -> Self {
        Self {
            incoming,
            outgoing: PollSender::new(outgoing),
            continuation_buffer: ContinuationBuffer::Empty,
        }
    }
}

impl SignalingSink for WebSocketAdapter {}
impl SignalingStream for WebSocketAdapter {}

impl Sink<SignalingSocketMessage> for WebSocketAdapter {
    type Error = websocket::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.outgoing.poll_ready_unpin(cx).map_err(Into::into)
    }

    fn start_send(
        mut self: Pin<&mut Self>,
        item: SignalingSocketMessage,
    ) -> Result<(), Self::Error> {
        self.outgoing.start_send_unpin(item).map_err(Into::into)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.outgoing.poll_flush_unpin(cx).map_err(Into::into)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.outgoing.poll_close_unpin(cx).map_err(Into::into)
    }
}

impl Stream for WebSocketAdapter {
    type Item = Result<SignalingSocketItem, websocket::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.incoming.poll_recv(cx) {
            Poll::Ready(Some(Ok(message))) => Poll::Ready(SignalingSocketItem::from_actix_message(
                message,
                &mut self.continuation_buffer,
            )),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
