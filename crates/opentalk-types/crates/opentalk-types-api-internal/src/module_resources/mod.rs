// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the module resource API

mod filter;
mod module_resource;
mod new_module_resource;
mod patch;

pub use filter::ModuleResourceFilter;
pub use module_resource::ModuleResource;
pub use new_module_resource::NewModuleResource;
pub use patch::{ModuleResourceOperation, PatchModuleResourceBody};
