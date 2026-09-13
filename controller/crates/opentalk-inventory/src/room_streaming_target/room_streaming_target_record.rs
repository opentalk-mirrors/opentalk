// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    rooms::RoomId,
    streaming::{StreamingKey, StreamingKind, StreamingTargetId},
};

/// The representation of a room streaming target record in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomStreamingTargetRecord {
    /// The id of the streaming target.
    pub id: StreamingTargetId,

    /// The id of the room to which the streaming target belongs.
    pub room_id: RoomId,

    /// The name of the streaming target.
    pub name: String,

    /// The kind of the streaming target.
    pub kind: StreamingKind,

    /// The endpoint for the streaming target.
    pub streaming_endpoint: String,

    /// The key for the streaming target.
    pub streaming_key: StreamingKey,

    /// The public URL where the stream can be watched.
    pub public_url: String,
}
