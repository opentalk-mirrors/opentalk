// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{streaming::RoomStreamingTargetResource, utils::ExampleData};

/// The body of a *GET /rooms/{room_id}/streaming_targets* response
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(GetRoomStreamingTargetsResponseBody::example_data()))
)]
pub struct GetRoomStreamingTargetsResponseBody(pub Vec<RoomStreamingTargetResource>);

impl ExampleData for GetRoomStreamingTargetsResponseBody {
    fn example_data() -> Self {
        Self(vec![RoomStreamingTargetResource::example_data()])
    }
}
