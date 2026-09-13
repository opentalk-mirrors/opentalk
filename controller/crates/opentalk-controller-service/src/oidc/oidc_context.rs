// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{ops::Deref, str::FromStr as _};

use icu_locid::LanguageIdentifier;
use openidconnect::{
    AccessToken, ClientId, ClientSecret, LocalizedClaim, SubjectIdentifier,
    TokenIntrospectionResponse as _, UserInfoClaims, core::CoreGenderClaim,
};
use opentalk_controller_utils::CaptureApiError;
use opentalk_types_api_v1::{
    auth::LogoutToken,
    error::{ApiError, AuthenticationError},
};
use opentalk_types_common::time::TimeZone;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use snafu::{OptionExt as _, Report, ResultExt as _, Whatever};
use url::Url;

use super::{
    IntrospectInfo, JWTAccessTokenClaims, JWTLogoutTokenClaims, OidcTokenHandler, OnlyExpiryClaim,
    OpenIdConnectUserInfo, OpenTalkAdditionalClaims, ProviderClient, RealmRoles, ServiceClaims,
    VerificationInfo, VerifyError, jwt,
};
use crate::Result;

pub fn make_client(default_headers: Option<HeaderMap>) -> Result<reqwest::Client, reqwest::Error> {
    default_headers
        .map_or(reqwest::Client::builder(), |headers| {
            reqwest::Client::builder().default_headers(headers)
        })
        .redirect(reqwest::redirect::Policy::none())
        .build()
}

/// The `OidcContext` contains all information about the Oidc provider and permissions matrix.
#[derive(Debug)]
pub(super) struct OidcContext {
    /// The provider client
    provider: ProviderClient,
    /// The HTTP client
    http_client: opentalk_keycloak_admin::reqwest::ClientWrapper,

    /// The HTTP client for calling the introspection endpoint.
    ///
    /// In some cases, e.g. when running in a test setup, the client must send different kinds
    /// of requests when performing the OIDC introspection (mainly because Keycloak requires
    /// the introspection request go to the same host as the one that issued the frontend token).
    ///
    /// This will be set to a different client if:
    /// - The `OIDC_INTROSPECT_AND_USERINFO_SET_X_FORWARDED_HOST` environment
    ///   variable is set to `true` (case-insensitive) or `1`.
    /// - The `frontend_auth_base_url` differs from the `controller_auth_base_url`.
    ///
    /// This client adds a `x-forwarded-host` header derived from the frontend auth base url,
    /// using its host part, and if present, the port as well.
    http_introspect_and_userinfo_client: Option<opentalk_keycloak_admin::reqwest::ClientWrapper>,
}

