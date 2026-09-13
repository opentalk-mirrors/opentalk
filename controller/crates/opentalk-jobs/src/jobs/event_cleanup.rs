// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Days, Utc};
use log::Log;
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_inventory::InventoryProvider;
use opentalk_log::{debug, error, info};
use serde::{Deserialize, Serialize};
use snafu::{Report, ResultExt};

use crate::{
    Error, Job, JobParameters,
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
    events::{DeleteSelector, perform_deletion},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCleanupParameters {
    #[serde(default = "default_days_since_last_occurrence")]
    days_since_last_occurrence: u64,

    #[serde(default)]
    fail_on_shared_folder_deletion_error: bool,
}

impl JobParameters for EventCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

/// A job for cleaning up events that ended at minimum a defined duration ago
#[derive(Debug)]
pub struct EventCleanup;

#[async_trait]
impl Job for EventCleanup {
    type Parameters = EventCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        inventory_provider: Arc<dyn InventoryProvider>,
        authorizer: Authorizer,
        stop_room_backend: &dyn StopRoomBackend,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting data protection cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        info!(log: logger, "");

        if parameters.days_since_last_occurrence < 1 {
            error!(log: logger, "Number of retention days must be 1 or greater");
            return Err(Error::JobExecutionFailed);
        }

        let now = Utc::now();
        let delete_before = now
            .checked_sub_days(Days::new(parameters.days_since_last_occurrence))
            .ok_or_else(|| {
                error!(log: logger, "Couldn't subtract number of retention days");
                Error::JobExecutionFailed
            })?;

        perform_deletion(
            logger,
            inventory_provider.clone(),
            authorizer,
            stop_room_backend,
            settings,
            parameters.fail_on_shared_folder_deletion_error,
            DeleteSelector::ScheduledThatEndedBefore(delete_before.into()),
        )
        .await
        .map_err(|err| {
            error!(log: logger, "{}", Report::from_error(err));
            Error::JobExecutionFailed
        })?;

        Ok(())
    }
}

fn default_days_since_last_occurrence() -> u64 {
    30
}
