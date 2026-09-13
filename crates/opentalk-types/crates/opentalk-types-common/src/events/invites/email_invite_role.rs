// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{sql_enum, utils::ExampleData};

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq)]
    #[cfg_attr(
        feature="serde",
        derive(serde::Serialize, serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(
        feature = "utoipa",
        derive(utoipa::ToSchema),
        schema(example = json!(EmailInviteRole::example_data()))
    )]
    EmailInviteRole,
    "email_invite_role",
    EmailInviteRoleType,
    {
        Guest = b"guest",
        Moderator = b"moderator",
    }
);

// This is a false positive, due the the sql_enum macro expansion.
#[allow(clippy::derivable_impls)]
impl Default for EmailInviteRole {
    fn default() -> Self {
        Self::Guest
    }
}

impl ExampleData for EmailInviteRole {
    fn example_data() -> Self {
        Self::Guest
    }
}
