// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{OidcController, OidcFrontend};

/// The OIDC configuration for the OpenTalk controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Oidc {
    /// The OIDC configuration used by the controller to authenticate against the authority.
    pub controller: OidcController,

    /// The OIDC sent to the frontend for authenticating against the authority.
    pub frontend: OidcFrontend,
}
