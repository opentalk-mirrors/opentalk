// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::ApiDescription;

/// The description of set of OpenTalk API versions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct OpenTalkApi {
    /// The description of the API v1, if provided by the service.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub v1: Option<ApiDescription>,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use crate::api_well_known::{ApiDescription, OpenTalkApi};

    #[test]
    fn open_talk_api_v1_only_de_serialize() {
        let json = serde_json::json!({
            "v1": {
                "base_url": "https://example.com/v1",
            }
        });

        let value = OpenTalkApi {
            v1: Some(ApiDescription {
                base_url: "https://example.com/v1"
                    .parse()
                    .expect("valid url required"),
            }),
        };

        let serialized = serde_json::to_value(&value);
        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "Serialized JSON matches");

        let deserialized = serde_json::from_value::<OpenTalkApi>(json);
        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "Deserialized JSON matches");
    }

    #[test]
    fn open_talk_api_empty_de_serialize() {
        let json = serde_json::json!({});

        let value = OpenTalkApi { v1: None };

        let serialized = serde_json::to_value(&value);
        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "Serialized JSON matches");

        let deserialized = serde_json::from_value::<OpenTalkApi>(json);
        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "Deserialized JSON matches");
    }
}
