// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    module_resources::ModuleResourceId, rooms::RoomId, tenants::TenantId, time::Timestamp,
    users::UserId,
};

/// The representation of a module resource in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleResource {
    /// The id of the module resource.
    pub id: ModuleResourceId,

    /// The tenant to which this module resource is associated.
    pub tenant_id: TenantId,

    /// The id of the room to which the module resource belongs..
    pub room_id: RoomId,

    /// The id of the user who created the module resource.
    pub created_by: UserId,

    /// The creation timestamp.
    pub created_at: Timestamp,

    /// The id of the user who last updated the module resource.
    pub updated_at: Timestamp,

    /// The namespace of the module resource.
    pub namespace: String,

    /// An optional tag for the module resource, may be used by the corresponding module.
    pub tag: Option<String>,

    /// The module resource data.
    pub data: serde_json::Value,
}
