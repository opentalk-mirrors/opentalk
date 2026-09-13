// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::sync::Arc;

use opentalk_inventory::InventoryProvider;
use opentalk_roomserver_room::storage::module_resources::{Error, ModuleResourceProvider};
use opentalk_types_api_internal::module_resources::{
    ModuleResource, ModuleResourceFilter, ModuleResourceOperation, NewModuleResource,
};

use crate::controller_backend::module_resources;

#[derive(Debug)]
pub(crate) struct ModuleResources {
    inventory_provider: Arc<dyn InventoryProvider>,
}

impl ModuleResources {
    pub fn new(inventory_provider: Arc<dyn InventoryProvider>) -> Self {
        Self { inventory_provider }
    }
}

#[async_trait::async_trait]
impl ModuleResourceProvider for ModuleResources {
    async fn create(&self, resource: NewModuleResource) -> Result<ModuleResource, Error> {
        let mut inventory = self
            .inventory_provider
            .get_inventory()
            .await
            .map_err(Error::internal)?;

        module_resources::create_module_resource(inventory.as_mut(), resource)
            .await
            .map_err(Error::internal)
    }

    async fn get(&self, filter: ModuleResourceFilter) -> Result<Vec<ModuleResource>, Error> {
        let mut inventory = self
            .inventory_provider
            .get_inventory()
            .await
            .map_err(Error::internal)?;

        module_resources::get_module_resources(inventory.as_mut(), filter)
            .await
            .map_err(Error::internal)
    }

    async fn patch(
        &self,
        filter: ModuleResourceFilter,
        operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>, Error> {
        let mut inventory = self
            .inventory_provider
            .get_inventory()
            .await
            .map_err(Error::internal)?;

        module_resources::patch_module_resources(inventory.as_mut(), filter, operations)
            .await
            .map_err(Error::internal)
    }

    async fn delete(&self, filter: ModuleResourceFilter) -> Result<Vec<ModuleResource>, Error> {
        let mut inventory = self
            .inventory_provider
            .get_inventory()
            .await
            .map_err(Error::internal)?;

        module_resources::delete_module_resources(inventory.as_mut(), filter)
            .await
            .map_err(Error::internal)
    }
}
