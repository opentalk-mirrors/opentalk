// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{ClientId, ClientSecret};
use url::Url;

/// The OIDC configuration which is used to authenticate the controller.
#[derive(Debug, Clone)]
pub struct OidcController {
    /// The URL of the OIDC authority.
    pub authority: Url,

    /// The client id to be used when authenticating the controller.
    pub client_id: ClientId,

    /// The client secret to be used when authenticating the controller.
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
