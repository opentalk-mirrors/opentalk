// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # Job execution system for OpenTalk
//!
//! OpenTalk can execute maintenance jobs, e.g. for cleaning up orphan meetings,
//! limiting retention time of data and similar tasks. This module contains both
//! the system used for executing these jobs as well as their implementations.

#![warn(
    bad_style,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]

pub mod jobs;

mod distributed;
mod error;
mod events;
mod users;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
pub use distributed::job_runner;
pub use error::Error;
use log::Log;
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_settings::Settings;
pub use opentalk_controller_utils::deletion::NoOpStopRoomBackend;
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_inventory::InventoryProvider;
use opentalk_log::{error, info};
use serde_json::json;
use snafu::Report;

/// Execute a job
#[expect(clippy::too_many_arguments)]
pub async fn execute<J: Job>(
    logger: &dyn Log,
    inventory_provider: Arc<dyn InventoryProvider>,
    authorizer: Authorizer,
    stop_room_backend: &dyn StopRoomBackend,
    settings: &Settings,
    parameters: serde_json::Value,
    timeout: Duration,
    hide_duration: bool,
) -> Result<(), Error> {
    let start = Instant::now();

    info!(log: logger, "Starting job execution");
    info!(log: logger, "Loading parameters");

    let parameters = match J::Parameters::try_from_json(parameters) {
        Ok(parameters) => parameters,
        Err(e) => {
            error!(log: logger, "{}", Report::from_error(&e));
            return Err(e);
        }
    };

    match tokio::time::timeout(
        timeout,
        J::execute(
            logger,
            inventory_provider,
            authorizer,
            stop_room_backend,
            settings,
            parameters,
        ),
    )
    .await
    {
        Ok(Ok(())) => {
            info!(log: logger, "");
            info!(log: logger, "Job finished successfully");
            if !hide_duration {
                info!(log: logger, "Duration: {:?}", start.elapsed());
            }
            Ok(())
        }
        Ok(Err(e)) => {
            info!(log: logger, "");
            error!(log: logger, "{}", e);
            info!(log: logger, "Job failed");
            if !hide_duration {
                info!(log: logger, "Duration: {:?}", start.elapsed());
            }
            Err(e)
        }
        Err(e) => {
            info!(log: logger, "");
            let e = Error::from(e);
            error!(log: logger, "{}", Report::from_error(&e));
            info!(log: logger, "Job failed");
            if !hide_duration {
                info!(log: logger, "Duration: {:?}", start.elapsed());
            }
            Err(e)
        }
    }
}

/// A trait for job parameters
pub trait JobParameters: Sized {
    /// Try to load the job parameters from JSON
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error>;

    /// Serialize the job parameters to JSON
    fn to_json(&self) -> Result<serde_json::Value, Error>;
}

impl JobParameters for () {
    fn try_from_json(_json: serde_json::Value) -> Result<Self, Error> {
        Ok(())
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        Ok(json!({}))
    }
}

/// A trait for defining jobs that can be executed by the job execution system
#[async_trait]
pub trait Job {
    /// The type of parameters required for executing the job
    type Parameters: JobParameters;

    /// Execute the job
    async fn execute(
        logger: &dyn Log,
        inventory_provider: Arc<dyn InventoryProvider>,
        authorizer: Authorizer,
        stop_room_backend: &dyn StopRoomBackend,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error>;
}
