// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{
    Inventory, ModuleResource as DbModuleResource, ModuleResourceFilter as DbModuleResourceFilter,
    ModuleResourceOperation as DbModuleResourceOperation, NewModuleResource as DbNewModuleResource,
};
use opentalk_types_api_internal::module_resources::{
    ModuleResource, ModuleResourceFilter, ModuleResourceOperation, NewModuleResource,
};
use opentalk_types_common::modules::{ModuleId, module_id};

use crate::ControllerBackend;

impl ControllerBackend {
    pub(crate) async fn create_module_resource(
        &self,
        resource: NewModuleResource,
    ) -> Result<ModuleResource, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        Ok(create_module_resource(inventory.as_mut(), resource).await?)
    }

    pub(crate) async fn get_module_resources(
        &self,
        filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        Ok(get_module_resources(inventory.as_mut(), filter).await?)
    }

    pub(crate) async fn patch_module_resources(
        &self,
        filter: ModuleResourceFilter,
        patch_operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        Ok(patch_module_resources(inventory.as_mut(), filter, patch_operations).await?)
    }

    pub(crate) async fn delete_module_resources(
        &self,
        filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        Ok(delete_module_resources(inventory.as_mut(), filter).await?)
    }
}

pub(crate) async fn create_module_resource(
    inventory: &mut dyn Inventory,
    resource: NewModuleResource,
) -> Result<ModuleResource, opentalk_inventory::Error> {
    let user = inventory.get_user(resource.created_by).await?;

    let new_resource = DbNewModuleResource {
        tenant_id: user.tenant_id,
        room_id: resource.room_id,
        created_by: resource.created_by,
        namespace: resource.namespace.to_string(),
        tag: resource.tag,
        data: resource.data,
    };

    let db_resource = inventory.create_module_resource(new_resource).await?;

    Ok(api_resource_from_db_resource(db_resource))
}

pub(crate) async fn get_module_resources(
    inventory: &mut dyn Inventory,
    filter: ModuleResourceFilter,
) -> Result<Vec<ModuleResource>, opentalk_inventory::Error> {
    let db_filter = api_to_db_filter(filter);

    let db_resources = inventory.get_module_resources(db_filter).await?;

    Ok(db_resources
        .into_iter()
        .map(api_resource_from_db_resource)
        .collect())
}

pub(crate) async fn patch_module_resources(
    inventory: &mut dyn Inventory,
    filter: ModuleResourceFilter,
    patch_operations: Vec<ModuleResourceOperation>,
) -> Result<Vec<ModuleResource>, opentalk_inventory::Error> {
    let db_filter = api_to_db_filter(filter);
    let db_operation = patch_operations
        .into_iter()
        .map(api_to_db_operation)
        .collect();

    let db_resources = inventory
        .patch_module_resources(db_filter, db_operation)
        .await?;

    Ok(db_resources
        .into_iter()
        .map(api_resource_from_db_resource)
        .collect())
}

pub(crate) async fn delete_module_resources(
    inventory: &mut dyn Inventory,
    filter: ModuleResourceFilter,
) -> Result<Vec<ModuleResource>, opentalk_inventory::Error> {
    let db_filter = api_to_db_filter(filter);

    let db_resources = inventory.delete_module_resources(db_filter).await?;

    Ok(db_resources
        .into_iter()
        .map(api_resource_from_db_resource)
        .collect())
}

fn api_to_db_operation(operation: ModuleResourceOperation) -> DbModuleResourceOperation {
    match operation {
        ModuleResourceOperation::Add { path, value } => {
            DbModuleResourceOperation::Add { path, value }
        }
        ModuleResourceOperation::Remove { path } => DbModuleResourceOperation::Remove { path },
        ModuleResourceOperation::Replace { path, value } => {
            DbModuleResourceOperation::Replace { path, value }
        }
        ModuleResourceOperation::Move { from, path } => {
            DbModuleResourceOperation::Move { from, path }
        }
        ModuleResourceOperation::Copy { from, path } => {
            DbModuleResourceOperation::Copy { from, path }
        }
        ModuleResourceOperation::Test { path, value } => {
            DbModuleResourceOperation::Test { path, value }
        }
    }
}

fn api_to_db_filter(filter: ModuleResourceFilter) -> DbModuleResourceFilter {
    DbModuleResourceFilter {
        id: filter.id,
        room_id: filter.room_id,
        namespace: filter.namespace.map(|module_id| module_id.to_string()),
        created_by: filter.created_by,
        tag: filter.tag,
        json: None,
    }
}

fn api_resource_from_db_resource(db_resource: DbModuleResource) -> ModuleResource {
    ModuleResource {
        id: db_resource.id,
        tenant_id: db_resource.tenant_id,
        room_id: db_resource.room_id,
        created_by: db_resource.created_by,
        created_at: db_resource.created_at,
        updated_at: db_resource.updated_at,
        // This should always work, inserting new ModuleResources requires a namespace with the ModuleId type. If we
        // would return an error, the ModuleResource would be indefinitely inaccessible for an API caller
        namespace: ModuleId::try_from(db_resource.namespace)
            .unwrap_or(module_id!("invalid_namespace")),
        tag: db_resource.tag,
        data: db_resource.data,
    }
}
