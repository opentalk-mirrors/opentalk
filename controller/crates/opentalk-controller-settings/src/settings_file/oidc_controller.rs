// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{ClientId, ClientSecret};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct OidcController {
    pub authority: Option<Url>,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

impl PartialEq for OidcController {
    fn eq(&self, other: &Self) -> bool {
        self.authority.eq(&other.authority)
            && self.client_id.eq(&other.client_id)
            && self.client_secret.secret().eq(other.client_secret.secret())
    }
}

impl Eq for OidcController {}
