// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{modules::ModuleId, rooms::RoomId, users::UserId};
use serde::{Deserialize, Serialize};

/// Type to create a new module resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct NewModuleResource {
    /// The id of the user who creates the module resource.
    pub created_by: UserId,

    /// The room of the resource
    pub room_id: RoomId,

    /// The namespace of the module resource.
    pub namespace: ModuleId,

    /// An optional tag for the module resource, may be used by the corresponding module.
    pub tag: Option<String>,

    /// The module resource data.
    pub data: serde_json::Value,
}
