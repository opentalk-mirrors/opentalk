// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_inventory::{
    ModuleResource, ModuleResourceFilter, ModuleResourceInventory, ModuleResourceOperation,
    NewModuleResource,
};
use opentalk_types_common::{module_resources::ModuleResourceId, rooms::RoomId};
use snafu::ResultExt as _;

use crate::{
    DatabaseConnection, Result,
    error::{DatabaseSnafu, JsonOperationSnafu},
};

#[async_trait::async_trait]
impl ModuleResourceInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_module_resource(
        &mut self,
        resource: NewModuleResource,
    ) -> Result<ModuleResource> {
        Ok(
            db::queries::module_resources::create_module_resource(&mut self.inner, resource.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_module_resources(
        &mut self,
        resource_filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>> {
        Ok(db::queries::module_resources::get_module_resource(
            &mut self.inner,
            resource_filter.into(),
        )
        .await
        .context(DatabaseSnafu)?
        .into_iter()
        .map(Into::into)
        .collect())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn patch_module_resources(
        &mut self,
        resource_filter: ModuleResourceFilter,
        operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>> {
        Ok(db::queries::module_resources::patch_module_resources(
            &mut self.inner,
            resource_filter.into(),
            operations,
        )
        .await
        .context(JsonOperationSnafu)?
        .into_iter()
        .map(Into::into)
        .collect())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_module_ids_for_room(
        &mut self,
        room_id: RoomId,
    ) -> Result<Vec<ModuleResourceId>> {
        Ok(
            db::queries::module_resources::get_all_module_ids_for_room(&mut self.inner, room_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_module_resources(
        &mut self,
        resource_filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>> {
        Ok(db::queries::module_resources::delete_module_resources(
            &mut self.inner,
            resource_filter.into(),
        )
        .await
        .context(DatabaseSnafu)?
        .into_iter()
        .map(Into::into)
        .collect())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_all_module_resources_for_room(&mut self, room_id: RoomId) -> Result<()> {
        Ok(
            db::queries::module_resources::delete_all_module_resources_for_room(
                &mut self.inner,
                room_id,
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }
}
