// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_inception)]

mod oidc_tenant_id;
mod tenant;
mod tenant_inventory;

pub use oidc_tenant_id::OidcTenantId;
pub use tenant::Tenant;
pub use tenant_inventory::TenantInventory;
