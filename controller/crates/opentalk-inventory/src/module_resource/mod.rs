// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_inception)]

mod module_resource;
mod module_resource_filter;
mod module_resource_inventory;
mod module_resource_operation;
mod new_module_resource;

pub use module_resource::ModuleResource;
pub use module_resource_filter::ModuleResourceFilter;
pub use module_resource_inventory::ModuleResourceInventory;
pub use module_resource_operation::ModuleResourceOperation;
pub use new_module_resource::NewModuleResource;
