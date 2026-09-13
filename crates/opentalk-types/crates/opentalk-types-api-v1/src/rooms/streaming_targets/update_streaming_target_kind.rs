// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{streaming::StreamingKey, utils::ExampleData};
use url::Url;

/// Data to update a streaming target kind (only fields with [`Some`] are updated)
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(UpdateStreamingTargetKind::example_data()))
)]
pub enum UpdateStreamingTargetKind {
    /// The "custom" kind
    Custom {
        /// The endpoint url of the streaming target
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        streaming_endpoint: Option<Url>,

        /// The streaming key
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        streaming_key: Option<StreamingKey>,
        /// The url from which the stream can be accessed
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        public_url: Option<Url>,
    },
}

impl ExampleData for UpdateStreamingTargetKind {
    fn example_data() -> Self {
        Self::Custom {
            streaming_endpoint: Some(
                "https://ingress.example.com"
                    .parse()
                    .expect("parseable url"),
            ),
            streaming_key: Some("aabbccddeeff".parse().expect("parseable streaming key")),
            public_url: Some(
                "https://owncast.example.com"
                    .parse()
                    .expect("parseable url"),
            ),
        }
    }
}
