// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use log::Log;
use opentalk_asset_storage::ObjectStorage;
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::deletion::StopRoomBackend;
use opentalk_inventory::{Inventory, InventoryProvider};
use opentalk_log::{debug, info};
use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{
    Error, Job, JobParameters,
    error::{ParameterLoadingSnafu, ParameterSerializingSnafu},
    events::delete_orphaned_rooms,
};

#[derive(Debug)]
pub struct RoomCleanup;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoomCleanupParameters {
    #[serde(default)]
    fail_on_shared_folder_deletion_error: bool,
}

impl JobParameters for RoomCleanupParameters {
    fn try_from_json(json: serde_json::Value) -> Result<Self, Error> {
        serde_json::from_value(json).context(ParameterLoadingSnafu)
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        serde_json::to_value(self).context(ParameterSerializingSnafu)
    }
}

#[async_trait]
impl Job for RoomCleanup {
    type Parameters = RoomCleanupParameters;

    async fn execute(
        logger: &dyn Log,
        inventory_provider: Arc<dyn InventoryProvider>,
        authorizer: Authorizer,
        stop_room_backend: &dyn StopRoomBackend,
        settings: &Settings,
        parameters: Self::Parameters,
    ) -> Result<(), Error> {
        info!(log: logger, "Starting orphaned rooms cleanup job");
        debug!(log: logger, "Job parameters: {parameters:?}");

        let mut inventory = inventory_provider.get_inventory().await?;

        let object_storage = ObjectStorage::new(&settings.minio).await?;

        let orphaned_rooms = find_orphaned_rooms(inventory.as_mut()).await?;

        if orphaned_rooms.is_empty() {
            info!(log: logger, "No orphaned rooms found. Job finished!");
            return Ok(());
        }

        delete_orphaned_rooms(
            logger,
            inventory.as_mut(),
            authorizer,
            stop_room_backend,
            settings,
            &object_storage,
            orphaned_rooms,
            parameters.fail_on_shared_folder_deletion_error,
        )
        .await?;

        Ok(())
    }
}

async fn find_orphaned_rooms(inventory: &mut dyn Inventory) -> Result<HashSet<RoomId>, Error> {
    let rooms = inventory.get_all_orphaned_room_ids().await?;

    Ok(HashSet::from_iter(rooms))
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::Path};

    use log::logger;
    use opentalk_controller_api_authorization::authorization::Authorizer;
    use opentalk_controller_api_authorization_database::OpenTalkAuthorizerBackend;
    use opentalk_controller_settings::SettingsProvider;
    use opentalk_inventory::InventoryProvider as _;
    use opentalk_test_util::{database::DatabaseContext, object_storage::ObjectStorageContext};

    use super::RoomCleanup;
    use crate::{
        Job as _,
        jobs::test_utils::{
            RecordingStopRoomBackend, create_generic_test_event, create_generic_test_room,
        },
    };

    /// The room cleanup job must notify the room delete backend for orphaned rooms
    /// only, leaving rooms that still belong to an event untouched.
    #[actix_rt::test]
    async fn room_cleanup_only_deletes_orphaned_rooms() {
        let settings_provider = SettingsProvider::load_from_path_or_standard_paths(Some(
            Path::new("../../example/controller.toml"),
        ))
        .unwrap();

        let object_storage_ctx = ObjectStorageContext::new().await;
        let mut settings = settings_provider.get().as_ref().clone();
        settings.minio = object_storage_ctx.minio.clone();

        let db_ctx = DatabaseContext::new().await;
        let mut inventory = db_ctx.inventory_provider.get_inventory().await.unwrap();

        let user = db_ctx.create_test_user(0, vec![]).await.unwrap();

        // A room without an associated event is orphaned and should be cleaned up.
        let orphaned_room = create_generic_test_room(inventory.as_mut(), &user).await;
        // A room that still belongs to an event is not orphaned and must be left alone.
        create_generic_test_event(inventory.as_mut(), &user, true).await;

        let authorizer = Authorizer::new(OpenTalkAuthorizerBackend::new(
            db_ctx.inventory_provider.clone(),
            settings_provider.clone(),
            BTreeMap::new(),
        ));
        let room_delete_backend = RecordingStopRoomBackend::default();

        RoomCleanup::execute(
            logger(),
            db_ctx.inventory_provider.clone(),
            authorizer,
            &room_delete_backend,
            &settings,
            serde_json::from_str("{}").unwrap(),
        )
        .await
        .unwrap();

        let deleted_rooms = room_delete_backend.deleted_rooms.lock().unwrap();

        assert_eq!(
            *deleted_rooms,
            vec![orphaned_room.id],
            "RoomCleanup should delete the orphaned room and leave the room with an event untouched"
        );
    }
}
