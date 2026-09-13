// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};
use url::Url;

use crate::StreamStatus;

/// The state information about a livestream
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StreamingTarget {
    /// The name of the stream
    pub name: String,
    /// Where to watch the stream
    pub public_url: Url,
    /// The state of the stream
    #[serde(flatten)]
    pub status: StreamStatus,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize() {
        let state = StreamingTarget {
            name: "Example Stream".into(),
            public_url: "http://example.org/".parse().unwrap(),
            status: StreamStatus::Active,
        };

        assert_json_snapshot!(state, @ r#"
        {
          "name": "Example Stream",
          "public_url": "http://example.org/",
          "status": "active"
        }
        "#);
    }

    #[test]
    fn deserialize() {
        let json = json!(
            {
                "name": "Example Stream",
                "public_url": "http://example.org/",
                "status": "active"
            }
        );

        let produced: StreamingTarget = serde_json::from_value(json).unwrap();
        let expected = StreamingTarget {
            name: "Example Stream".into(),
            public_url: "http://example.org/".parse().unwrap(),
            status: StreamStatus::Active,
        };

        assert_eq!(produced, expected)
    }
}
