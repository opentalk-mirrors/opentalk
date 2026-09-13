// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Debug;

use bytes::Bytes;
use futures::{
    Sink, Stream,
    stream::{Peekable, SplitSink, SplitStream},
};
use tokio_util::sync::PollSendError;

#[derive(Debug, thiserror::Error)]
#[error("Websocket error")]
pub struct Error {
    #[from]
    source: Box<dyn std::error::Error + Send + Sync>,
}

#[cfg(feature = "axum")]
impl From<axum::Error> for Error {
    fn from(value: axum::Error) -> Self {
        Self {
            source: Box::new(value),
        }
    }
}

#[cfg(feature = "actix")]
impl From<actix_ws::ProtocolError> for Error {
    fn from(value: actix_ws::ProtocolError) -> Self {
        Self {
            source: Box::new(value),
        }
    }
}

#[cfg(feature = "actix")]
impl From<crate::signaling::continuation_buffer::ContinuationError> for Error {
    fn from(value: crate::signaling::continuation_buffer::ContinuationError) -> Self {
        Self {
            source: Box::new(value),
        }
    }
}

impl From<PollSendError<LiveKitSocketMessage>> for Error {
    fn from(value: PollSendError<LiveKitSocketMessage>) -> Self {
        Self {
            source: Box::new(value),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

#[cfg(feature = "axum")]
impl From<CloseFrame> for axum::extract::ws::CloseFrame {
    fn from(value: CloseFrame) -> Self {
        Self {
            code: value.code,
            reason: value.reason.into(),
        }
    }
}

#[cfg(feature = "axum")]
impl From<axum::extract::ws::CloseFrame> for CloseFrame {
    fn from(value: axum::extract::ws::CloseFrame) -> Self {
        Self {
            code: value.code,
            reason: value.reason.to_string(),
        }
    }
}

#[cfg(feature = "actix")]
impl From<CloseFrame> for actix_ws::CloseReason {
    fn from(value: CloseFrame) -> Self {
        let description = if value.reason.is_empty() {
            None
        } else {
            Some(value.reason)
        };

        Self {
            code: value.code.into(),
            description,
        }
    }
}

#[cfg(feature = "actix")]
impl From<actix_ws::CloseReason> for CloseFrame {
    fn from(value: actix_ws::CloseReason) -> Self {
        Self {
            code: value.code.into(),
            reason: value.description.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LiveKitSocketMessage {
    Text(String),
    Binary(Bytes),
    Ping(Bytes),
    Pong(Bytes),
    Close(Option<CloseFrame>),
}

impl From<String> for LiveKitSocketMessage {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

#[cfg(feature = "axum")]
impl From<LiveKitSocketMessage> for axum::extract::ws::Message {
    fn from(value: LiveKitSocketMessage) -> Self {
        use axum::extract::ws::Message;
        match value {
            LiveKitSocketMessage::Text(text) => Message::Text(text.into()),
            LiveKitSocketMessage::Binary(bytes) => Message::Binary(bytes),
            LiveKitSocketMessage::Ping(bytes) => Message::Ping(bytes),
            LiveKitSocketMessage::Pong(bytes) => Message::Pong(bytes),
            LiveKitSocketMessage::Close(close_frame) => Message::Close(close_frame.map(Into::into)),
        }
    }
}

#[cfg(feature = "axum")]
impl From<axum::extract::ws::Message> for LiveKitSocketMessage {
    fn from(value: axum::extract::ws::Message) -> Self {
        use axum::extract::ws::Message;
        match value {
            Message::Text(text) => LiveKitSocketMessage::Text(text.to_string()),
            Message::Binary(bytes) => LiveKitSocketMessage::Binary(bytes),
            Message::Ping(bytes) => LiveKitSocketMessage::Ping(bytes),
            Message::Pong(bytes) => LiveKitSocketMessage::Pong(bytes),
            Message::Close(close_frame) => LiveKitSocketMessage::Close(close_frame.map(Into::into)),
        }
    }
}

#[cfg(feature = "actix")]
impl From<crate::signaling::continuation_buffer::ContinuationMessage> for LiveKitSocketMessage {
    fn from(value: crate::signaling::continuation_buffer::ContinuationMessage) -> Self {
        use crate::signaling::continuation_buffer::ContinuationMessage;

        match value {
            ContinuationMessage::Text(text) => Self::Text(text),
            ContinuationMessage::Binary(bytes) => Self::Binary(bytes),
        }
    }
}

#[cfg(feature = "actix")]
impl LiveKitSocketMessage {
    pub fn from_actix_message(
        message: actix_ws::Message,
        continuation_buffer: &mut crate::signaling::continuation_buffer::ContinuationBuffer,
    ) -> Option<Result<Self, Error>> {
        match message {
            actix_ws::Message::Text(byte_string) => {
                Some(Ok(LiveKitSocketMessage::Text(byte_string.to_string())))
            }
            actix_ws::Message::Binary(bytes) => Some(Ok(LiveKitSocketMessage::Binary(bytes))),
            actix_ws::Message::Continuation(item) => continuation_buffer
                .extend(item)
                .map(|result| result.map(Self::from).map_err(Error::from)),
            actix_ws::Message::Ping(bytes) => Some(Ok(LiveKitSocketMessage::Ping(bytes))),
            actix_ws::Message::Pong(bytes) => Some(Ok(LiveKitSocketMessage::Pong(bytes))),
            actix_ws::Message::Close(close_reason) => Some(Ok(LiveKitSocketMessage::Close(
                close_reason.map(CloseFrame::from),
            ))),
            actix_ws::Message::Nop => None,
        }
    }
}

/// A stream of messages for a single signaling connection
pub trait LiveKitStream: Stream<Item = Result<LiveKitSocketMessage, Error>> + Send + Unpin {}
impl<S: LiveKitStream> LiveKitStream for Peekable<S> {}
impl<S: LiveKitSocket> LiveKitStream for SplitStream<S> {}

/// A sink for outgoing messages of a single signaling connection
pub trait LiveKitSink: Sink<LiveKitSocketMessage, Error = Error> + Send + Unpin {}
impl<S: LiveKitSocket> LiveKitSink for SplitSink<S, LiveKitSocketMessage> {}

/// A socket implementing both [`LiveKitSink`] and [`LiveKitStream`].
pub trait LiveKitSocket: LiveKitStream + LiveKitSink + Debug {}
impl<S: LiveKitSink + LiveKitStream + Debug> LiveKitSocket for S {}
