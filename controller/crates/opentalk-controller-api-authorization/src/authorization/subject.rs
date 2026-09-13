// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::UserId;

/// A subject for which the access to resources can be checked.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Subject {
    /// An authenticated user.
    User(UserId),

    /// A user without authentication.
    Unauthenticated,
}

impl Subject {
    /// Check whether the subject is a user
    pub fn is_user(&self) -> bool {
        matches!(self, Subject::User(_))
    }

    /// Returns `true` if the subject is [`Unauthenticated`].
    ///
    /// [`Unauthenticated`]: Subject::Unauthenticated
    #[must_use]
    pub fn is_unauthenticated(&self) -> bool {
        matches!(self, Self::Unauthenticated)
    }

    pub fn as_user(&self) -> Option<&UserId> {
        if let Self::User(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl From<UserId> for Subject {
    fn from(user_id: UserId) -> Self {
        Self::User(user_id)
    }
}

impl From<&UserId> for Subject {
    fn from(user_id: &UserId) -> Self {
        Self::from(*user_id)
    }
}
