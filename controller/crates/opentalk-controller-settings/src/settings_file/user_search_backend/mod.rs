// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Deserialize;

mod keycloak_webapi;

pub use keycloak_webapi::UserSearchBackendKeycloakWebapi;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "backend")]
pub enum UserSearchBackend {
    KeycloakWebapi(UserSearchBackendKeycloakWebapi),
}
