// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{fmt::Debug, sync::Arc};

use opentalk_types_common::users::UserId;
use thiserror::Error;

pub mod provider;

use opentalk_types_api_internal::module_resources::{
    ModuleResource, ModuleResourceFilter, ModuleResourceOperation, NewModuleResource,
};
pub use opentalk_types_common::module_resources::ModuleResourceId;

use crate::storage::{StorageContext, module_resources::provider::ModuleResourceProvider};

#[derive(Debug, Error)]
pub enum Error {
    #[error("The requested resource could not be found")]
    NotFound,
    #[error("Failed to apply the patch to the selected resources: {msg}")]
    PatchFailed { msg: String },
    #[error("Internal error: {0:?}")]
    Internal(#[from] anyhow::Error),
}

impl Error {
    pub fn internal<E: Debug>(err: E) -> Self {
        Self::Internal(anyhow::anyhow!("{err:?}"))
    }
}

/// The interface to access [`ModuleResource`]s
///
/// Allows signaling modules to create, update and remove module resources. Every operation is
/// scoped to the modules namespace and current room.
pub struct ModuleResourceStorage {
    /// The internal storage api
    storage_api: Arc<dyn ModuleResourceProvider>,
    /// Context for the storage
    ctx: StorageContext,
}

impl ModuleResourceStorage {
    pub(crate) fn new(api: Arc<dyn ModuleResourceProvider>, ctx: StorageContext) -> Self {
        Self {
            storage_api: api,
            ctx,
        }
    }

    /// Create a new module resource
    pub async fn create(
        &self,
        created_by: UserId,
        tag: Option<String>,
        data: serde_json::Value,
    ) -> anyhow::Result<ModuleResource> {
        let resource = NewModuleResource {
            room_id: self.ctx.room_id,
            created_by,
            namespace: self.ctx.namespace.clone(),
            tag,
            data,
        };

        let resource = self.storage_api.create(resource).await?;

        Ok(resource)
    }

    /// Get a module resource by its id
    pub async fn get(&self, id: ModuleResourceId) -> Result<Option<ModuleResource>, Error> {
        let filter = ModuleResourceFilter::new(self.ctx.room_id, self.ctx.namespace.clone()).id(id);

        let mut resources = self.storage_api.get(filter).await?;

        Ok(resources.drain(0..).next())
    }

    /// Get all resources for the current room and namespace
    pub async fn get_all(
        &self,
        filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>, Error> {
        let filter = filter
            .room_id(self.ctx.room_id)
            .namespace(self.ctx.namespace.clone());

        let resources = self.storage_api.get(filter).await?;

        Ok(resources)
    }

    /// Apply a set of json operation to a specific module resource
    pub async fn patch(
        &self,
        id: ModuleResourceId,
        operations: Vec<ModuleResourceOperation>,
    ) -> Result<ModuleResource, Error> {
        let filter = ModuleResourceFilter::new(self.ctx.room_id, self.ctx.namespace.clone()).id(id);

        let mut resources = self.storage_api.patch(filter, operations).await?;

        let Some(affected_resource) = resources.drain(0..).next() else {
            return Err(Error::NotFound);
        };

        Ok(affected_resource)
    }

    /// Delete a module resource
    pub async fn delete(&self, id: ModuleResourceId) -> Result<ModuleResource, Error> {
        let filter = ModuleResourceFilter::new(self.ctx.room_id, self.ctx.namespace.clone()).id(id);

        let mut resources = self.storage_api.delete(filter).await?;

        let Some(deleted_resource) = resources.drain(0..).next() else {
            return Err(Error::NotFound);
        };

        Ok(deleted_resource)
    }
}
