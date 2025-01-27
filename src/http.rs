// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! HTTP calls made by this library (except for websockets)

use crate::settings::{AuthSettings, ControllerSettings};
use crate::websocket::Ticket;
use anyhow::{bail, Context, Result};
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

    async fn refresh_access_tokens(&self, invalid_token: AccessToken) -> Result<()> {
        let mut token = self.access_token.write().await;

        if token.secret() != invalid_token.secret() {
            return Ok(());
        }

        let response = self
            .oidc
            .exchange_client_credentials()
            .request_async(&self.client)
            .await?;

        *token = response.access_token().clone();

        Ok(())
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

        // max 10 tries
        for _ in 0..10 {
            let token = {
                // Scope the access to the lock to avoid holding it for the entire loop-body
                let l = self.access_token.read().await;
                l.clone()
            };

            let response = self
                .client
                .post(&uri)
                .header("content-type", "application/json")
                .bearer_auth(token.secret())
                .json(&StartRequest { id, pin })
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => {
                    let response = response.json::<StartResponse>().await?;

                    return Ok(Ticket(response.ticket));
                }
                StatusCode::UNAUTHORIZED => {
                    let ApiError { code } = response.json::<ApiError>().await?;

                    if code == "unauthorized" {
                        self.refresh_access_tokens(token).await?;
                    } else {
                        bail!(InvalidCredentials);
                    }
                }
                StatusCode::BAD_REQUEST => {
                    let ApiError { code } = response.json::<ApiError>().await?;

                    if code == "invalid_credentials" {
                        bail!(InvalidCredentials);
                    }
                }
                code => bail!("unexpected status code {code:?}"),
            }
        }

        bail!("failed to authenticate")
    }
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
struct ApiError {
    code: String,
}

#[derive(Deserialize)]
struct StartResponse {
    ticket: String,
}
