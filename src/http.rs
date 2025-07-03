// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result, bail};
use reqwest::StatusCode;
use types::{
    api::v1::{
        rooms::by_room_id::{
            PostRoomsStartInvitedRequestBody, PostRoomsStartRequestBody, RoomsStartResponseBody,
        },
        services::call_in::PostCallInStartRequestBody,
    },
    common::{
        auth::TicketToken,
        call_in::{CallInId, CallInPassword},
        rooms::{BreakoutRoomId, RoomId, RoomPassword},
    },
};

use crate::{
    authentication::{
        Authentication, ClientCredentialsAuthentication, DirectAccessCredentialsAuthentication,
        NoAuthentication,
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
            AuthConfig::None => Box::new(NoAuthentication),
            AuthConfig::Other(authentication) => authentication,
        };

        Ok(Self {
            http_client,
            authentication,
        })
    }

    pub(crate) async fn start_authorized(
        &self,
        config: &ControllerConfig,
        room_id: RoomId,
        breakout_room: Option<BreakoutRoomId>,
    ) -> Result<TicketToken> {
        let uri = format!("{}/rooms/{room_id}/start", config.v1_api_base_url());

        let token = self
            .authentication
            .access_token()
            .await
            .context("Failed to get access token from authentication backend")?;

        let response = self
            .http_client
            .post(&uri)
            .bearer_auth(token.secret())
            .json(&PostRoomsStartRequestBody {
                breakout_room,
                resumption: None,
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
                let response = serde_json::from_str::<RoomsStartResponseBody>(&response_body)
                    .context("Failed to parse response body")?;

                Ok(response.ticket)
            }
            code => bail!("unexpected status code {code:?}"),
        }
    }

    pub(crate) async fn start_invited(
        &self,
        config: &ControllerConfig,
        room_id: RoomId,
        breakout_room: Option<BreakoutRoomId>,
        invite_code: String,
        password: Option<RoomPassword>,
    ) -> Result<TicketToken> {
        let uri = format!("{}/rooms/{room_id}/start_invited", config.v1_api_base_url());

        let response = self
            .http_client
            .post(&uri)
            .json(&PostRoomsStartInvitedRequestBody {
                password,
                invite_code,
                breakout_room,
                resumption: None,
            })
            .send()
            .await
            .context("Failed to send start-invited request")?;

        let status = response.status();
        let response_body = response
            .text()
            .await
            .context("Failed to read HTTP response body")?;

        log::trace!("start-invited response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<RoomsStartResponseBody>(&response_body)
                    .context("Failed to parse response body")?;

                Ok(response.ticket)
            }
            code => bail!("unexpected status code {code:?}"),
        }
    }

    pub(crate) async fn start_call_in(
        &self,
        config: &ControllerConfig,
        id: CallInId,
        pin: CallInPassword,
    ) -> Result<TicketToken> {
        let uri = format!("{}/services/call_in/start", config.v1_api_base_url());

        let token = self
            .authentication
            .access_token()
            .await
            .context("Failed to get access token from authentication backend")?;

        let response = self
            .http_client
            .post(&uri)
            .bearer_auth(token.secret())
            .json(&PostCallInStartRequestBody { id, pin })
            .send()
            .await
            .context("Failed to send start-call-in request")?;

        let status = response.status();
        let response_body = response
            .text()
            .await
            .context("Failed to read HTTP response body")?;

        log::trace!("start-call-in response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<RoomsStartResponseBody>(&response_body)
                    .context("Failed to parse response body")?;

                Ok(response.ticket)
            }
            code => bail!("unexpected status code {code:?}"),
        }
    }
}
