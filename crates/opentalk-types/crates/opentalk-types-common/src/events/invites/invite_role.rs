// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{events::invites::EmailInviteRole, sql_enum, utils::ExampleData};

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    #[cfg_attr(
        feature="serde",
        derive(serde::Serialize, serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(
        feature = "utoipa",
        derive(utoipa::ToSchema),
        schema(example = json!(InviteRole::example_data()))
    )]
    InviteRole,
    "invite_role",
    InviteRoleType,
    {
        User = b"user",
        Moderator = b"moderator",
    }
);

// This is a false positive, due the the sql_enum macro expansion.
#[allow(clippy::derivable_impls)]
impl Default for InviteRole {
    fn default() -> Self {
        Self::User
    }
}

impl From<EmailInviteRole> for InviteRole {
    fn from(value: EmailInviteRole) -> Self {
        match value {
            EmailInviteRole::Guest => Self::User,
            EmailInviteRole::Moderator => Self::Moderator,
        }
    }
}

impl ExampleData for InviteRole {
    fn example_data() -> Self {
        Self::User
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::InviteRole;

    #[test]
    fn ordering() {
        assert!(InviteRole::User < InviteRole::Moderator);
        assert!(InviteRole::User <= InviteRole::Moderator);
        assert!(InviteRole::Moderator > InviteRole::User);
        assert!(InviteRole::Moderator >= InviteRole::User);

        // Simply using `assert!(!(…comparison…))` triggers clippy warnings
        // because the comparison could be inverted easily
        assert_eq!(InviteRole::User > InviteRole::Moderator, false);
        assert_eq!(InviteRole::User >= InviteRole::Moderator, false);
        assert_eq!(InviteRole::Moderator < InviteRole::User, false);
        assert_eq!(InviteRole::Moderator <= InviteRole::User, false);
    }
}
