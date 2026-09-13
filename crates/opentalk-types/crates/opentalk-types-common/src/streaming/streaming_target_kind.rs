// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use url::Url;

use crate::{streaming::StreamingKey, utils::ExampleData};

/// A streaming target kind
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "kind", rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(StreamingTargetKind::example_data()))
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export, export_to = "common/")
)]
pub enum StreamingTargetKind {
    /// The "custom" kind
    Custom {
        /// The endpoint url of the streaming target
        streaming_endpoint: Url,
        /// The streaming key
        streaming_key: StreamingKey,
        /// The url from which the stream can be accessed
        public_url: Url,
    },
}

impl ExampleData for StreamingTargetKind {
    fn example_data() -> Self {
        Self::Custom {
            streaming_endpoint: "https://ingress.streaming.example.com/"
                .parse()
                .expect("url should be valid"),
            streaming_key: StreamingKey::example_data(),
            public_url: "https://streaming.example.com/livestream123"
                .parse()
                .expect("url should be valid"),
        }
    }
}

impl StreamingTargetKind {
    /// Return streaming_endpoint + streaming_key
    pub fn get_stream_target_location(&self) -> Option<Url> {
        match self {
            StreamingTargetKind::Custom {
                streaming_endpoint,
                streaming_key,
                public_url: _,
            } => {
                let mut endpoint = streaming_endpoint.clone();
                if !endpoint.as_str().ends_with('/') {
                    endpoint.set_path(&format!("{}/", endpoint.path()));
                }

                endpoint.join(streaming_key.as_str()).ok()
            }
        }
    }
}
