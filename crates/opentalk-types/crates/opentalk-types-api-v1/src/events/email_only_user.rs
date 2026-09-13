// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

/// User profile with only email and avatar url
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EmailOnlyUser::example_data()))
)]
pub struct EmailOnlyUser {
    /// Email of the user
    pub email: String,
    /// Avatar url of the user
    pub avatar_url: String,
}

impl ExampleData for EmailOnlyUser {
    fn example_data() -> Self {
        Self {
            email: "alice@example.com".to_string(),
            avatar_url: "https://secure.gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060"
                .to_string(),
        }
    }
}
