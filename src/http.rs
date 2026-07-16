// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::option::Option;

use anyhow::Context;
use openidconnect::url;
use opentalk_roomserver_types::breakout::breakout_id::BreakoutId;
use opentalk_types::{
    api::v1::{
        pagination::PagePaginationQuery,
        rooms::{
            PostRoomsRequestBody, RoomResource,
            by_room_id::{PostRoomsRoomserverStartRequestBody, RoomserverStartResponseBody},
        },
    },
    common::{
        call_in::{CallInId, CallInPassword},
        pagination::{Page, PageSize},
        rooms::{GuestAccess, RoomId, RoomPassword},
        roomserver::{DEVICE_SECRET_MIN_LENGTH, DeviceSecret},
        users::DisplayName,
    },
};
use opentalk_types_api_internal::{
    call_in::PostCallInStartRoomServerRequestBody, recording::RecordingTarget,
};
use rand::distr::SampleString;
use reqwest::{StatusCode, Url};

use crate::{
    authentication::{
        ApiKeyAuthentication, Authentication, ClientCredentialsAuthentication,
        DirectAccessCredentialsAuthentication, NoAuthentication,
    },
    config::{AuthConfig, BaseUrl},
};

#[derive(Debug, thiserror::Error)]
pub enum StartAsUserError {
    #[error("Failed to construct destination url")]
    UrlParseError(#[from] url::ParseError),
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
    #[error("Failed to construct destination url")]
    UrlParseError(#[from] url::ParseError),
    #[error("Failed to make the HTTP request")]
    HttpFailure(#[from] reqwest::Error),
    #[error("Failed to parse HTTP response body")]
    InvalidResponseBody(#[from] serde_json::Error),
    #[error("Received an unexpected status: {0}")]
    UnexpectedStatusCode(StatusCode),
}

#[derive(Debug, thiserror::Error)]
pub enum StartCallInError {
    #[error("Failed to construct destination url")]
    UrlParseError(#[from] url::ParseError),
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
    #[error("Failed to construct destination url")]
    UrlParseError(#[from] url::ParseError),
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
    ConnectionFailed(#[source] tokio_tungstenite::tungstenite::Error),
    #[error("Uri can't be turned into request")]
    UriConversionFailed,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateRoomError {
    #[error("Failed to construct destination url")]
    UrlParseError(#[from] url::ParseError),
    #[error("Failed to get access token from authentication backend")]
    FailedToGetAccessToken(#[source] anyhow::Error),
    #[error("Request failed because of: {0}")]
    ResponseError(#[from] reqwest::Error),
    #[error("Failed to create room: server responded with: {0}")]
    FailedToCreateRoom(String),
}

#[derive(Debug, thiserror::Error)]
pub enum GetRoomError {
    #[error("Failed to construct destination url")]
    UrlParseError(#[from] url::ParseError),
    #[error("Failed to get access token from authentication backend")]
    FailedToGetAccessToken(#[source] anyhow::Error),
    #[error("Request failed because of {0}")]
    ResponseError(#[from] reqwest::Error),
    #[error("Failed to get all rooms: {0}")]
    FailedToGetRooms(String),
}

pub(crate) struct HttpClient {
    device_secret: DeviceSecret,
    http_client: reqwest::Client,
    pub(crate) authentication: Box<dyn Authentication>,
}

impl HttpClient {
    pub(crate) async fn create(config: AuthConfig) -> Result<Self, anyhow::Error> {
        let http_client = reqwest::Client::new();

        let authentication: Box<dyn Authentication> = match config {
            AuthConfig::ApiKey(api_key) => Box::new(ApiKeyAuthentication::from(api_key)),
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
            device_secret: rand::distr::Alphanumeric
                .sample_string(&mut rand::rng(), DEVICE_SECRET_MIN_LENGTH)
                .parse()
                .context("Failed to create device secret")?,
            http_client,
            authentication,
        })
    }

    pub(crate) async fn start_as_user(
        &self,
        controller_url: &BaseUrl,
        room_id: RoomId,
        display_name: Option<DisplayName>,
    ) -> Result<RoomserverStartResponseBody, StartAsUserError> {
        let url = controller_url.join(format!("v1/rooms/{room_id}/start"))?;

        let token = self
            .authentication
            .access_token()
            .await
            .map_err(StartAsUserError::FailedToGetAccessToken)?;

        let response = self
            .http_client
            .post(url)
            .bearer_auth(token.secret())
            .json(&PostRoomsRoomserverStartRequestBody {
                device_secret: self.device_secret.clone(),
                display_name,
                password: None,
            })
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        log::trace!("start-authorized response status={status} body={response_body}");

        match status {
            StatusCode::OK => Ok(serde_json::from_str::<RoomserverStartResponseBody>(
                &response_body,
            )?),
            status => Err(StartAsUserError::UnexpectedStatusCode(status)),
        }
    }

    pub(crate) async fn start_as_recorder(
        &self,
        controller_url: &BaseUrl,
        room_id: RoomId,
        breakout_id: Option<BreakoutId>,
    ) -> Result<RoomserverStartResponseBody, StartRecorderError> {
        let url = controller_url.join("/internal/recording/start")?;
        self.start_as_service(url, room_id, breakout_id).await
    }

    pub(crate) async fn start_as_transcription(
        &self,
        controller_url: &BaseUrl,
        room_id: RoomId,
        breakout_id: Option<BreakoutId>,
    ) -> Result<RoomserverStartResponseBody, StartRecorderError> {
        let url = controller_url.join("/internal/transcription/start")?;
        self.start_as_service(url, room_id, breakout_id).await
    }

    async fn start_as_service(
        &self,
        controller_url: Url,
        room_id: RoomId,
        breakout_id: Option<BreakoutId>,
    ) -> Result<RoomserverStartResponseBody, StartRecorderError> {
        let token = self
            .authentication
            .access_token()
            .await
            .map_err(StartRecorderError::FailedToGetAccessToken)?;

        let response = self
            .http_client
            .post(controller_url)
            .bearer_auth(token.secret())
            .json(&RecordingTarget {
                room_id,
                breakout_room: breakout_id.map(BreakoutId::into),
            })
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        log::trace!("start-authorized response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<RoomserverStartResponseBody>(&response_body)?;

                Ok(response)
            }
            status => Err(StartRecorderError::UnexpectedStatusCode(status)),
        }
    }

    pub(crate) async fn start_call_in(
        &self,
        controller_url: &BaseUrl,
        id: CallInId,
        pin: CallInPassword,
        display_name: DisplayName,
    ) -> Result<RoomserverStartResponseBody, StartCallInError> {
        let url = controller_url.join("/internal/call_in/start")?;

        let token = self
            .authentication
            .access_token()
            .await
            .map_err(StartCallInError::FailedToGetAccessToken)?;

        let response = self
            .http_client
            .post(url)
            .bearer_auth(token.secret())
            .json(&PostCallInStartRoomServerRequestBody {
                id,
                pin,
                display_name,
            })
            .send()
            .await?;

        let status = response.status();
        let response_body = response.text().await?;

        log::trace!("start-call-in response status={status} body={response_body}");

        match status {
            StatusCode::OK => {
                let response = serde_json::from_str::<RoomserverStartResponseBody>(&response_body)?;

                Ok(response)
            }
            StatusCode::BAD_REQUEST => Err(StartCallInError::InvalidIdOrPin),
            status => Err(StartCallInError::UnexpectedStatusCode(status)),
        }
    }

    /// Get a list ids of rooms that are accessible
    pub(crate) async fn get_rooms(
        &self,
        controller_url: &BaseUrl,
        max: usize,
    ) -> Result<Vec<RoomResource>, GetRoomError> {
        let url = controller_url.join("v1/rooms")?;

        let mut ret = Vec::new();

        let token = self
            .authentication
            .access_token()
            .await
            .map_err(GetRoomError::FailedToGetAccessToken)?;

        // Check 100 pages at most
        for page in 1..=(max as i64) {
            let response = self
                .http_client
                .get(url.clone())
                .query(&PagePaginationQuery {
                    per_page: PageSize::MAX,
                    page: Page::from_i64_clamped(page),
                })
                .bearer_auth(token.secret())
                .send()
                .await?;

            if response.status().is_success() {
                let rooms = response.json::<Vec<RoomResource>>().await?;

                let rooms_len = rooms.len();

                ret.extend(rooms);

                // Getting fewer results than requested, must be last page
                if ret.len() >= max || PageSize::from_i64_clamped(rooms_len as i64) < PageSize::MAX
                {
                    break;
                }
            } else {
                return Err(GetRoomError::FailedToGetRooms(format!(
                    "{:?}",
                    response.json::<serde_json::Value>().await
                )));
            }
        }

        Ok(ret)
    }

    // Create a room and return it's id
    pub(crate) async fn create_room(
        &self,
        controller_url: &BaseUrl,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        guest_access: GuestAccess,
        e2e_encryption: bool,
    ) -> Result<RoomResource, CreateRoomError> {
        let url = controller_url.join("v1/rooms")?;
        let token = self
            .authentication
            .access_token()
            .await
            .map_err(CreateRoomError::FailedToGetAccessToken)?;

        let response = self
            .http_client
            .post(url)
            .bearer_auth(token.secret())
            .json(&PostRoomsRequestBody {
                password,
                enable_sip,
                waiting_room,
                guest_access: Some(guest_access),
                e2e_encryption,
            })
            .send()
            .await?;

        if response.status().is_success() {
            response
                .json::<RoomResource>()
                .await
                .map_err(CreateRoomError::ResponseError)
        } else {
            Err(CreateRoomError::FailedToCreateRoom(format!(
                "{:?}",
                response.json::<serde_json::Value>().await
            )))
        }
    }
}
