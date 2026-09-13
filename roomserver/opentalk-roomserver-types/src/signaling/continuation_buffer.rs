// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::string::FromUtf8Error;

use bytes::{BufMut as _, Bytes, BytesMut};
use derive_more::{Display, Error};

#[cfg(not(test))]
/// The maximum allowed message size (64MiB)
const MAX_MESSAGE_SIZE: usize = 64 * 1024 * 1024;
#[cfg(test)]
const MAX_MESSAGE_SIZE: usize = 64;

/// The result of a completed WebSocket continuation sequence.
///
/// This is a generic intermediate representation that can be converted
/// into specific message types like `SignalingSocketItem` or `LiveKitSocketMessage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinuationMessage {
    Text(String),
    Binary(Bytes),
}

/// A buffer for handling actix WebSocket message continuations.
#[derive(Debug, Default, PartialEq, Eq)]
pub enum ContinuationBuffer {
    #[default]
    Empty,
    #[allow(private_interfaces)]
    Text(LimitedBytesMut),
    #[allow(private_interfaces)]
    Binary(LimitedBytesMut),
}

impl ContinuationBuffer {
    /// Extends the buffer with a new [`actix_ws::Item`] and returns the resulting websocket
    /// message. If the item completes a message, it returns `Some(Ok(ContinuationMessage))`. If
    /// an error occurs, it returns `Some(Err(ContinuationError))` and resets the buffer to empty.
    /// If the item is part of an ongoing message but does not complete it, it returns [`None`].
    pub fn extend(
        &mut self,
        item: actix_ws::Item,
    ) -> Option<Result<ContinuationMessage, ContinuationError>> {
        match self.extend_inner(item) {
            Err(err) => {
                *self = ContinuationBuffer::Empty;
                Some(Err(err))
            }
            other => other.transpose(),
        }
    }

    fn extend_inner(
        &mut self,
        item: actix_ws::Item,
    ) -> Result<Option<ContinuationMessage>, ContinuationError> {
        match (&mut *self, item) {
            (ContinuationBuffer::Empty, actix_ws::Item::FirstText(bytes)) => {
                let limited_bytes = LimitedBytesMut::new(bytes)?;
                *self = ContinuationBuffer::Text(limited_bytes);

                Ok(None)
            }
            (ContinuationBuffer::Empty, actix_ws::Item::FirstBinary(bytes)) => {
                let limited_bytes = LimitedBytesMut::new(bytes)?;
                *self = ContinuationBuffer::Binary(limited_bytes);

                Ok(None)
            }
            (
                ContinuationBuffer::Text(limited_bytes) | ContinuationBuffer::Binary(limited_bytes),
                actix_ws::Item::Continue(bytes),
            ) => {
                limited_bytes.put(bytes)?;

                Ok(None)
            }
            (ContinuationBuffer::Text(bytes_mut), actix_ws::Item::Last(bytes)) => {
                let final_bytes = bytes_mut.put_final(bytes)?;
                *self = ContinuationBuffer::Empty;
                let text = String::from_utf8(final_bytes.into())?;

                Ok(Some(ContinuationMessage::Text(text)))
            }
            (ContinuationBuffer::Binary(bytes_mut), actix_ws::Item::Last(bytes)) => {
                let final_bytes = bytes_mut.put_final(bytes)?;
                *self = ContinuationBuffer::Empty;

                Ok(Some(ContinuationMessage::Binary(final_bytes.into())))
            }
            (ContinuationBuffer::Empty, actix_ws::Item::Last(_))
            | (ContinuationBuffer::Empty, actix_ws::Item::Continue(_))
            | (ContinuationBuffer::Text(_), actix_ws::Item::FirstText(_))
            | (ContinuationBuffer::Text(_), actix_ws::Item::FirstBinary(_))
            | (ContinuationBuffer::Binary(_), actix_ws::Item::FirstText(_))
            | (ContinuationBuffer::Binary(_), actix_ws::Item::FirstBinary(_)) => {
                Err(ContinuationError::Protocol)
            }
        }
    }
}

#[derive(Debug, Error, Display, PartialEq, Eq)]
pub enum ContinuationError {
    Protocol,
    MessageTooLarge,
    ParsingFailed(FromUtf8Error),
}

impl From<FromUtf8Error> for ContinuationError {
    fn from(value: FromUtf8Error) -> Self {
        Self::ParsingFailed(value)
    }
}

/// A mutable byte buffer that enforces a [`MAX_MESSAGE_SIZE`].
#[derive(Default, Debug, PartialEq, Eq)]
struct LimitedBytesMut {
    inner: BytesMut,
}

