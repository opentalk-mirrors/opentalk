// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::ClientId;
use url::Url;

/// The OIDC configuration which can be sent to the frontend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcFrontend {
    /// The URL of the OIDC authority.
    pub authority: Url,

    /// The client id to be used when authenticating the frontend.
    pub client_id: ClientId,
}
