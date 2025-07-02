// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! HTTP calls made by this library (except for websockets)

use std::time::{Duration, SystemTime};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc, serde::ts_seconds::deserialize as from_ts};
use jsonwebtoken::{self, DecodingKey, Validation, decode};
use openidconnect::{
    AccessToken, EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointNotSet, EndpointSet,
    IdTokenFields, OAuth2TokenResponse, ResourceOwnerPassword, ResourceOwnerUsername,
    RevocationErrorResponseType, Scope, StandardErrorResponse, StandardTokenIntrospectionResponse,
    StandardTokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreClient, CoreErrorResponseType, CoreGenderClaim,
        CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
        CoreProviderMetadata, CoreRevocableToken, CoreTokenType,
    },
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::settings::{AuthSettings, ControllerSettings};

type OidcClient = openidconnect::Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            EmptyAdditionalClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        >,
        CoreTokenType,
    >,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, CoreTokenType>,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
    EndpointNotSet,
>;

#[derive(Debug)]
pub(crate) struct HttpClient {
    client: reqwest::Client,
    oidc: OidcClient,
    access_token: RwLock<AccessToken>,
}

impl HttpClient {
    pub(crate) async fn discover(settings: &AuthSettings) -> Result<Self> {
        let client = reqwest::Client::new();

        let metadata =
            CoreProviderMetadata::discover_async(settings.issuer.clone(), &client).await?;

        let oidc = CoreClient::new(
            settings.client_id.clone(),
            settings.issuer.clone(),
            metadata.jwks().clone(),
        )
        .set_client_secret(settings.client_secret.clone())
        .set_auth_uri(metadata.authorization_endpoint().clone())
        .set_token_uri(
            metadata
                .token_endpoint()
                .context("Missing token endpoint url in OIDC metadata")?
                .clone(),
        );

        let response = oidc
            // FIXME
            .exchange_password(
                &ResourceOwnerUsername::new(String::from("bob")),
                &ResourceOwnerPassword::new(String::from("bob")),
            )
            .add_scope(Scope::new(String::from("openid")))
            // .exchange_client_credentials()
            .request_async(&client)
            .await?;

        Ok(Self {
            client,
            oidc,
            access_token: RwLock::new(response.access_token().clone()),
        })
    }

    pub(crate) async fn start(
        &self,
        settings: &ControllerSettings,
        room_id: &str,
        breakout_room: Option<&str>,
    ) -> Result<String> {
        let uri = format!("{}/rooms/{room_id}/start", settings.v1_api_base_url());

        let token = self.get_valid_access_token().await?;

        let response = self
            .client
            .post(&uri)
            .bearer_auth(token.secret())
            .json(&StartRequest {
                // room_id,
                breakout_room,
            })
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let response = response.json::<StartResponse>().await?;

                Ok(response.ticket)
            }
            StatusCode::UNAUTHORIZED => {
                let ApiError { code } = response.json::<ApiError>().await?;

                bail!("failed to authorize, {code:?}");
            }
            code => bail!("unexpected status code {code:?}"),
        }
    }

    async fn get_valid_access_token(&self) -> Result<AccessToken> {
        let mut token = self.access_token.write().await;

        // Refresh access token if it's expired
        if check_if_token_is_expired(token.secret())? {
            let response = self
                .oidc
                .exchange_client_credentials()
                .request_async(&self.client)
                .await?;

            *token = response.access_token().clone();
        }

        Ok(token.clone())
    }
}

#[derive(Deserialize)]
struct TokenClaims {
    #[serde(deserialize_with = "from_ts")]
    exp: DateTime<Utc>,
}

/// Check if the token is expired or is expiring within a minute
fn check_if_token_is_expired(token: &str) -> Result<bool> {
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    validation.validate_aud = false;

    let token = decode::<TokenClaims>(token, &DecodingKey::from_secret(&[]), &validation)?;

    let now = DateTime::<Utc>::from(SystemTime::now());

    Ok(now > token.claims.exp + Duration::from_secs(60))
}

#[derive(Serialize)]
struct StartRequest<'s> {
    // resumption: &'s str,
    breakout_room: Option<&'s str>,
}

#[derive(Deserialize)]
struct ApiError {
    code: String,
}

#[derive(Deserialize)]
struct StartResponse {
    ticket: String,
}
