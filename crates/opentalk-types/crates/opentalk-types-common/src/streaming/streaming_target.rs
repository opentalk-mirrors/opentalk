// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{streaming::StreamingTargetKind, utils::ExampleData};

/// A streaming target
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingTarget::example_data()))
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export, export_to = "common/")
)]
pub struct StreamingTarget {
    /// The name of the streaming target
    pub name: String,

    /// The kind of the streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub kind: StreamingTargetKind,
}

impl ExampleData for StreamingTarget {
    fn example_data() -> Self {
        Self {
            name: "Example Stream".to_string(),
            kind: StreamingTargetKind::example_data(),
        }
    }
}
