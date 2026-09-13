// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Functionality for deleting database entries alongside the resources they reference

mod deleter;
mod error;
mod event;
pub mod room;
mod shared_folders;
mod stop_room_backend;
pub mod user;

pub use deleter::Deleter;
pub use error::Error;
pub use event::EventDeleter;
pub use room::RoomDeleter;
pub use stop_room_backend::{NoOpStopRoomBackend, StopRoomBackend, StopRoomError};
