// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{rooms::RoomId, tenants::TenantId, users::UserId};

/// Representation of an asset that should be created in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewModuleResource {
    /// The tenant to which this module resource is associated.
    pub tenant_id: TenantId,

    /// The id of the room to which the module resource belongs..
    pub room_id: RoomId,

    /// The id of the user who created the module resource.
    pub created_by: UserId,

    /// The namespace of the module resource.
    pub namespace: String,

    /// An optional tag for the module resource, may be used by the corresponding module.
    pub tag: Option<String>,

    /// The module resource data.
    pub data: serde_json::Value,
}
