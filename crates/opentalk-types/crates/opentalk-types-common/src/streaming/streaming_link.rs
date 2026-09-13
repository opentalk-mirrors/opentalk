// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use url::Url;

use crate::utils::ExampleData;

/// Streaming link for an event
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingLink::example_data()))
)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export_to = "common/"))]
pub struct StreamingLink {
    /// The name of the streaming link
    pub name: String,

    /// The url of the streaming link
    pub url: Url,
}

impl ExampleData for StreamingLink {
    fn example_data() -> Self {
        Self {
            name: "My OwnCast Stream".to_string(),
            url: "https://owncast.example.com/mystream".parse().unwrap(),
        }
    }
}
