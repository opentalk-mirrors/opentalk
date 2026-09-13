// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{streaming::RoomStreamingTarget, utils::ExampleData};

/// The body of a *POST /rooms/{room_id}/streaming_targets* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PostRoomStreamingTargetResponseBody::example_data()))
)]
pub struct PostRoomStreamingTargetResponseBody(pub RoomStreamingTarget);

impl ExampleData for PostRoomStreamingTargetResponseBody {
    fn example_data() -> Self {
        Self(RoomStreamingTarget::example_data())
    }
}
