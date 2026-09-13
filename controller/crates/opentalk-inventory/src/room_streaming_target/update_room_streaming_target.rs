// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::streaming::StreamingKind;

/// Representation of an update to a room in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRoomStreamingTarget {
    /// The name of the streaming target.
    pub name: Option<String>,

    /// The kind of the streaming target.
    pub kind: Option<StreamingKind>,

    /// The endpoint for the streaming target.
    pub streaming_endpoint: Option<String>,

    /// The key for the streaming target.
    pub streaming_key: Option<String>,

    /// The public URL where the stream can be watched.
    pub public_url: Option<String>,
}
