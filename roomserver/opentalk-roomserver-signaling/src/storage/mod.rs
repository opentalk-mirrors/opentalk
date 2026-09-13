// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::room_parameters::EventContext;
use opentalk_types_common::{modules::ModuleId, rooms::RoomId};

pub mod assets;
pub mod module_resources;

/// Context for scoped storage access
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageContext {
    pub room_id: RoomId,
    pub namespace: ModuleId,
    pub event: Option<EventContext>,
}