impl LimitedBytesMut {
    pub fn new(bytes: Bytes) -> Result<Self, ContinuationError> {
        if bytes.len() > MAX_MESSAGE_SIZE {
            Err(ContinuationError::MessageTooLarge)
        } else {
            Ok(Self {
                inner: bytes.into(),
            })
        }
    }

    fn put(&mut self, bytes: Bytes) -> Result<(), ContinuationError> {
        if self.inner.len() + bytes.len() > MAX_MESSAGE_SIZE {
            return Err(ContinuationError::MessageTooLarge);
        }

        self.inner.put(bytes);
        Ok(())
    }

    fn put_final(&mut self, bytes: Bytes) -> Result<LimitedBytesMut, ContinuationError> {
        self.put(bytes)?;
        Ok(std::mem::take(self))
    }
}

impl From<LimitedBytesMut> for Vec<u8> {
    fn from(value: LimitedBytesMut) -> Self {
        value.inner.into()
    }
}

impl From<LimitedBytesMut> for Bytes {
    fn from(value: LimitedBytesMut) -> Self {
        value.inner.into()
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_matches, iter};

    use actix_ws::Item;
    use bytes::Bytes;
    use pretty_assertions::assert_eq;

    use crate::signaling::continuation_buffer::{
        ContinuationBuffer, ContinuationError, ContinuationMessage, MAX_MESSAGE_SIZE,
    };

    #[test]
    fn binary() {
        let mut buffer = ContinuationBuffer::Empty;

        let bytes = Bytes::from_iter(0..=2);
        let message = buffer.extend(Item::FirstBinary(bytes));

        assert!(message.is_none());

        let bytes = Bytes::from_iter(3..=5);
        let message = buffer.extend(Item::Continue(bytes));

        assert!(message.is_none());

        let bytes = Bytes::from_iter(6..=8);
        let message = buffer.extend(Item::Last(bytes));

        let expected_bytes = Bytes::from_iter(0..=8);
        assert_matches!(
            message,
            Some(
               Ok(ContinuationMessage::Binary(produced_bytes))
            ) if produced_bytes == expected_bytes
        );

        assert_eq!(buffer, ContinuationBuffer::Empty);
    }

    #[test]
    fn text() {
        let mut buffer = ContinuationBuffer::Empty;

        let bytes = Bytes::from("Hello, ");
        let message = buffer.extend(Item::FirstText(bytes));

        assert!(message.is_none());

        let bytes = Bytes::from("World");
        let message = buffer.extend(Item::Continue(bytes));

        assert!(message.is_none());

        let bytes = Bytes::from("!");
        let message = buffer.extend(Item::Last(bytes));

        let expected_text = "Hello, World!".to_string();
        assert_matches!(
            message,
            Some(
               Ok(ContinuationMessage::Text(produced_text))
            ) if produced_text == expected_text
        );

        assert_eq!(buffer, ContinuationBuffer::Empty);
    }

    #[test]
    fn continue_before_first() {
        let mut buffer = ContinuationBuffer::Empty;

        let bytes = Bytes::from_iter(0..=2);
        let error = buffer.extend(Item::Continue(bytes)).unwrap().unwrap_err();

        assert_eq!(error, ContinuationError::Protocol);
        assert_eq!(buffer, ContinuationBuffer::Empty);
    }

    #[test]
    fn last_before_first() {
        let mut buffer = ContinuationBuffer::Empty;

        let bytes = Bytes::from_iter(0..=2);
        let error = buffer.extend(Item::Last(bytes)).unwrap().unwrap_err();

        assert_eq!(error, ContinuationError::Protocol);
        assert_eq!(buffer, ContinuationBuffer::Empty);
    }

    #[test]
    fn message_too_large() {
        let mut buffer = ContinuationBuffer::Empty;

        let bytes = Bytes::from_iter(iter::repeat_n(0, MAX_MESSAGE_SIZE + 1));
        let error = buffer
            .extend(Item::FirstBinary(bytes))
            .unwrap()
            .unwrap_err();

        assert_eq!(error, ContinuationError::MessageTooLarge);

        let bytes = Bytes::from_iter(iter::repeat_n(0, MAX_MESSAGE_SIZE));
        let message = buffer.extend(Item::FirstBinary(bytes));

        assert!(message.is_none());

        let bytes = Bytes::from_iter([1]);
        let error = buffer.extend(Item::Continue(bytes)).unwrap().unwrap_err();

        assert_eq!(error, ContinuationError::MessageTooLarge);
        assert_eq!(buffer, ContinuationBuffer::Empty);
    }
}
