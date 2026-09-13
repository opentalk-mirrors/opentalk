// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::OidcProvider;

/// Body of the response to a *GET* request on `/auth/login`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct GetLoginResponseBody {
    /// Description of the OIDC provider to use for the login
    pub oidc: OidcProvider,
}
