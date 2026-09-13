// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Outcome of an authorization check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// Admission to the requested resource is allowed.
    Allowed,

    /// Admission to the requested resource is denied.
    Denied,

    /// Prior authentication is required to request the resource.
    AuthenticationRequired,
}

impl Admission {
    /// Check whether the admission is [`Admission::Allowed`].
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

impl From<bool> for Admission {
    fn from(value: bool) -> Self {
        match value {
            true => Admission::Allowed,
            false => Admission::Denied,
        }
    }
}

impl From<Admission> for bool {
    fn from(value: Admission) -> Self {
        value.is_allowed()
    }
}
