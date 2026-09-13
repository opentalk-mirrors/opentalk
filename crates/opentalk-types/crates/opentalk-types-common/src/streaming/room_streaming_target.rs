// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains commonly used types for streaming target endpoints.

use crate::{
    streaming::{StreamingLink, StreamingTarget, StreamingTargetId, StreamingTargetKind},
    utils::ExampleData,
};

/// A streaming target which is specific for a Room
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(RoomStreamingTarget::example_data()))
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export, export_to = "common/")
)]
pub struct RoomStreamingTarget {
    /// The streaming target id
    pub id: StreamingTargetId,

    /// The streaming target
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub streaming_target: StreamingTarget,
}

impl ExampleData for RoomStreamingTarget {
    fn example_data() -> Self {
        Self {
            id: StreamingTargetId::example_data(),
            streaming_target: StreamingTarget::example_data(),
        }
    }
}

/// Extracts the public URLs from streaming targets
pub async fn get_public_urls_from_room_streaming_targets(
    streaming_targets: Vec<RoomStreamingTarget>,
) -> Vec<StreamingLink> {
    streaming_targets
        .into_iter()
        .map(|target| match target.streaming_target.kind {
            StreamingTargetKind::Custom {
                streaming_endpoint: _,
                streaming_key: _,
                public_url,
            } => StreamingLink {
                name: target.streaming_target.name,
                url: public_url,
            },
        })
        .collect()
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::streaming::StreamingKey;

    #[test]
    fn streaming_target_basic() {
        let expected = json!({
            "id": "00000000-0000-0000-0000-000000000000",
            "name": "my streaming target",
            "kind": "custom",
            "streaming_endpoint": "http://127.0.0.1/",
            "streaming_key": "1337",
            "public_url": "https://localhost/",
        });

        let produced = serde_json::to_value(RoomStreamingTarget {
            id: StreamingTargetId::nil(),
            streaming_target: StreamingTarget {
                name: "my streaming target".to_string(),
                kind: StreamingTargetKind::Custom {
                    streaming_endpoint: "http://127.0.0.1/".parse().unwrap(),
                    streaming_key: StreamingKey::from("1337".to_string()),
                    public_url: "https://localhost/".parse().unwrap(),
                },
            },
        })
        .unwrap();

        assert_eq!(expected, produced);
    }
}
