// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use opentalk_types_common::{events::invites::InviteRole, users::UserId};

use crate::authorization::Subject;

/// A collection of subjects.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubjectCollection(pub BTreeSet<Subject>);

impl SubjectCollection {
    /// Query whether a specific subject is contained in the collection.
    pub fn contains(&self, subject: &Subject) -> bool {
        self.0.contains(subject)
    }

    /// Query whether the subject collection contains an entry for a user.
    pub fn contains_any_user(&self) -> bool {
        self.0.iter().any(Subject::is_user)
    }

    /// Returns any user out of the subject collection.
    pub fn all_user_ids(&self) -> impl Iterator<Item = UserId> {
        self.0.iter().filter_map(Subject::as_user).copied()
    }

    /// Query whether the subject collection contains any of the users present in the keys of a `BTreeMap`.
    pub fn contains_any_user_by_key<V>(&self, users: &BTreeMap<UserId, V>) -> bool {
        self.0
            .iter()
            .filter_map(|s| match s {
                Subject::User(id) => Some(id),
                Subject::Unauthenticated => None,
            })
            .any(|id| users.contains_key(id))
    }

    /// Query whether any of the users in the subject collection has a role equal or higher in a
    /// given `BTreeMap` of users with their roles.
    pub fn any_user_has_role_or_higher(
        &self,
        users: &BTreeMap<UserId, InviteRole>,
        minimum_role: InviteRole,
    ) -> bool {
        users.iter().any(|(&user_id, &role)| {
            role >= minimum_role && self.0.contains(&Subject::User(user_id))
        })
    }
}

impl FromIterator<Subject> for SubjectCollection {
    fn from_iter<I: IntoIterator<Item = Subject>>(iter: I) -> Self {
        Self(BTreeSet::from_iter(iter))
    }
}

#[cfg(feature = "actix-web")]
pub(super) mod actix_web_impls {

    use actix_web::{HttpMessage, dev::ServiceRequest};
    use opentalk_types_common::users::UserId;

    use super::*;

    impl From<&ServiceRequest> for SubjectCollection {
        fn from(req: &ServiceRequest) -> Self {
            let maybe_user_id = req.extensions().get::<UserId>().map(Subject::from);

            Self(
                [maybe_user_id, Some(Subject::Unauthenticated)]
                    .into_iter()
                    .flatten()
                    .collect(),
            )
        }
    }
}
