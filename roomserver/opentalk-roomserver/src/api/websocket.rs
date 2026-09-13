// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use axum::extract::ws::WebSocket;
use futures::{Sink, SinkExt as _, Stream, StreamExt};
use opentalk_roomserver_types::signaling::websocket::{
    Error, SignalingSink, SignalingSocketItem, SignalingSocketMessage, SignalingStream,
};

/// Maps incoming and outgoing websocket messages from [`axum::extract::ws::Message`] to
/// [`SignalingSocketItem`]/[`SignalingSocketMessage`].
#[derive(Debug)]
pub struct WebSocketAdapter {
    inner: WebSocket,
}

impl WebSocketAdapter {
    pub fn new(inner: WebSocket) -> Self {
        Self { inner }
    }
}

impl From<WebSocketAdapter> for WebSocket {
    fn from(value: WebSocketAdapter) -> Self {
        value.inner
    }
}

impl SignalingSink for WebSocketAdapter {}
impl SignalingStream for WebSocketAdapter {}

impl Sink<SignalingSocketMessage> for WebSocketAdapter {
    type Error = Error;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready_unpin(cx).map_err(Error::from)
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: SignalingSocketMessage,
    ) -> Result<(), Self::Error> {
        self.inner
            .start_send_unpin(item.into())
            .map_err(Error::from)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_flush_unpin(cx).map_err(Error::from)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_close_unpin(cx).map_err(Error::from)
    }
}

impl Stream for WebSocketAdapter {
    type Item = Result<SignalingSocketItem, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            std::task::Poll::Ready(Some(Ok(item))) => {
                std::task::Poll::Ready(Some(Ok(SignalingSocketItem {
                    message: item.into(),
                    done: None,
                })))
            }
            std::task::Poll::Ready(Some(Err(e))) => {
                std::task::Poll::Ready(Some(Err(Error::from(e))))
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
