// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use url::Url;

use crate::{streaming::StreamingKey, utils::ExampleData};

/// A resource for a streaming target kind
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingTargetKindResource::example_data()))
)]
pub enum StreamingTargetKindResource {
    /// The "custom" kind
    Custom {
        /// The endpoint url of the streaming target
        streaming_endpoint: Url,
        /// The streaming key
        #[cfg_attr(
            feature = "serde",
            serde(default, skip_serializing_if = "Option::is_none")
        )]
        // Field is non-required already, utoipa adds a `nullable: true` entry
        // by default which creates a false positive in the spectral linter when
        // combined with example data.
        #[cfg_attr(feature = "utoipa", schema(nullable = false))]
        streaming_key: Option<StreamingKey>,
        /// The url from which the stream can be accessed
        public_url: Url,
    },
}

impl ExampleData for StreamingTargetKindResource {
    fn example_data() -> Self {
        Self::Custom {
            streaming_endpoint: "https://ingress.streaming.example.com/"
                .parse()
                .expect("url should be valid"),
            streaming_key: Some(StreamingKey::example_data()),
            public_url: "https://streaming.example.com/livestream123"
                .parse()
                .expect("url should be valid"),
        }
    }
}
