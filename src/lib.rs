// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use reqwest::{Client, RequestBuilder, Response};
use snafu::Snafu;
use url::Url;

mod authorized_client;

pub mod users;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub use authorized_client::AuthorizedClient;

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

/// HTTP client to access some Keycloak admin APIs
pub struct KeycloakAdminClient {
    api_base_url: Url,

    authorized_client: AuthorizedClient,
}

impl KeycloakAdminClient {
    /// Create a new client from all required configurations
    pub fn new(api_base_url: Url, authorized_client: AuthorizedClient) -> Result<Self, Error> {
        if api_base_url.cannot_be_a_base() {
            return Err(Error::NotBaseUrl { url: api_base_url });
        }

        Ok(Self {
            api_base_url,
            authorized_client,
        })
    }

    fn api_url<I>(&self, path_segments: I) -> Result<Url>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        build_url(self.api_base_url.clone(), path_segments)
    }

    async fn send_authorized(&self, f: impl Fn(&Client) -> RequestBuilder) -> Result<Response> {
        self.authorized_client.send_authorized(f).await
    }
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
