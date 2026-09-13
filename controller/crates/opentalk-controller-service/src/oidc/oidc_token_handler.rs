// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use openidconnect::{AccessToken, ClientId, ClientSecret, SubjectIdentifier};
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_v1::auth::LogoutToken;
use url::Url;

use super::{OidcContext, OpenIdConnectUserInfo, RealmRoles, VerificationInfo, VerifyError};
use crate::Result;

/// The handler for OIDC tokens
#[async_trait::async_trait(?Send)]
pub trait OidcTokenHandler: Sync + Send {
    /// Call the OIDC's userinfo endpoint to fetch the user data associated with the access token
    async fn user_info(
        &self,
        access_token: AccessToken,
    ) -> Result<OpenIdConnectUserInfo, CaptureApiError>;

    /// Check an access token
    async fn check_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Result<RealmRoles, CaptureApiError>;

    /// Verify the access token and return its expiry timestamp.
    ///
    /// Uses the introspect endpoint if available [RFC7662](https://www.rfc-editor.org/rfc/rfc7662),
    /// otherwise the token must be a JWT.
    async fn verify_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerificationInfo, CaptureApiError>;

    /// Verifies the signature and expiration of an ID Token encoded as JWT (Json Web Token)
    ///
    /// Only used by the deprecated login endpoint
    fn verify_id_token(&self, id_token: &str) -> Result<(), VerifyError>;

    /// Verify the logout token and returns the subject of the token
    fn verify_logout_token(
        &self,
        logout_token: &LogoutToken,
    ) -> Result<SubjectIdentifier, VerifyError>;
}

/// Build the oidc token handler
pub async fn build_oidc_token_handler(
    frontend_auth_base_url: Url,
    controller_auth_base_url: Url,
    client_id: ClientId,
    client_secret: ClientSecret,
) -> Result<Arc<dyn OidcTokenHandler>> {
    let oidc_context = OidcContext::new(
        frontend_auth_base_url,
        controller_auth_base_url,
        client_id,
        client_secret,
    )
    .await?;
    Ok(Arc::new(oidc_context))
}
