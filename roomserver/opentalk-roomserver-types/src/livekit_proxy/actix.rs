// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use actix_ws::Message;
use futures::{Sink, SinkExt, Stream};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::sync::PollSender;

use crate::{
    livekit_proxy::websocket::{Error, LiveKitSink, LiveKitSocketMessage, LiveKitStream},
    signaling::continuation_buffer::ContinuationBuffer,
};

#[derive(Debug)]
pub struct LiveKitSocketAdapter {
    incoming: Receiver<Result<Message, Error>>,
    outgoing: PollSender<LiveKitSocketMessage>,
    continuation_buffer: ContinuationBuffer,
}

impl LiveKitSocketAdapter {
    pub fn new(
        incoming: Receiver<Result<Message, Error>>,
        outgoing: Sender<LiveKitSocketMessage>,
    ) -> Self {
        Self {
            incoming,
            outgoing: PollSender::new(outgoing),
            continuation_buffer: ContinuationBuffer::Empty,
        }
    }
}

impl LiveKitStream for LiveKitSocketAdapter {}
impl LiveKitSink for LiveKitSocketAdapter {}

impl Sink<LiveKitSocketMessage> for LiveKitSocketAdapter {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.outgoing.poll_ready_unpin(cx).map_err(Into::into)
    }

    fn start_send(mut self: Pin<&mut Self>, item: LiveKitSocketMessage) -> Result<(), Self::Error> {
        self.outgoing.start_send_unpin(item).map_err(Into::into)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.outgoing.poll_flush_unpin(cx).map_err(Into::into)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.outgoing.poll_close_unpin(cx).map_err(Into::into)
    }
}

impl Stream for LiveKitSocketAdapter {
    type Item = Result<LiveKitSocketMessage, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Loop to skip over messages that don't produce output, such as Message::Continuation or
        // Message::Nop. We must re-poll the receiver rather than returning Poll::Pending, because
        // poll_recv already consumed its waker registration when it returned Ready.
        loop {
            match self.incoming.poll_recv(cx) {
                Poll::Ready(Some(Ok(message))) => {
                    match LiveKitSocketMessage::from_actix_message(
                        message,
                        &mut self.continuation_buffer,
                    ) {
                        Some(result) => return Poll::Ready(Some(result)),
                        // Skip messages that don't produce output (Message::Continuation,
                        // Message::Nop).
                        None => continue,
                    }
                }
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
