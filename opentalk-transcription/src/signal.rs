// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use tokio::{
    select,
    signal::{
        self,
        unix::{SignalKind, signal},
    },
};

use crate::tasks::ShutdownReceiver;

/// Waits for an external shutdown signal (Ctrl-C or `SIGTERM`) or an internal shutdown request.
pub(crate) async fn run_shutdown_signal_handler(
    mut shutdown_signal: ShutdownReceiver,
) -> Result<()> {
    let mut sig_term =
        signal(SignalKind::terminate()).context("Failed to set up SIGTERM handler")?;
    select! {
        _ = signal::ctrl_c() => { log::debug!("received Ctrl-C"); }
        _ = sig_term.recv() => { log::debug!("received SIGTERM"); }
        _ = shutdown_signal.wait_for_shutdown() => {
            log::trace!("Shutdown handler received shutdown signal from application state");
            return Ok(());
        }
    }

    log::info!("Received shutdown signal...");

    Ok(())
}
