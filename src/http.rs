// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! HTTP calls made by this library (except for websockets)

use anyhow::{Context, Result, bail};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    authentication::{
        Authentication, ClientCredentialsAuthentication, DirectAccessCredentialsAuthentication,
    },
    config::{AuthConfig, ControllerConfig},
};

pub(crate) struct HttpClient {
    http_client: reqwest::Client,
    authentication: Box<dyn Authentication>,
}

impl HttpClient {
    pub(crate) async fn create(config: AuthConfig) -> Result<Self> {
        let http_client = reqwest::Client::new();

        let authentication: Box<dyn Authentication> = match config {
            AuthConfig::ClientCredentials(config) => Box::new(
                ClientCredentialsAuthentication::create(http_client.clone(), config).await?,
            ),
            AuthConfig::DirectAccess(config) => Box::new(
                DirectAccessCredentialsAuthentication::create(http_client.clone(), config).await?,
            ),
            AuthConfig::Other(authentication) => authentication,
        };

        Ok(Self {
            http_client,
            authentication,
        })
    }

    pub(crate) async fn start(
        &self,
        settings: &ControllerConfig,
        room_id: &str,
        breakout_room: Option<&str>,
    ) -> Result<String> {
        let uri = format!("{}/rooms/{room_id}/start", settings.v1_api_base_url());

        let token = self
            .authentication
            .access_token()
            .await
            .context("Failed to get access token from authentication backend")?;

        let response = self
            .http_client
            .post(&uri)
            .bearer_auth(token.secret())
            .json(&StartRequest {
                // room_id,
                breakout_room,
            })
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

                Ok(response.ticket)
            }
            code => bail!("unexpected status code {code:?}"),
        }
    }
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
