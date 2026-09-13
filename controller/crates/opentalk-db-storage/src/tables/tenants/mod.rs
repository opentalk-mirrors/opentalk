// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains tenants table structs

mod new_tenant;
mod oidc_tenant_id;
mod tenant;
mod update_tenant;

pub use new_tenant::NewTenant;
pub use oidc_tenant_id::OidcTenantId;
pub use tenant::Tenant;
pub use update_tenant::UpdateTenant;
