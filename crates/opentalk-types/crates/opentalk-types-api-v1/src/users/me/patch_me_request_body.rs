// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    users::{DisplayName, Language, Theme, UserTitle},
    utils::ExampleData,
};

/// Used to modify user settings.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PatchMeRequestBody::example_data()
    )
))]
pub struct PatchMeRequestBody {
    /// The user's title
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub title: Option<UserTitle>,

    /// The user's display name
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub display_name: Option<DisplayName>,

    /// The user's language
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub language: Option<Language>,

    /// The dashboard theme
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub dashboard_theme: Option<Theme>,

    /// The conference theme
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub conference_theme: Option<Theme>,
}

impl PatchMeRequestBody {
    /// Check if any field is empty in `PatchMeRequestBody`.
    pub fn is_empty(&self) -> bool {
        let PatchMeRequestBody {
            title,
            display_name,
            language,
            dashboard_theme,
            conference_theme,
        } = self;

        title.is_none()
            && display_name.is_none()
            && language.is_none()
            && dashboard_theme.is_none()
            && conference_theme.is_none()
    }
}

impl ExampleData for PatchMeRequestBody {
    fn example_data() -> Self {
        Self {
            display_name: Some("Alice Adams".parse().expect("valid display name")),
            language: Some("en".parse().expect("valid language")),
            ..Default::default()
        }
    }
}
