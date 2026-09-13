// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::{
    auth::{
        GetLoginResponseBody, LogoutToken, PostLoginResponseBody, login::AuthLoginPostRequestBody,
    },
    error::{ApiError, AuthenticationError},
};

use crate::{ControllerBackend, oidc::VerifyError};

impl ControllerBackend {
    pub(crate) async fn get_login(&self) -> GetLoginResponseBody {
        GetLoginResponseBody {
            oidc: self.frontend_oidc_provider.clone(),
        }
    }

    pub(crate) async fn post_login(
        &self,
        body: AuthLoginPostRequestBody,
    ) -> Result<PostLoginResponseBody, ApiError> {
        if let Err(e) = self.oidc_token_handler.verify_id_token(&body.id_token) {
            return match e {
                VerifyError::InvalidClaims => Err(ApiError::bad_request()
                    .with_code("invalid_claims")
                    .with_message("some required attributes are missing or malformed")),
                VerifyError::Expired { .. } => Err(ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::SessionExpired)),
                VerifyError::MissingKeyID
                | VerifyError::UnknownKeyID
                | VerifyError::MalformedSignature
                | VerifyError::InvalidJwt { .. }
                | VerifyError::InvalidSignature => Err(ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::InvalidIdToken)),
            };
        };

        Ok(PostLoginResponseBody {
            permissions: Default::default(),
        })
    }

    pub(crate) async fn post_logout(&self, logout_token: &LogoutToken) -> Result<(), ApiError> {
        let result = self.oidc_token_handler.verify_logout_token(logout_token);

        let sub = result.map_err(|_| {
            log::warn!("Received an invalid logout token");
            ApiError::bad_request()
                .with_code("invalid_token")
                .with_message("provided logout token is invalid")
        })?;

        self.oidc_cache
            .upsert_sub_logout_marker(sub)
            .await
            .map_err(|_| {
                log::warn!("Could not update logout marker in cache");
                ApiError::bad_request()
                    .with_code("service_unavailable")
                    .with_message("an internal error occured")
            })?;

        Ok(())
    }
}
