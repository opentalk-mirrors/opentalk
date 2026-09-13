// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{ClientId, ClientSecret};
use serde::Deserialize;
use url::Url;

/// Settings for Keycloak
#[derive(Debug, Clone, Deserialize)]
pub struct Keycloak {
    pub base_url: Url,
    pub realm: String,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub external_id_user_attribute_name: Option<String>,
}

impl PartialEq for Keycloak {
    fn eq(&self, other: &Self) -> bool {
        self.base_url.eq(&other.base_url)
            && self.realm.eq(&other.realm)
            && self.client_id.eq(&other.client_id)
            && self.client_secret.secret().eq(other.client_secret.secret())
            && self
                .external_id_user_attribute_name
                .eq(&other.external_id_user_attribute_name)
    }
}

impl Eq for Keycloak {}
