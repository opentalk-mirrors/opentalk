// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{streaming::RoomStreamingTarget, utils::ExampleData};

/// The body of a *PATCH /rooms/{room_id}/streaming_targets/{streaming_target_id}* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PatchRoomStreamingTargetResponseBody::example_data()))
)]
pub struct PatchRoomStreamingTargetResponseBody(pub RoomStreamingTarget);

impl ExampleData for PatchRoomStreamingTargetResponseBody {
    fn example_data() -> Self {
        Self(RoomStreamingTarget::example_data())
    }
}
