// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::option::Option;

use reqwest::StatusCode;
use types::{
    api::v1::{
        rooms::by_room_id::{
            PostRoomsStartInvitedRequestBody, PostRoomsStartRequestBody, RoomsStartResponseBody,
        },
        services::{call_in::PostCallInStartRequestBody, recording::PostRecordingStartRequestBody},
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

#[derive(Debug, thiserror::Error)]
pub enum StartAsUserError {
    #[error("Failed to get access token from authentication backend")]
    FailedToGetAccessToken(#[source] anyhow::Error),
    #[error("Failed to make the HTTP request")]
    HttpFailure(#[from] reqwest::Error),
    #[error("Failed to parse HTTP response body")]
    InvalidResponseBody(#[from] serde_json::Error),
    #[error("Received an unexpected status: {0}")]
    UnexpectedStatusCode(StatusCode),
}

#[derive(Debug, thiserror::Error)]
pub enum StartInvitedError {
    #[error("Failed to make the HTTP request")]
    HttpFailure(#[from] reqwest::Error),
    #[error("Failed to parse HTTP response body")]
    InvalidResponseBody(#[from] serde_json::Error),
    #[error("Received an unexpected status: {0}")]
    UnexpectedStatusCode(StatusCode),
}

#[derive(Debug, thiserror::Error)]
pub enum StartCallInError {
    #[error("Failed to get access token from authentication backend")]
    FailedToGetAccessToken(#[source] anyhow::Error),
    #[error("Failed to make the HTTP request")]
    HttpFailure(#[from] reqwest::Error),
    #[error("Failed to parse HTTP response body")]
    InvalidResponseBody(#[from] serde_json::Error),
    #[error("Given id or pin is invalid")]
    InvalidIdOrPin,
    #[error("Received an unexpected status: {0}")]
    UnexpectedStatusCode(StatusCode),
}

#[derive(Debug, thiserror::Error)]
pub enum StartRecorderError {
    #[error("Failed to get access token from authentication backend")]
    FailedToGetAccessToken(#[source] anyhow::Error),
    #[error("Failed to make the HTTP request")]
    HttpFailure(#[from] reqwest::Error),
    #[error("Failed to parse HTTP response body")]
    InvalidResponseBody(#[from] serde_json::Error),
    #[error("Received an unexpected status: {0}")]
    UnexpectedStatusCode(StatusCode),
}

#[derive(Debug, thiserror::Error)]
pub enum WebsocketError {
    #[error("Failed to get access token from authentication backend")]
    FailedToGetAccessToken(#[source] anyhow::Error),
    #[error("Invalid header value")]
    InvalidHeader,
    #[error("Connecting to websocket failed: {0}")]
    ConnectionFailed(#[source] tt::tungstenite::Error),
    #[error("Uri can't be turned into request")]
    UriConversionFailed,
}

pub(crate) struct HttpClient {
    http_client: reqwest::Client,
    pub(crate) authentication: Box<dyn Authentication>,
}

impl HttpClient {
    pub(crate) async fn create(config: AuthConfig) -> Result<Self, anyhow::Error> {
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

    pub(crate) async fn start_as_user(
        &self,
        config: &ControllerConfig,
        room_id: RoomId,
        breakout_room: Option<BreakoutRoomId>,
    ) -> Result<TicketToken, StartAsUserError> {
        let uri = format!("{}/rooms/{room_id}/start", config.v1_api_base_url());

        let token = self
            .authentication
            .access_token()
            .await
            .map_err(StartAsUserError::FailedToGetAccessToken)?;

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
        let response_body = response.text().await?;

        log::trace!("start-authorized response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<RoomsStartResponseBody>(&response_body)?;

                Ok(response.ticket)
            }
            status => Err(StartAsUserError::UnexpectedStatusCode(status)),
        }
    }

    pub(crate) async fn start_as_recorder(
        &self,
        config: &ControllerConfig,
        room_id: RoomId,
        breakout_room: Option<BreakoutRoomId>,
    ) -> Result<TicketToken, StartRecorderError> {
        let uri = format!("{}/services/recording/start", config.v1_api_base_url());

        let token = self
            .authentication
            .access_token()
            .await
            .map_err(StartRecorderError::FailedToGetAccessToken)?;

        let response = self
            .http_client
            .post(&uri)
            .bearer_auth(token.secret())
            .json(&PostRecordingStartRequestBody {
                room_id,
                breakout_room,
            })
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        log::trace!("start-authorized response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<RoomsStartResponseBody>(&response_body)?;

                Ok(response.ticket)
            }
            status => Err(StartRecorderError::UnexpectedStatusCode(status)),
        }
    }

    pub(crate) async fn start_invited(
        &self,
        config: &ControllerConfig,
        room_id: RoomId,
        breakout_room: Option<BreakoutRoomId>,
        invite_code: String,
        password: Option<RoomPassword>,
    ) -> Result<TicketToken, StartInvitedError> {
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
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        log::trace!("start-invited response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<RoomsStartResponseBody>(&response_body)?;

                Ok(response.ticket)
            }
            status => Err(StartInvitedError::UnexpectedStatusCode(status)),
        }
    }

    pub(crate) async fn start_call_in(
        &self,
        config: &ControllerConfig,
        id: CallInId,
        pin: CallInPassword,
    ) -> Result<TicketToken, StartCallInError> {
        let uri = format!("{}/services/call_in/start", config.v1_api_base_url());

        let token = self
            .authentication
            .access_token()
            .await
            .map_err(StartCallInError::FailedToGetAccessToken)?;

        let response = self
            .http_client
            .post(&uri)
            .bearer_auth(token.secret())
            .json(&PostCallInStartRequestBody { id, pin })
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        log::trace!("start-call-in response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<RoomsStartResponseBody>(&response_body)?;

                Ok(response.ticket)
            }
            StatusCode::BAD_REQUEST => Err(StartCallInError::InvalidIdOrPin),
            status => Err(StartCallInError::UnexpectedStatusCode(status)),
        }
    }
}
