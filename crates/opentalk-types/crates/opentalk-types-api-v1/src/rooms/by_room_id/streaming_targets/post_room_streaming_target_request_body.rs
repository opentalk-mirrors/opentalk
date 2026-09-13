// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{streaming::StreamingTarget, utils::ExampleData};

/// The body of a *POST /rooms/{room_id}/streaming_targets* request
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(PostRoomStreamingTargetRequestBody::example_data()))
)]
pub struct PostRoomStreamingTargetRequestBody(pub StreamingTarget);

impl ExampleData for PostRoomStreamingTargetRequestBody {
    fn example_data() -> Self {
        Self(StreamingTarget::example_data())
    }
}
