// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::AccessToken;
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_service::oidc::{Cache, OidcTokenHandler};
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::{InventoryProvider, Tenant, User};

use super::provisioning;

/// Authenticate user via access token and provision user data
/// Cache authentication results for access tokens to optimize subsequent authentications with the same token
/// Return user profile on success or an appropriate error on failure
#[tracing::instrument(skip_all)]
pub(super) async fn authenticate_user(
    settings: &Settings,
    authorizer: &Authorizer,
    inventory_provider: &dyn InventoryProvider,
    oidc_ctx: &dyn OidcTokenHandler,
    oidc_cache: &Cache,
    access_token: &AccessToken,
) -> Result<(Tenant, User), CaptureApiError> {
    // Check if access token has been cached already
    if let Some(result) = oidc_cache.get_access_token(access_token).await {
        return result;
    }

    // Verifiy access token which is not cached yet
    // On verification error, cache the error and return early
    let verification_result = oidc_ctx.verify_access_token(access_token).await;
    match verification_result {
        Ok(verification_info) => {
            // Provision user data from a valid access token
            // Cache retreival errors which are relevant for the authentication result
            let user_profile_result = provisioning::provision_user(
                settings,
                authorizer,
                inventory_provider,
                oidc_ctx,
                access_token,
            )
            .await;
            oidc_cache
                .insert_access_token(
                    access_token,
                    user_profile_result.clone(),
                    verification_info.exp,
                )
                .await
                .is_err()
                .then(|| tracing::warn!("Failed to cache user data error for access token"));

            user_profile_result
        }
        Err(error) => {
            oidc_cache
                .insert_access_token(access_token, Err(error.clone()), None)
                .await
                .is_err()
                .then(|| tracing::warn!("Failed to cache verification error for access token"));
            return Err(error);
        }
    }
}
