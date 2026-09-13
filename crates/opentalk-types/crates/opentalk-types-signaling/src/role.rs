// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::events::invites::{EmailInviteRole, InviteRole};
use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, IntoStaticStr, VariantNames};

/// Role of the participant inside a room
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumCount,
    EnumIter,
    EnumString,
    VariantNames,
    IntoStaticStr,
)]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(Display), from_redis_value(FromStr))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
#[strum(serialize_all = "snake_case")]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export_to = "signaling/", repr(enum = name))
)]
pub enum Role {
    /// Guest participant without a registered user account
    Guest,

    /// Regular participant with a registered user account
    User,

    /// Participant with a registered user account and moderation permissions
    Moderator,
}

impl Role {
    /// Returns `true` if the role is a [`Role::Moderator`] value.
    pub const fn is_moderator(&self) -> bool {
        matches!(self, Role::Moderator)
    }

    /// Returns `true` if the role is a [`Role::User`] value.
    pub const fn is_user(&self) -> bool {
        matches!(self, Role::User)
    }

    /// Returns `true` if the role is a [`Role::Guest`] value.
    pub const fn is_guest(&self) -> bool {
        matches!(self, Role::Guest)
    }
}

impl From<EmailInviteRole> for Role {
    fn from(value: EmailInviteRole) -> Self {
        match value {
            EmailInviteRole::Guest => Self::Guest,
            EmailInviteRole::Moderator => Self::Moderator,
        }
    }
}

impl From<InviteRole> for Role {
    fn from(value: InviteRole) -> Self {
        match value {
            InviteRole::User => Self::User,
            InviteRole::Moderator => Self::Moderator,
        }
    }
}

/// Trait for modifying data types for a specific [`Role`].
///
/// This can be used to e.g. remove certain contents that not should be
/// available for certain roles.
pub trait ForRole {
    /// Modify the data type for the specified role.
    fn for_role(self, role: Role) -> Self;
}

impl ForRole for opentalk_types_common::shared_folders::SharedFolder {
    /// Get an equivalent shared folder, cut down to match the signaling role
    fn for_role(self, role: Role) -> Self {
        if role.is_moderator() {
            self
        } else {
            self.without_write_access()
        }
    }
}
