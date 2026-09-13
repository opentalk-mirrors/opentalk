// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{assets::AssetId, modules::ModuleId, tenants::TenantId};

/// Representation of an asset that should be created in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAsset {
    /// The id of the asset.
    pub id: AssetId,

    /// An optional module namespace.
    pub namespace: Option<ModuleId>,

    /// The asset kind.
    pub kind: String,

    /// The filename of the asset.
    pub filename: String,

    /// The id of the tenant to which the asset belongs.
    pub tenant_id: TenantId,

    /// The size of the asset, in bytes.
    pub size: i64,
}
