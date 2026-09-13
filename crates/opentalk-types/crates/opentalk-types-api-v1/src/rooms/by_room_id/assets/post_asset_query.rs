// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    assets::{AssetFileKind, FileExtension},
    events::EventTitle,
    modules::ModuleId,
    utils::ExampleData,
};

/// Query parameters for the `POST /rooms/{room_id}/assets` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::IntoParams),
    derive(utoipa::ToSchema),
    schema(example = json!(PostAssetQuery::example_data()))
)]
pub struct PostAssetQuery {
    /// The title of the event
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub event_title: Option<EventTitle>,

    /// The file extension
    pub file_extension: FileExtension,

    /// The kind of the asset
    pub kind: AssetFileKind,

    /// The namespace the asset is assigned to
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub namespace: Option<ModuleId>,
}

impl ExampleData for PostAssetQuery {
    fn example_data() -> Self {
        Self {
            event_title: Some(EventTitle::example_data()),
            file_extension: FileExtension::example_data(),
            kind: AssetFileKind::example_data(),
            namespace: Some(ModuleId::example_data()),
        }
    }
}
