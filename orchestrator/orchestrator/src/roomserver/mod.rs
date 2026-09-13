// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use async_trait::async_trait;
use axum::{Router, http::StatusCode};
use bytes::Bytes;
use opentalk_roomserver_types::{
    api::{RoomServerAccess, TokenRequestBody},
    client_parameters::ClientParameters,
    room_parameters::RoomParameters,
    room_parameters_patch::RoomParametersPatch,
};
use opentalk_roomserver_web_api::v1::{RoomAction, RoomBackend, rooms};
use opentalk_types_api_common::error::{ApiError, ErrorBody};
use opentalk_types_common::rooms::RoomId;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

use crate::{
    AppState,
    service_instance::selection::{InstanceError, SelectedInstance},
};

pub mod signaling;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(signaling::routes())
        .merge(rooms::routes())
}

#[async_trait]
impl RoomBackend for AppState {
    async fn put_room(
        &self,
        room_id: RoomId,
        room_parameters: RoomParameters,
    ) -> Result<RoomAction, ApiError> {
        let SelectedInstance {
            address,
            auth_header,
        } = match self.select_roomserver(room_id).await {
            Ok(instance) => instance,
            Err(err) => {
                tracing::error!("Failed to select roomserver instance {err:?}");
                return Err(err.into());
            }
        };

        tracing::debug!("send put request to roomserver '{address}'");

        let response = self
            .client
            .put(format!("{address}v1/rooms/{room_id}"))
            .header(AUTHORIZATION, auth_header)
            .json(&room_parameters)
            .send()
            .await;

        let Ok(response) = response else {
            tracing::debug!("Request failed with: '{response:?}'");
            return Err(ApiError::internal().with_message("Roomserver unreachable"));
        };

        let status = response.status();
        let body = response.text().await.map_err(|err| {
            ApiError::internal().with_message(format!(
                "Failed to read response body: {:?}",
                err.to_string()
            ))
        })?;

        tracing::trace!("received status code: {status:?}, response body: {body}");

        let Some(room_action) = RoomAction::from_status_code(status) else {
            let body = serde_json::from_str::<ErrorBody>(&body)
                .map_err(|err| ApiError::internal().with_message(err.to_string()))?;

            return Err(ApiError {
                status,
                www_authenticate: None,
                body,
            });
        };

        Ok(room_action)
    }

    async fn patch_room(
        &self,
        room_id: RoomId,
        patch: RoomParametersPatch,
    ) -> Result<RoomAction, ApiError> {
        let SelectedInstance {
            address,
            auth_header,
        } = match self.select_roomserver(room_id).await {
            Ok(instance) => instance,
            Err(err) => {
                match err {
                    InstanceError::NotAvailable => {
                        // This is a special case where we would not want to return the 503 from the
                        // instance selection. Instead we return a 404 to act as if we were the
                        // roomserver and to avoid confusing the client.
                        return Err(ApiError::not_found()
                            .with_message("The requested room could not be found"));
                    }
                    err => {
                        tracing::error!("Failed to select roomserver instance {err:?}");
                        return Err(err.into());
                    }
                }
            }
        };

        tracing::debug!("send patch request to roomserver '{address}'");

        let response = self
            .client
            .patch(format!("{address}v1/rooms/{room_id}"))
            .header(AUTHORIZATION, auth_header)
            .json(&patch)
            .send()
            .await;

        let Ok(response) = response else {
            tracing::debug!("Request failed with: '{response:?}'");
            return Err(ApiError::internal().with_message("Roomserver unreachable"));
        };

        let status = response.status();
        let body = response.text().await.map_err(|err| {
            ApiError::internal().with_message(format!(
                "Failed to read response body: {:?}",
                err.to_string()
            ))
        })?;

        tracing::trace!("received status code: {status:?}, response body: {body}");

        let Some(room_action) = RoomAction::from_status_code(status) else {
            let body = serde_json::from_str::<ErrorBody>(&body)
                .map_err(|err| ApiError::internal().with_message(err.to_string()))?;

            return Err(ApiError {
                status,
                www_authenticate: None,
                body,
            });
        };

        Ok(room_action)
    }

    async fn delete_room(&self, room_id: RoomId) {
        let SelectedInstance {
            address,
            auth_header,
        } = match self.select_roomserver(room_id).await {
            Ok(instance) => instance,
            Err(err) => {
                tracing::error!(
                    "received delete request for unkown room {room_id}, instance selection failed with: {err:?}"
                );
                return;
            }
        };

        tracing::debug!("sending delete room {room_id} request to roomserver '{address}'");

        if let Err(err) = self
            .client
            .delete(format!("{address}v1/rooms/{room_id}"))
            .header(AUTHORIZATION, auth_header)
            .send()
            .await
        {
            tracing::error!("Error response from roomserver for room deletion request: {err:?}");
        }
    }

    async fn request_room_token(
        &mut self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> Result<RoomServerAccess, ApiError> {
        let SelectedInstance {
            address,
            auth_header,
        } = match self.select_roomserver(room_id).await {
            Ok(instance) => instance,
            Err(err) => {
                tracing::error!("Failed to select roomserver instance {err:?}");
                return Err(err.into());
            }
        };
        tracing::debug!("send token request to roomserver '{address}'");

        let token_request = TokenRequestBody {
            client_parameters,
            room_parameters,
        };

        let response = self
            .client
            .post(format!("{address}v1/rooms/{room_id}/token"))
            .header(AUTHORIZATION, auth_header)
            .json(&token_request)
            .send()
            .await;

        let Ok(response) = response else {
            tracing::debug!("Request failed with: '{response:?}'");
            return Err(ApiError::internal().with_message("Roomserver unreachable"));
        };

        let status = response.status();
        let body = response.bytes().await.map_err(|e| {
            ApiError::internal().with_message(format!("Roomserver connection interrupted: {e}"))
        })?;

        tracing::trace!(
            "received status code: {status:?}, response body: {:?}",
            std::str::from_utf8(&body)
        );

        let signaling_url = self.public_url.join("roomserver/").map_err(|e| {
            tracing::error!("Failed to build roomserver proxy endpoint: {e}");
            ApiError::internal()
        })?;

        match status {
            StatusCode::OK => {
                let mut roomserver_access: RoomServerAccess =
                    deserialize_token_response(status, &body)?;

                if let Err(e) = self
                    .storage
                    .add_roomserver_token(room_id, roomserver_access.token)
                    .await
                {
                    tracing::error!("Failed to store roomserver token: {e:?}");
                    return Err(
                        ApiError::internal().with_message("Failed to store roomserver token")
                    );
                };

                roomserver_access.public_url = signaling_url;

                Ok(roomserver_access)
            }
            StatusCode::UNAUTHORIZED => {
                Err(ApiError::internal().with_message("Failed to authorize at roomserver"))
            }
            error_code => Err(ApiError {
                status: error_code,
                www_authenticate: None,
                body: deserialize_token_response(error_code, &body)?,
            }),
        }
    }
}

fn deserialize_token_response<T: for<'a> Deserialize<'a>>(
    status: StatusCode,
    body: &Bytes,
) -> Result<T, ApiError> {
    match serde_json::from_slice::<T>(body) {
        Ok(t) => Ok(t),
        Err(e) => {
            tracing::error!(
                "Unexpected roomserver token response body for status {status}: {e}\nBody:\n{}",
                String::from_utf8_lossy(body)
            );
            Err(ApiError::internal().with_message("Unexpected roomserver response body"))
        }
    }
}
