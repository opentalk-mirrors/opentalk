// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::ClientId;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct OidcFrontend {
    pub authority: Option<Url>,
    pub client_id: ClientId,
}
