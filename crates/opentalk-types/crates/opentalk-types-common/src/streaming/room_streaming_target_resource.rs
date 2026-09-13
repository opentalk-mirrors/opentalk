// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains commonly used types for streaming target endpoints.

use crate::{
    streaming::{StreamingTargetId, StreamingTargetResource},
    utils::ExampleData,
};

/// A resource for a streaming target which is specific for a Room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(RoomStreamingTargetResource::example_data()))
)]
pub struct RoomStreamingTargetResource {
    /// The streaming target id
    pub id: StreamingTargetId,

    /// The streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub streaming_target: StreamingTargetResource,
}

impl ExampleData for RoomStreamingTargetResource {
    fn example_data() -> Self {
        Self {
            id: StreamingTargetId::example_data(),
            streaming_target: StreamingTargetResource::example_data(),
        }
    }
}
