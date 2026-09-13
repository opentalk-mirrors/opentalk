// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{pin::Pin, time::Duration};

use tokio::time::Sleep;

/// Helper type to start and stop a timeout in the [`super::RoomTask`]
pub struct Timeout {
    timeout: Option<Pin<Box<Sleep>>>,
    duration: Duration,
}

impl Timeout {
    /// Creates a new timeout without starting it
    pub(super) fn new(duration: Duration) -> Self {
        Self {
            timeout: None,
            duration,
        }
    }

    /// Creates a new timeout and starts it
    pub(super) fn start_new(duration: Duration) -> Self {
        let mut this = Self::new(duration);
        this.restart();
        this
    }

    /// Starts the timeout
    ///
    /// Does nothing if the timeout is already running
    pub(super) fn start(&mut self) {
        if self.timeout.is_none() {
            self.restart();
        }
    }

    /// Starts the timeout
    ///
    /// Discards the current timeout if one was running
    pub(super) fn restart(&mut self) {
        tracing::debug!("Idle timer restarted with duration: {:?}", self.duration);
        self.timeout = Some(Box::pin(tokio::time::sleep(self.duration)));
    }

    /// Resets the timeout
    ///
    /// Does nothing when no timeout is currently running
    pub(super) fn reset(&mut self) {
        if self.timeout.is_some() {
            self.restart();
        }
    }

    /// Stops the timeout
    pub(super) fn stop(&mut self) {
        tracing::debug!("Idle timer stopped");
        self.timeout = None;
    }

    /// Returns only when the timeout is reached
    pub(super) async fn wait_for_completion(&mut self) {
        if let Some(timeout) = &mut self.timeout {
            timeout.await;
        } else {
            std::future::pending::<()>().await;
        }
    }
}
