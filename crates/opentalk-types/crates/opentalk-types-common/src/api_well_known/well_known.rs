// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::OpenTalkApi;

/// The resource served at `.well-known/opentalk/api` by OpenTalk API services.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct WellKnown {
    /// The description of OpenTalk API versions provided by the service.
    pub opentalk_api: OpenTalkApi,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use crate::api_well_known::{ApiDescription, OpenTalkApi, WellKnown};

    #[test]
    fn well_known_de_serialize() {
        let json = serde_json::json!({
            "opentalk_api": {
                "v1": {
                    "base_url": "https://example.com/v1",
                }
            }
        });

        let value = WellKnown {
            opentalk_api: OpenTalkApi {
                v1: Some(ApiDescription {
                    base_url: "https://example.com/v1"
                        .parse()
                        .expect("valid url required"),
                }),
            },
        };

        let serialized = serde_json::to_value(&value);
        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "Serialized JSON matches");

        let deserialized = serde_json::from_value::<WellKnown>(json);
        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "Deserialized JSON matches");
    }
}
