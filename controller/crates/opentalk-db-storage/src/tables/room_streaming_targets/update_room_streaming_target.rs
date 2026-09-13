// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_inventory as inventory;
use opentalk_types_common::streaming::StreamingKind;

use crate::schema::room_streaming_targets;

/// Diesel streaming target struct
///
/// Represents a changeset of in invite
#[derive(Debug, AsChangeset)]
#[diesel(table_name = room_streaming_targets)]
pub struct UpdateRoomStreamingTarget {
    pub name: Option<String>,
    pub kind: Option<StreamingKind>,
    pub streaming_endpoint: Option<String>,
    pub streaming_key: Option<String>,
    pub public_url: Option<String>,
}

impl From<inventory::UpdateRoomStreamingTarget> for UpdateRoomStreamingTarget {
    fn from(
        inventory::UpdateRoomStreamingTarget {
            name,
            kind,
            streaming_endpoint,
            streaming_key,
            public_url,
        }: inventory::UpdateRoomStreamingTarget,
    ) -> Self {
        Self {
            name,
            kind,
            streaming_endpoint,
            streaming_key,
            public_url,
        }
    }
}