impl OidcContext {
    /// Creates the OidcContext.
    /// This reads the OIDC provider configuration and tries to fetch the metadata from it.
    /// If a provider is misconfigured or not reachable this function will fail.
    #[tracing::instrument(name = "oidc_discover", skip(client_secret))]
    pub(super) async fn new(
        frontend_auth_base_url: Url,
        controller_auth_base_url: Url,
        client_id: ClientId,
        client_secret: ClientSecret,
    ) -> Result<Self> {
        let http_client: opentalk_keycloak_admin::reqwest::ClientWrapper = make_client(None)
            .whatever_context("Failed to make http client")?
            .into();

        let http_introspect_and_userinfo_client =
            match std::env::var("OIDC_INTROSPECT_AND_USERINFO_SET_X_FORWARDED_HOST").ok() {
                Some(v) if v.eq_ignore_ascii_case("true") || v == "1" => {
                    let host = frontend_auth_base_url
                        .host_str()
                        .whatever_context("Frontend auth base url doesn't have a host part")?;
                    let port_appendix = frontend_auth_base_url
                        .port()
                        .map(|p| format!(":{p}"))
                        .unwrap_or_default();
                    let forwarded_host = format!("{host}{port_appendix}");
                    let mut default_headers = HeaderMap::new();
                    _ = default_headers.insert(
                        HeaderName::from_str("X-Forwarded-Host")
                            .whatever_context("Invalid header header name for introspect client")?,
                        HeaderValue::from_str(&forwarded_host)
                            .whatever_context("Invalid header value for introspect client")?,
                    );
                    Some(
                        make_client(Some(default_headers))
                            .whatever_context("Failed to make oidc introspection http client")?
                            .into(),
                    )
                }
                Some(_) | None => None,
            };

        let client = ProviderClient::discover(
            http_client.clone(),
            controller_auth_base_url,
            client_id,
            client_secret,
        )
        .await
        .whatever_context("Failed to discover provider client")?;

        Ok(Self {
            provider: client,
            http_client,
            http_introspect_and_userinfo_client,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn check_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Result<RealmRoles, CaptureApiError> {
        let claims = match self.verify_jwt_token::<ServiceClaims>(access_token.secret().as_str()) {
            Ok(claims) => claims,
            Err(e) => {
                log::error!("Invalid access token, {}", Report::from_error(e));
                return Err(ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::InvalidAccessToken)
                    .into());
            }
        };

        let mut realm_roles = claims.realm_access.roles;
        realm_roles
            .iter_mut()
            .for_each(|role| role.make_ascii_lowercase());

        Ok(RealmRoles(realm_roles.into()))
    }

    #[tracing::instrument(skip_all)]
    async fn verify_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerificationInfo, CaptureApiError> {
        // Verify the access token via introspection.
        // Even if the access token is a JWT, we prefer introspection if available,
        // because it also checks if the token is active
        if self.supports_introspect() {
            let introspect_info = self.introspect(access_token.clone()).await.map_err(|e| {
                log::error!(
                    "Failed to introspect access token: {}",
                    Report::from_error(e)
                );
                ApiError::internal()
            })?;

            if !introspect_info.active {
                return Err(ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::AccessTokenInactive)
                    .into());
            }

            return Ok(VerificationInfo::from(introspect_info));
        }

        // Access token format must correspond [rfc9068](https://datatracker.ietf.org/doc/html/rfc9068)
        // to be able to verify it locally
        match self.verify_jwt_token::<JWTAccessTokenClaims>(access_token.secret().as_str()) {
            Ok(claims) => {
                return Ok(VerificationInfo {
                    exp: Some(claims.exp),
                    sub: Some(claims.sub),
                });
            }
            Err(_) => {
                log::error!(
                    "Access token can not be verified: OIDC provider does not support introspection and token is not a valid JWT"
                );
                return Err(ApiError::unauthorized()
                    .with_www_authenticate(AuthenticationError::InvalidAccessToken)
                    .into());
            }
        }
    }

    #[tracing::instrument(skip_all)]
    fn verify_logout_token(
        &self,
        logout_token: &LogoutToken,
    ) -> Result<SubjectIdentifier, VerifyError> {
        let claims = self.verify_jwt_token::<JWTLogoutTokenClaims>(logout_token.as_str())?;

        if !claims
            .events
            .contains_key("http://schemas.openid.net/event/backchannel-logout")
        {
            return Err(VerifyError::InvalidClaims);
        }

        Ok(SubjectIdentifier::new(claims.sub))
    }

    /// Verifies that a JWT token is valid and contains specified claims
    #[tracing::instrument(name = "oidc_verify_jwt_token", skip(self, token))]
    fn verify_jwt_token<C: jwt::VerifyClaims>(&self, token: &str) -> Result<C, VerifyError> {
        jwt::verify::<C>(self.provider.metadata.jwks(), token)
    }

    /// Returns if the configured provider support introspection
    fn supports_introspect(&self) -> bool {
        self.provider
            .metadata
            .additional_metadata()
            .introspection_endpoint
            .is_some()
    }

    /// Call the OIDC's userinfo endpoint to fetch the user data associated with the access token
    #[tracing::instrument(name = "introspect", skip_all)]
    async fn introspect(&self, access_token: AccessToken) -> Result<IntrospectInfo> {
        let client = self
            .http_introspect_and_userinfo_client
            .as_ref()
            .unwrap_or(&self.http_client);
        let claims = self
            .provider
            .client
            .introspect(&access_token)
            .request_async(client)
            .await
            .whatever_context("AccessToken introspect request failed")?;

        Ok(IntrospectInfo {
            active: claims.active(),
            exp: claims.exp(),
            sub: claims.sub().map(|s| s.to_string()),
        })
    }

    /// Call the OIDC's userinfo endpoint to fetch the user data associated with the access token
    #[tracing::instrument(err(level = "debug"), name = "oidc_user_info", skip_all)]
    async fn user_info(
        &self,
        access_token: AccessToken,
    ) -> Result<OpenIdConnectUserInfo, CaptureApiError> {
        let client = self
            .http_introspect_and_userinfo_client
            .as_ref()
            .unwrap_or(&self.http_client);
        let claims: UserInfoClaims<OpenTalkAdditionalClaims, CoreGenderClaim> = self
            .provider
            .client
            .user_info(access_token, None)
            .whatever_context::<_, Whatever>("Failed to build userinfo request")?
            .request_async(client)
            .await
            .whatever_context::<_, Whatever>("Failed to fetch userinfo")?;

        let locale_parse_result = claims.locale().map(|loc| (loc, loc.parse()));
        let locale: Option<LanguageIdentifier> = match locale_parse_result {
            // Locale exists and has correct BCP47 format
            Some((_, Ok(lang))) => Some(lang),
            // Locale exists but has wrong format
            Some((loc, Err(_))) => {
                log::warn!(
                    "Invalid locale value in token for OIDC sub {}: \"{}\"",
                    claims.subject().as_str(),
                    loc.as_str(),
                );
                None
            }
            None => None,
        };

        let timezone_parse_result = claims.zoneinfo().map(|zi| (zi, zi.parse()));
        let timezone: Option<TimeZone> = match timezone_parse_result {
            // Zoneinfo exists and has correct IANA format
            Some((_, Ok(tz))) => Some(tz),
            // Zoneinfo exists but has wrong format
            Some((zi, Err(_))) => {
                log::warn!(
                    "Invalid zoneinfo value in token for OIDC sub {}: \"{}\"",
                    claims.subject().as_str(),
                    zi.as_str(),
                );
                None
            }
            None => None,
        };

        fn expect_present<'a, T>(t: Option<&'a T>, name: &str) -> Result<&'a T, CaptureApiError> {
            match t {
                Some(t) => Ok(t),
                None => Err(ApiError::bad_request()
                    .with_message(format!(
                        "userinfo claims are missing mandatory '{name}' field"
                    ))
                    .into()),
            }
        }

