// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, TimeZone as _, Utc};
use opentalk_types_common::{
    assets::{AssetId, FileSize},
    modules::ModuleId,
    utils::ExampleData,
};

/// Representation of an asset resource
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature="utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(AssetResource::example_data())),
)]
pub struct AssetResource {
    /// The ID of an asset
    pub id: AssetId,

    /// The filename of the asset
    pub filename: String,

    /// The namespace of the asset
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub namespace: Option<ModuleId>,

    /// The timestamp the asset was created
    pub created_at: DateTime<Utc>,

    /// The asset kind
    pub kind: String,

    /// The size of the asset in bytes
    pub size: FileSize,
}

impl ExampleData for AssetResource {
    fn example_data() -> Self {
        Self {
            id: AssetId::example_data(),
            filename: "recording.webm".to_string(),
            namespace: Some("recording".parse().expect("valid module id")),
            created_at: Utc.with_ymd_and_hms(2024, 6, 18, 11, 22, 33).unwrap(),
            kind: "record".to_string(),
            size: 98765432.into(),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use opentalk_types_common::{assets::AssetId, utils::ExampleData as _};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::AssetResource;

    #[test]
    fn serialize_example_data() {
        let example = AssetResource::example_data();
        assert_eq!(
            json!(example),
            json!({"id": AssetId::example_data(),
                "filename": "recording.webm",
                "namespace": "recording",
                "created_at": "2024-06-18T11:22:33Z",
                "kind":"record",
                "size":98765432
            })
        );
    }

    #[test]
    fn deserialize_example_data() {
        let example = AssetResource::example_data();
        assert_eq!(
            example,
            serde_json::from_value(json!({"id": AssetId::example_data(),
                "filename": "recording.webm",
                "namespace": "recording",
                "created_at": "2024-06-18T11:22:33Z",
                "kind":"record",
                "size":98765432
            }))
            .unwrap()
        );
    }
}
