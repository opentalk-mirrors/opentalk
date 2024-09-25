// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use reqwest::{header::AUTHORIZATION, Client, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use tokio::sync::RwLock;
use url::Url;

pub mod users;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Reqwest error: {}", source), context(false))]
    Reqwest { source: reqwest::Error },
    #[snafu(display("Keycloak error response"))]
    KeyCloak,
    #[snafu(display("Invalid credentials"))]
    InvalidCredentials,
    #[snafu(display("Given base URL is not a base: {}", url))]
    NotBaseUrl { url: Url },
}

#[derive(Serialize)]
struct Credentials {
    client_id: String,
    client_secret: String,
}

/// HTTP client to access some Keycloak admin APIs
pub struct KeycloakAdminClient {
    credentials: Credentials,

    api_base_url: Url,
    auth_base_url: Url,

    client: reqwest::Client,

    token: RwLock<Option<String>>,
}

impl KeycloakAdminClient {
    /// Create a new client from all required configurations
    pub fn new(
        api_base_url: Url,
        auth_base_url: Url,
        client_id: String,
        client_secret: String,
    ) -> Result<Self, Error> {
        if api_base_url.cannot_be_a_base() {
            return Err(Error::NotBaseUrl { url: api_base_url });
        }
        if auth_base_url.cannot_be_a_base() {
            return Err(Error::NotBaseUrl { url: auth_base_url });
        }

        Ok(Self {
            credentials: Credentials {
                client_id,
                client_secret,
            },
            api_base_url,
            auth_base_url,
            client: reqwest::Client::new(),
            token: RwLock::new(None),
        })
    }

    /// Internal function to aquire a new admin access token
    ///
    /// Uses the `client_credentials` grant to authorize using `client_id` and `client_secret`
    async fn get_access_token(&self) -> Result<String> {
        let url = self.auth_url(["protocol", "openid-connect", "token"])?;

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

    /// Internal function to send a request by passing a closure which constructs the request to send
    ///
    /// Will try the request with the clients current token once and if the token has expired it will
    /// refresh the token and try the request again once
    async fn send_authorized(&self, f: impl Fn(&Client) -> RequestBuilder) -> Result<Response> {
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

    fn api_url<I>(&self, path_segments: I) -> Result<Url>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::build_url(self.api_base_url.clone(), path_segments)
    }

    fn auth_url<I>(&self, path_segments: I) -> Result<Url>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        Self::build_url(self.auth_base_url.clone(), path_segments)
    }

    /// internal url builder
    fn build_url<I>(base_url: Url, path_segments: I) -> Result<Url>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let err_url = base_url.clone();
        let mut url = base_url;
        url.path_segments_mut()
            .map_err(|_| Error::NotBaseUrl { url: err_url })?
            .extend(path_segments);
        Ok(url)
    }
}
