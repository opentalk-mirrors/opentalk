// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::UserSearchBackendKeycloak;

/// The definition of available user search backends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserSearchBackend {
    /// Search backend querying results through the Keycloak web api.
    Keycloak(UserSearchBackendKeycloak),
}
