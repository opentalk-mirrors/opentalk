// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use opentalk_types_common::{
    rooms::RoomId,
    streaming::{StreamingKind, StreamingTargetKind},
};

use crate::{schema::room_streaming_targets, tables::rooms::Room};

#[derive(Debug, Associations, Insertable)]
#[diesel(belongs_to(Room, foreign_key = room_id))]
#[diesel(table_name = room_streaming_targets)]
pub struct NewRoomStreamingTarget {
    pub room_id: RoomId,
    pub name: String,
    pub kind: StreamingKind,
    pub streaming_endpoint: String,
    pub streaming_key: String,
    pub public_url: String,
}

impl NewRoomStreamingTarget {
    pub fn from_streaming_target_kind(
        streaming_target_kind: StreamingTargetKind,
        room_id: RoomId,
        name: String,
    ) -> Self {
        match streaming_target_kind {
            StreamingTargetKind::Custom {
                streaming_endpoint,
                streaming_key,
                public_url,
            } => Self {
                room_id,
                name,
                kind: StreamingKind::Custom,
                streaming_endpoint: streaming_endpoint.into(),
                streaming_key: streaming_key.into(),
                public_url: public_url.into(),
            },
        }
    }
}
