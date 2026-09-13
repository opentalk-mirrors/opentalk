// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use async_trait::async_trait;
use log::Log;
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_inventory::InventoryProvider;
use opentalk_log::{debug, info};
use opentalk_types_common::time::Timestamp;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    Error, Job, JobParameters,
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InviteCleanupParameters {
    expired_before: Option<Timestamp>,
}

impl JobParameters for InviteCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

/// A job for cleaning up expired invites
#[derive(Debug)]
pub struct InviteCleanup;

#[async_trait]
impl Job for InviteCleanup {
    type Parameters = InviteCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        inventory_provider: Arc<dyn InventoryProvider>,
        _authorizer: Authorizer,
        _stop_room_backend: &dyn StopRoomBackend,
        _settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Executing invite cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");
        info!(log: logger, "");

        let mut inventory = inventory_provider.get_inventory().await?;

        let expired_before = parameters.expired_before.unwrap_or_else(|| {
            info!(log: logger, "Parameter field expired_before not set. Using current timestamp.");
            Timestamp::now()
        });

        info!(log: logger, "Clearing permissions for invites that are inactive or expired before {expired_before:?}.");

        // TODO: romve inactive or expired invites from authorizer
        let _inactive_invites = inventory
            .get_room_invites_with_room_inactive_or_expired_before(expired_before)
            .await?;
        // these must be removed…

        Ok(())
    }
}
