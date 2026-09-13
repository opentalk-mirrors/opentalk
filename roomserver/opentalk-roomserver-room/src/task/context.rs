// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use opentalk_roomserver_common::{application_state::ApplicationState, settings};
use opentalk_roomserver_signaling::storage::{
    assets::provider::AssetStorageProvider, module_resources::provider::ModuleResourceProvider,
};
use tokio::sync::watch;

use crate::ModuleRegistry;

/// Context for the room task, containing shared resources.
pub struct RoomTaskContext {
    // The module registry, which contains the registered modules and their initializers.
    pub module_registry: Arc<ModuleRegistry>,

    pub asset_storage: Arc<dyn AssetStorageProvider>,

    pub module_resources: Arc<dyn ModuleResourceProvider>,

    // The task-specific settings.
    pub settings: Arc<settings::Task>,

    // The global application state, which is watched for a shutdown signal.
    pub app_state: watch::Receiver<ApplicationState>,
}
