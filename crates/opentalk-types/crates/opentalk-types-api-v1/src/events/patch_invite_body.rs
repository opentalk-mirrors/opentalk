// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::invites::InviteRole, utils::ExampleData};

/// Request body for the `PATCH /events/{event_id}/invites/{user_id}` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(
        PatchInviteBody::example_data()
    ))
)]
pub struct PatchInviteBody {
    /// Invite role of the user
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub role: Option<InviteRole>,
}

impl ExampleData for PatchInviteBody {
    fn example_data() -> Self {
        Self {
            role: Some(InviteRole::Moderator),
        }
    }
}
