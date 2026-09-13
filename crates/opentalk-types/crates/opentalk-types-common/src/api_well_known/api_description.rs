// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The description of an API version.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub struct ApiDescription {
    /// The relative or absolute base url of the OpenTalk API version.
    ///
    /// If this is a relative url (e.g. `v1`), then it is relative to the same
    /// location from where the `.well-known` path is served.
    ///
    /// For example, if the well-known endpoint is
    /// `https://example.com/controller/.well-known/opentalk/api`
    /// and the contained `base_url` is `api/v1`, that would resolve to
    /// `https://example.com/controller/api/v1`.
    pub base_url: String,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;

    use crate::api_well_known::ApiDescription;

    #[test]
    fn api_description_relative_de_serialize() {
        let json = serde_json::json!({
            "base_url": "v1",
        });

        let value = ApiDescription {
            base_url: "v1".to_string(),
        };

        let serialized = serde_json::to_value(&value);
        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "Serialized JSON matches");

        let deserialized = serde_json::from_value::<ApiDescription>(json);
        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "Deserialized JSON matches");
    }

    #[test]
    fn api_description_absolute_de_serialize() {
        let json = serde_json::json!({
            "base_url": "https://example.com/v1",
        });

        let value = ApiDescription {
            base_url: "https://example.com/v1".to_string(),
        };

        let serialized = serde_json::to_value(&value);
        assert!(serialized.is_ok());
        assert_eq!(json, serialized.unwrap(), "Serialized JSON matches");

        let deserialized = serde_json::from_value::<ApiDescription>(json);
        assert!(deserialized.is_ok());
        assert_eq!(value, deserialized.unwrap(), "Deserialized JSON matches");
    }
}
