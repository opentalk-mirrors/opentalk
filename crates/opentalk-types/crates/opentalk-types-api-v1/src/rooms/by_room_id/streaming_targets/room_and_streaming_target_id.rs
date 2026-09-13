// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{rooms::RoomIdOrAlias, streaming::StreamingTargetId};

/// The parameter set for */rooms/{room_id_or_alias}/streaming_targets/{streaming_target_id}* endpoints
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct RoomAndStreamingTargetId {
    /// The room id or alias for the invite
    pub room_id_or_alias: RoomIdOrAlias,

    /// The streaming target id
    pub streaming_target_id: StreamingTargetId,
}