        fn optional<T: Deref<Target = String>>(t: Option<&LocalizedClaim<T>>) -> Option<String> {
            t.and_then(|c| c.get(None)).map(|c| c.to_string())
        }

        let email = {
            let raw_email = expect_present(claims.email(), "email")?.as_str();
            if raw_email.chars().any(|c| c.is_ascii_uppercase()) {
                log::warn!(
                    "Invalid uppercase email in token for OIDC sub {}: \"{}\"",
                    claims.subject().as_str(),
                    raw_email,
                );
            }
            raw_email.to_lowercase()
        };

        // Convert missing claims to empty strings to hotfix https://git.opentalk.dev/opentalk/product/tickets/-/work_items/223
        // Later on we should represent data correctly, storing missing claims as `null` in the database or passing them to the frontend
        let firstname = claims
            .given_name()
            .and_then(|claim| claim.get(None))
            .map(|first_name| first_name.to_string())
            .unwrap_or_default();
        let lastname = claims
            .family_name()
            .and_then(|claim| claim.get(None))
            .map(|family_name| family_name.to_string())
            .unwrap_or_default();

        Ok(OpenIdConnectUserInfo {
            sub: claims.subject().to_string(),
            email,
            firstname,
            lastname,
            avatar_url: optional(claims.picture()),
            locale,
            timezone,
            groups: claims.additional_claims().x_grp.clone(),
            phone_number: claims
                .phone_number()
                .map(|phone_number| phone_number.to_string()),
            display_name: optional(claims.nickname()),
            tenant_id: claims.additional_claims().tenant_id.clone(),
            tariff_id: claims.additional_claims().tariff_id.clone(),
            tariff_status: claims.additional_claims().tariff_status.clone(),
        })
    }

    /// Verifies the signature and expiration of an ID Token encoded as JWT (Json Web Token)
    ///
    /// Only used by the deprecated login endpoint
    #[tracing::instrument(name = "oidc_verify_id_token", skip_all)]
    fn verify_id_token(&self, id_token: &str) -> Result<(), VerifyError> {
        let _ = jwt::verify::<OnlyExpiryClaim>(self.provider.metadata.jwks(), id_token)?;
        Ok(())
    }
}

#[async_trait::async_trait(?Send)]
impl OidcTokenHandler for OidcContext {
    async fn user_info(
        &self,
        access_token: AccessToken,
    ) -> Result<OpenIdConnectUserInfo, CaptureApiError> {
        OidcContext::user_info(self, access_token).await
    }

    async fn check_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Result<RealmRoles, CaptureApiError> {
        OidcContext::check_access_token(self, access_token).await
    }

    async fn verify_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Result<VerificationInfo, CaptureApiError> {
        OidcContext::verify_access_token(self, access_token).await
    }

    fn verify_id_token(&self, id_token: &str) -> Result<(), VerifyError> {
        OidcContext::verify_id_token(self, id_token)
    }

    fn verify_logout_token(
        &self,
        logout_token: &LogoutToken,
    ) -> Result<SubjectIdentifier, VerifyError> {
        OidcContext::verify_logout_token(self, logout_token)
    }
}
