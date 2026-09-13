// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{ClientId, ClientSecret};
use url::Url;

/// Search backend querying results through the Keycloak web api.
#[derive(Debug, Clone)]
pub struct UserSearchBackendKeycloak {
    /// The base url for the Keycloak API.
    pub api_base_url: Url,

    /// The client id for authenticating against the Keycloak api.
    pub client_id: ClientId,

    /// The client secret for authenticating against the Keycloak api.
    pub client_secret: ClientSecret,

    /// The name of the attribute that is used as the external user id.
    pub external_id_user_attribute_name: Option<String>,
}

impl PartialEq for UserSearchBackendKeycloak {
    fn eq(&self, other: &Self) -> bool {
        self.api_base_url.eq(&other.api_base_url)
            && self.client_id.eq(&other.client_id)
            && self.client_secret.secret().eq(other.client_secret.secret())
            && self
                .external_id_user_attribute_name
                .eq(&other.external_id_user_attribute_name)
    }
}

impl Eq for UserSearchBackendKeycloak {}
