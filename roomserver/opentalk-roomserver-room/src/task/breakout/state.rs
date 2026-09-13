// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    pin::Pin,
    time::{Duration, SystemTime},
};

use opentalk_roomserver_types::breakout::breakout_config::BreakoutConfig;
use opentalk_types_common::time::Timestamp;
use tokio::time::Sleep;

#[derive(Debug)]
pub struct BreakoutState {
    pub config: BreakoutConfig,

    pub expires_at: Option<Timestamp>,

    timeout: Option<Pin<Box<Sleep>>>,
}

impl BreakoutState {
    /// Initialize the breakout state
    ///
    /// Starts the breakout expiry if configured
    pub(crate) fn init(config: BreakoutConfig) -> Self {
        let mut this = Self {
            config,
            expires_at: None,
            timeout: None,
        };

        if let Some(duration) = this.config.duration {
            this.set_expiry(duration);
        }

        this
    }

    /// Set the expiry for the breakout rooms
    pub(crate) fn set_expiry(&mut self, duration: Duration) {
        self.timeout = Some(Box::pin(tokio::time::sleep(duration)));

        self.expires_at = Some((SystemTime::now() + duration).into());
    }

    /// Returns when the breakout rooms have expired
    pub(crate) async fn wait_for_expiry(&mut self) {
        if let Some(timeout) = &mut self.timeout {
            timeout.await;
        } else {
            std::future::pending::<()>().await;
        }
    }
}
