// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Debug;

use tokio::sync::mpsc::error::SendError;

/// The RoomServer runner is not reachable anymore.
#[derive(Debug)]
pub struct RunnerGoneError;

impl<T: Debug> From<SendError<T>> for RunnerGoneError {
    fn from(SendError(command): SendError<T>) -> Self {
        log::error!("Failed to send command to runner: {command:?}");
        Self
    }
}
