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
use opentalk_log::{debug, error, info, trace, warn};

use crate::{Error, Job};

/// A simple more or less empty job that checks whether job execution works
#[derive(Debug)]
pub struct SelfCheck;

#[async_trait]
impl Job for SelfCheck {
    type Parameters = ();

    async fn execute(
        logger: &dyn Log,
        _inventory_provider: Arc<dyn InventoryProvider>,
        _authorizer: Authorizer,
        _stop_room_backend: &dyn StopRoomBackend,
        _settings: &Settings,
        _parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Executing self-check job");
        info!(log: logger, "");

        trace!(log: logger, "Test output in TRACE level");
        debug!(log: logger, "Test output in DEBUG level");
        info!(log: logger, "Test output in INFO level");
        warn!(log: logger, "Test output in WARN level");
        error!(log: logger, "Test output in ERROR level");

        Ok(())
    }
}
