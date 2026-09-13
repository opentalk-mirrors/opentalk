// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{streaming::StreamingTargetKindResource, utils::ExampleData};

/// A resource for a streaming target
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingTargetResource::example_data()))
)]
pub struct StreamingTargetResource {
    /// The name of the streaming target
    pub name: String,

    /// The kind of the streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StreamingTargetKindResource,
}

impl ExampleData for StreamingTargetResource {
    fn example_data() -> Self {
        Self {
            name: "Example Stream".to_string(),
            kind: StreamingTargetKindResource::example_data(),
        }
    }
}
