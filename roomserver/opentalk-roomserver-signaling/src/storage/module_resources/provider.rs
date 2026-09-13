// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{any::Any, fmt::Debug};

use async_trait::async_trait;

use crate::storage::module_resources::{
    Error, ModuleResource, ModuleResourceFilter, ModuleResourceOperation, NewModuleResource,
};

/// The internal interface to create or retrieve [`ModuleResource`]s
///
/// Can be implemented by some backend that allows to store module resources
#[async_trait]
pub trait ModuleResourceProvider: Any + Send + Sync + Debug {
    /// Create a new [`ModuleResource`]
    ///
    /// Returns the created resource
    async fn create(&self, resource: NewModuleResource) -> Result<ModuleResource, Error>;

    /// Get all [`ModuleResource`]s where the filter applies
    async fn get(&self, filter: ModuleResourceFilter) -> Result<Vec<ModuleResource>, Error>;

    /// Apply the given json operations to all [`ModuleResource`]s where the filter applies
    ///
    /// When one patch operation fails on any resource, none of the resources will be modified
    async fn patch(
        &self,
        filter: ModuleResourceFilter,
        operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>, Error>;

    /// Delete all [`ModuleResource`]s where the filter applies
    async fn delete(&self, filter: ModuleResourceFilter) -> Result<Vec<ModuleResource>, Error>;
}
