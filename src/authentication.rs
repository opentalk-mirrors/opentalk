use std::time::{Duration, SystemTime};

use crate::config::{ClientCredentialsAuthConfig, DirectionAccessCredentialsAuthConfig};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc, serde::ts_seconds::deserialize as from_ts};
use jsonwebtoken::{DecodingKey, Validation, decode};
use openidconnect::{
    AccessToken, EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointNotSet, EndpointSet,
    IdTokenFields, OAuth2TokenResponse, RevocationErrorResponseType, StandardErrorResponse,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
    core::{
        CoreAuthDisplay, CoreAuthPrompt, CoreClient, CoreErrorResponseType, CoreGenderClaim,
        CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm,
        CoreProviderMetadata, CoreRevocableToken, CoreTokenType,
    },
};
use serde::Deserialize;
use tokio::sync::Mutex;

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

#[async_trait]
pub trait Authentication: Send {
    async fn access_token(&self) -> Result<AccessToken>;
}

pub(crate) struct ClientCredentialsAuthentication {
    http_client: reqwest::Client,
    oidc_client: OidcClient,
    current_token: Mutex<AccessToken>,
}

impl ClientCredentialsAuthentication {
    pub async fn create(
        http_client: reqwest::Client,
        config: ClientCredentialsAuthConfig,
    ) -> Result<Self> {
        let metadata = CoreProviderMetadata::discover_async(config.issuer.clone(), &http_client)
            .await
            .context("Failed to discover OIDC metadata")?;

        let oidc_client = CoreClient::new(
            config.client_id.clone(),
            config.issuer.clone(),
            metadata.jwks().clone(),
        )
        .set_client_secret(config.client_secret.clone())
        .set_auth_uri(metadata.authorization_endpoint().clone())
        .set_token_uri(
            metadata
                .token_endpoint()
                .context("Missing token endpoint url in OIDC metadata")?
                .clone(),
        );

        let response = oidc_client
            .exchange_client_credentials()
            .request_async(&http_client)
            .await
            .context("Failed to exchange client credentials")?;

        Ok(Self {
            http_client,
            oidc_client,
            current_token: response.access_token().clone().into(),
        })
    }
}

#[async_trait]
impl Authentication for ClientCredentialsAuthentication {
    async fn access_token(&self) -> Result<AccessToken> {
        let mut current_token = self.current_token.lock().await;

        if !check_if_token_is_expired(current_token.secret())? {
            return Ok(current_token.clone());
        }

        let response = self
            .oidc_client
            .exchange_client_credentials()
            .request_async(&self.http_client)
            .await
            .context("Failed to exchange client credentials")?;

        *current_token = response.access_token().clone();

        Ok(current_token.clone())
    }
}

pub(crate) struct DirectAccessCredentialsAuthentication {
    config: DirectionAccessCredentialsAuthConfig,

    http_client: reqwest::Client,
    oidc_client: OidcClient,
    current_token: Mutex<AccessToken>,
}

impl DirectAccessCredentialsAuthentication {
    pub async fn create(
        http_client: reqwest::Client,
        config: DirectionAccessCredentialsAuthConfig,
    ) -> Result<Self> {
        let metadata = CoreProviderMetadata::discover_async(config.issuer.clone(), &http_client)
            .await
            .context("Failed to discover OIDC metadata")?;

        let oidc_client = CoreClient::new(
            config.client_id.clone(),
            config.issuer.clone(),
            metadata.jwks().clone(),
        )
        .set_client_secret(config.client_secret.clone())
        .set_auth_uri(metadata.authorization_endpoint().clone())
        .set_token_uri(
            metadata
                .token_endpoint()
                .context("Missing token endpoint url in OIDC metadata")?
                .clone(),
        );

        let response = oidc_client
            .exchange_password(&config.username, &config.password)
            .request_async(&http_client)
            .await
            .context("Failed to exchange username & password")?;

        Ok(Self {
            config,
            http_client,
            oidc_client,
            current_token: response.access_token().clone().into(),
        })
    }
}

#[async_trait]
impl Authentication for DirectAccessCredentialsAuthentication {
    async fn access_token(&self) -> Result<AccessToken> {
        let mut current_token = self.current_token.lock().await;

        if !check_if_token_is_expired(current_token.secret())? {
            return Ok(current_token.clone());
        }

        let response = self
            .oidc_client
            .exchange_password(&self.config.username, &self.config.password)
            .request_async(&self.http_client)
            .await
            .context("Failed to exchange username & password")?;

        *current_token = response.access_token().clone();

        Ok(current_token.clone())
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
