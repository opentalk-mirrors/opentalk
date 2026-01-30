// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! HTTP calls made by this library (except for websockets)

use std::time::{Duration, SystemTime};

use crate::settings::{AuthSettings, ControllerSettings};
use crate::websocket::Ticket;
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc, serde::ts_seconds::deserialize as from_ts};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreClient, CoreErrorResponseType, CoreGenderClaim,
    CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
    CoreProviderMetadata, CoreRevocableToken, CoreTokenType,
};
use openidconnect::{
    AccessToken, EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointNotSet, EndpointSet,
    IdTokenFields, OAuth2TokenResponse, RevocationErrorResponseType, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

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

pub struct HttpClient {
    client: reqwest::Client,
    oidc: OidcClient,
    access_token: RwLock<AccessToken>,
}

impl HttpClient {
    pub async fn discover(settings: &AuthSettings) -> Result<Self> {
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
            .exchange_client_credentials()
            .request_async(&client)
            .await?;

        Ok(Self {
            client,
            oidc,
            access_token: RwLock::new(response.access_token().clone()),
        })
    }

    async fn get_valid_access_token(&self) -> Result<AccessToken> {
        let mut token = self.access_token.write().await;

        // Refresh access token if it's expired
        if check_if_token_is_expired(token.secret())? {
            log::trace!("stored access token has expired");

            let response = self
                .oidc
                .exchange_client_credentials()
                .request_async(&self.client)
                .await
                .context("Failed to request new access token")?;

            *token = response.access_token().clone();
        }

        Ok(token.clone())
    }

    /// Request a signaling ticket with the given DTMF digits `id` and `pin`
    ///
    /// The ticket is then used to establish a websocket connection
    pub async fn start(
        &self,
        settings: &ControllerSettings,
        id: &str,
        pin: &str,
    ) -> Result<Ticket> {
        let uri = if settings.insecure {
            log::warn!("using insecure connection");
            format!("http://{}/v1/services/call_in/start", settings.domain)
        } else {
            format!("https://{}/v1/services/call_in/start", settings.domain)
        };

        let token = self
            .get_valid_access_token()
            .await
            .context("Failed to retrieve valid AccessToken")?;

        let response = self
            .client
            .post(&uri)
            .bearer_auth(token.secret())
            .json(&StartRequest { id, pin })
            .send()
            .await?;

        let status = response.status();
        let response_body = response
            .text()
            .await
            .context("Failed to read HTTP response body")?;

        log::trace!("start response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<StartResponse>(&response_body)
                    .context("Failed to parse response body")?;

                Ok(Ticket(response.ticket))
            }
            StatusCode::BAD_REQUEST => bail!(InvalidCredentials),
            status => bail!("unexpected status code {status:?}"),
        }
    }
}

#[derive(Deserialize)]
struct TokenClaims {
    #[serde(deserialize_with = "from_ts")]
    exp: DateTime<Utc>,
}

/// Check if the token is expired or is expiring within a minute
fn check_if_token_is_expired(token: &str) -> Result<bool> {
    let token = jsonwebtoken::dangerous::insecure_decode::<TokenClaims>(token)?;

    let now = DateTime::<Utc>::from(SystemTime::now());

    log::trace!(
        "access token is still valid for {:?}",
        token.claims.exp.signed_duration_since(now)
    );

    Ok(now > token.claims.exp - Duration::from_secs(60))
}

/// Error returned by the `start` function when the given digits were incorrect
#[derive(Debug, thiserror::Error)]
#[error("given credentials were invalid")]
pub struct InvalidCredentials;

#[derive(Serialize)]
struct StartRequest<'s> {
    id: &'s str,
    pin: &'s str,
}

#[derive(Deserialize)]
struct StartResponse {
    ticket: String,
}
