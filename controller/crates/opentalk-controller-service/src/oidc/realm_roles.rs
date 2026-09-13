// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::rc::Rc;

/// Contains a list of string representing the service-account's roles in a realm
///
/// Roles can be used to represent certain permissions a service-account has
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealmRoles(pub Rc<[String]>);

impl RealmRoles {
    /// Query whether a role is contained in the list of realm roles
    pub fn contains(&self, role: &str) -> bool {
        self.0.iter().any(|r| r == role)
    }
}
