// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::{core::CoreProviderMetadata, IssuerUrl};
use reqwest::{
    header::AUTHORIZATION, redirect::Policy, Client, ClientBuilder, RequestBuilder, Response,
    StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use url::Url;

use crate::{Error, Error::OidcMetadataError, Result};

#[derive(Debug, Serialize)]
struct Credentials {
    client_id: String,
    client_secret: String,
}

#[derive(Debug)]
pub struct AuthorizedClient {
    credentials: Credentials,

    token: RwLock<Option<String>>,

    base_url: Url,

    client: reqwest::Client,
}

impl AuthorizedClient {
    pub fn new(base_url: Url, client_id: String, client_secret: String) -> Result<Self> {
        if base_url.cannot_be_a_base() {
            return Err(Error::NotBaseUrl { url: base_url });
        }

        // Using Policy::none() to prevent SSRF vulnerabilities, see
        // https://github.com/ramosbugs/openidconnect-rs/blob/main/UPGRADE.md#use-stateful-http-clients
        let client = ClientBuilder::new().redirect(Policy::none()).build()?;

        Ok(Self {
            credentials: Credentials {
                client_id,
                client_secret,
            },
            token: RwLock::new(None),
            base_url,
            client,
        })
    }

    /// Internal function used to aquire a new token when an `401 Unauthorized` http response was received
    ///
    /// Acquires the internal token and checks if it has changed from the current one or else
    /// calls `get_access_token` to get a new one from keycloak which it then saves.
    async fn update_token(&self, old_token: Option<String>) -> Result<String> {
        let mut token = self.token.write().await;

        if let (Some(current_token), Some(old_token)) = (&*token, old_token) {
            if current_token != &old_token {
                return Ok(current_token.clone());
            }
        }

        let new_token = self.get_access_token().await?;

        *token = Some(new_token.clone());

        Ok(new_token)
    }

    /// Internal function to aquire a new admin access token
    ///
    /// Uses the `client_credentials` grant to authorize using `client_id` and `client_secret`
    async fn get_access_token(&self) -> Result<String> {
        let issuer_url =
            IssuerUrl::new(self.base_url.to_string()).map_err(|_| OidcMetadataError {
                url: self.base_url.clone(),
            })?;

        let metadata = CoreProviderMetadata::discover_async(issuer_url, &self.client)
            .await
            .map_err(|_| OidcMetadataError {
                url: self.base_url.clone(),
            })?;

        #[derive(Serialize)]
        struct RequestBody<'cred> {
            grant_type: &'static str,
            #[serde(flatten)]
            credentials: &'cred Credentials,
        }

        #[derive(Deserialize)]
        struct ResponseBody {
            access_token: String,
        }

        let url = metadata
            .token_endpoint()
            .cloned()
            .map(|t| t.url().clone())
            .ok_or_else(|| OidcMetadataError {
                url: self.base_url.clone(),
            })?;

        let response = self
            .client
            .post(url)
            .form(&RequestBody {
                grant_type: "client_credentials",
                credentials: &self.credentials,
            })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::InvalidCredentials);
        }

        let response: ResponseBody = response.json().await?;

        Ok(response.access_token)
    }

    /// Internal function to send a request by passing a closure which constructs the request to send
    ///
    /// Will try the request with the clients current token once and if the token has expired it will
    /// refresh the token and try the request again once
    pub(crate) async fn send_authorized(
        &self,
        f: impl Fn(&Client) -> RequestBuilder,
    ) -> Result<Response> {
        let token = {
            let token = self.token.read().await;

            if let Some(token) = &*token {
                token.clone()
            } else {
                drop(token);
                self.update_token(None).await?
            }
        };

        let response = f(&self.client)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            let token = self.update_token(Some(token)).await?;

            let response = f(&self.client)
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .send()
                .await?;

            Ok(response)
        } else {
            Ok(response)
        }
    }
}
