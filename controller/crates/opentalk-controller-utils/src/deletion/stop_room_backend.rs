// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt;

use opentalk_types_common::rooms::RoomId;

use crate::deletion::Error;

/// A trait for backends that can stop (possibly running) rooms.
#[async_trait::async_trait]
pub trait StopRoomBackend: Send + Sync {
    /// Close and stop a room
    ///
    /// This function does nothing and returns no error when the specified room does not exist.
    async fn stop_room(&self, room_id: RoomId) -> Result<(), StopRoomError>;
}

/// An error that can occur when stopping a room.
///
/// The concrete cause is backend-specific and only relevant for logging, so it is
/// kept as an opaque boxed error rather than an enumerated set of variants.
#[derive(Debug)]
pub struct StopRoomError(pub Box<dyn std::error::Error + Send + Sync + 'static>);

impl fmt::Display for StopRoomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for StopRoomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<StopRoomError> for Error {
    fn from(source: StopRoomError) -> Self {
        Self::RoomDelete { source }
    }
}

/// An implementation of [`StopRoomBackend`] that doesn't do anything.
#[derive(Debug)]
pub struct NoOpStopRoomBackend;

#[async_trait::async_trait]
impl StopRoomBackend for NoOpStopRoomBackend {
    async fn stop_room(&self, _room_id: RoomId) -> Result<(), StopRoomError> {
        Ok(())
    }
}
