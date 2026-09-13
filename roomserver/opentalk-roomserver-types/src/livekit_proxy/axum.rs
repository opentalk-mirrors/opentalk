// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use axum::extract::ws::WebSocket;
use futures::{Sink, SinkExt as _, Stream, StreamExt};

use crate::livekit_proxy::websocket::{Error, LiveKitSink, LiveKitSocketMessage, LiveKitStream};

/// Maps incoming and outgoing websocket messages from [`axum::extract::ws::Message`] to
/// [`LiveKitSocketMessage`].
#[derive(Debug)]
pub struct LiveKitSocketAdapter {
    inner: WebSocket,
}

impl LiveKitSocketAdapter {
    pub fn new(inner: WebSocket) -> Self {
        Self { inner }
    }
}

impl From<LiveKitSocketAdapter> for WebSocket {
    fn from(value: LiveKitSocketAdapter) -> Self {
        value.inner
    }
}

impl LiveKitStream for LiveKitSocketAdapter {}
impl LiveKitSink for LiveKitSocketAdapter {}

impl Sink<LiveKitSocketMessage> for LiveKitSocketAdapter {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready_unpin(cx).map_err(Error::from)
    }

    fn start_send(mut self: Pin<&mut Self>, item: LiveKitSocketMessage) -> Result<(), Self::Error> {
        self.inner
            .start_send_unpin(item.into())
            .map_err(Error::from)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_flush_unpin(cx).map_err(Error::from)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_close_unpin(cx).map_err(Error::from)
    }
}

impl Stream for LiveKitSocketAdapter {
    type Item = Result<LiveKitSocketMessage, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(item))) => Poll::Ready(Some(Ok(item.into()))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(Error::from(e)))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
