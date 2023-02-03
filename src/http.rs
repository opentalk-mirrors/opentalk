// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! HTTP calls made by this library (except for websockets)

use crate::settings::{AuthSettings, ControllerSettings};
use crate::websocket::Ticket;
use anyhow::{bail, Result};
use openidconnect::reqwest::Error;
use openidconnect::{AccessToken, OAuth2TokenResponse};
use openidconnect::{HttpRequest, HttpResponse};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::RwLock;

pub struct HttpClient {
    client: reqwest::Client,
    oidc: openidconnect::core::CoreClient,
    access_token: RwLock<AccessToken>,
}

impl HttpClient {
    pub async fn discover(settings: &AuthSettings) -> Result<Self> {
        let client = reqwest::Client::new();

        let metadata = openidconnect::core::CoreProviderMetadata::discover_async(
            settings.issuer.clone(),
            async_http_client(client.clone()),
        )
        .await?;

        let oidc = openidconnect::core::CoreClient::new(
            settings.client_id.clone(),
            Some(settings.client_secret.clone()),
            settings.issuer.clone(),
            metadata.authorization_endpoint().clone(),
            metadata.token_endpoint().cloned(),
            None,
            metadata.jwks().clone(),
        );

        let response = oidc
            .exchange_client_credentials()
            .request_async(async_http_client(client.clone()))
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
            .request_async(async_http_client(self.client.clone()))
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

fn async_http_client(
    client: reqwest::Client,
) -> impl Fn(
    HttpRequest,
) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Error<reqwest::Error>>> + Send>> {
    move |request| Box::pin(async_http_client_inner(client.clone(), request))
}

async fn async_http_client_inner(
    client: reqwest::Client,
    request: HttpRequest,
) -> Result<HttpResponse, Error<reqwest::Error>> {
    let mut request_builder = client
        .request(request.method, request.url.as_str())
        .body(request.body);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name.as_str(), value.as_bytes());
    }
    let request = request_builder.build().map_err(Error::Reqwest)?;

    let response = client.execute(request).await.map_err(Error::Reqwest)?;

    let status_code = response.status();
    let headers = response.headers().to_owned();
    let chunks = response.bytes().await.map_err(Error::Reqwest)?;
    Ok(HttpResponse {
        status_code,
        headers,
        body: chunks.to_vec(),
    })
}
