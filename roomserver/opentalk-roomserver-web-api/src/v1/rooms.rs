// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Debug;

use async_trait::async_trait;
use axum::{
    Json,
    extract::{Path, State},
    routing::{delete, patch, post, put},
};
pub use opentalk_roomserver_types::room_action::RoomAction;
use opentalk_roomserver_types::{
    api::{RoomServerAccess, TokenRequestBody},
    client_parameters::ClientParameters,
    room_parameters::RoomParameters,
    room_parameters_patch::RoomParametersPatch,
};
use opentalk_types_api_internal::error::ApiError;
use opentalk_types_common::rooms::RoomId;

use crate::Router;

#[async_trait]
pub trait RoomBackend: Clone + Send + Sync + Debug {
    /// Create or update the room.
    async fn put_room(
        &self,
        room_id: RoomId,
        room_parameters: RoomParameters,
    ) -> Result<RoomAction, ApiError>;

    async fn patch_room(
        &self,
        room_id: RoomId,
        patch: RoomParametersPatch,
    ) -> Result<RoomAction, ApiError>;

    /// Delete the room and close the room task
    async fn delete_room(&self, room_id: RoomId);

    async fn request_room_token(
        &mut self,
        room_id: RoomId,
        client_parameters: ClientParameters,
        room_parameters: Option<RoomParameters>,
    ) -> Result<RoomServerAccess, ApiError>;
}

/// Creates a new room instance with the specified parameters if no room with the provided id
/// exists.
///
/// If a room with the provided room ID already exists, the rooms idle timeout is refreshed.
#[utoipa::path(
    put,
    path = "/rooms/{room_id}",
    request_body = RoomParameters,
    params(
        ("room_id" = RoomId, Path, description = "The UUID that identifies the room")
    ),
    responses(
        (status = StatusCode::CREATED, description = "Successfully created a new room"),
        (status = StatusCode::NO_CONTENT, description = "The room did exist before and the parameter were updated if necessary"),
        (status = StatusCode::UNAUTHORIZED, description = "The provided API token is invalid"),
        (status = StatusCode::BAD_REQUEST, description = "The provided API token or json body could not be parsed"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "The request body did not match the expected format"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    security(
        ("API-Token" = [])
    )
    )]
#[tracing::instrument(level = "info", name = "/rooms/{room_id}", skip_all, fields(opentalk.room_id = %path.0))]
pub(crate) async fn put_room<B: RoomBackend>(
    State(ctx): State<B>,
    path: Path<RoomId>,
    Json(room_parameters): Json<RoomParameters>,
) -> Result<RoomAction, ApiError> {
    ctx.put_room(path.0, room_parameters).await
}

/// Modifies the room parameters of an active room.
#[utoipa::path(
    patch,
    path = "/rooms/{room_id}",
    description = "Change the parameters of an existing room",
    request_body = RoomParametersPatch,
    params(
        ("room_id" = RoomId, Path, description = "The UUID that identifies the room")
    ),
    responses(
        (status = StatusCode::NO_CONTENT, description = "Successfully updated the room parameters"),
        (status = StatusCode::UNAUTHORIZED, description = "The provided API token is invalid"),
        (status = StatusCode::BAD_REQUEST, description = "The provided API token or json body could not be parsed"),
        (status = StatusCode::NOT_FOUND, description = "The provided room does not exist"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "The patch could not be applied"),
    ),
    security(
        ("API-Token" = [])
    )
    )]
pub(crate) async fn patch_room<B: RoomBackend>(
    State(ctx): State<B>,
    Path(room_id): Path<RoomId>,
    Json(patch): Json<RoomParametersPatch>,
) -> Result<RoomAction, ApiError> {
    ctx.patch_room(room_id, patch).await
}

#[utoipa::path(
   delete,
   path = "/rooms/{room_id}",
   summary = "Close the room",
   params(
      ("room_id" = RoomId, Path, description = "The UUID that identifies the room"),
   ),
   responses(
      (status = StatusCode::OK, description = "The room was closed successfully or did not exist"),
      (status = StatusCode::UNAUTHORIZED, description = "The provided API token is invalid"),
      (status = StatusCode::BAD_REQUEST, description = "The provided API token could not be parsed"),
   ),
   security(
       ("API-Token" = [])
   )
)]
pub(crate) async fn delete_room<B: RoomBackend>(State(ctx): State<B>, Path(room_id): Path<RoomId>) {
    ctx.delete_room(room_id).await;
}

/// Creates a new signaling token for the specified user and room
///
/// The signaling token can be used to establish a websocket connection with the roomserver through
/// the `/signaling/<token>` endpoint. The token has a limited lifetime (30 seconds by default) and
/// can only be used once.
///
/// Calling this endpoint will start a new room task or refresh existing ones. To get a token for an
/// unknown room, the request body has to contain the `room_parameters` field (See
/// [`TokenRequestBody`]). If the room is already running, any provided `room_parameters` will be
/// ignored.
#[utoipa::path(
    post,
    path = "/rooms/{room_id}/token",
    request_body = TokenRequestBody,
    params(
        ("room_id" = RoomId, Path, description = "The UUID that identifies the room")
    ),
    responses(
        (status = StatusCode::OK, description = "The response body contains the signaling token and the public URL of the roomserver", body = RoomServerAccess),
        (status = StatusCode::UNAUTHORIZED, description = "The provided API token is invalid"),
        (status = StatusCode::BAD_REQUEST, description = "Failed to parse request body"),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "The request body did not match the expected format or the room does not exist and no room parameters were provided"),
        (status = StatusCode::FORBIDDEN, description = "The requesting participant is banned from the room"),
        (status = StatusCode::BAD_REQUEST, description = "The provided API token or json body could not be parsed"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "An internal server error occurred"),
    ),
    security(
        ("API-Token" = [])
    )
    )]
#[tracing::instrument(level = "info", name = "/rooms/{room_id}/token", skip_all, fields(opentalk.room_id = %path.0, http.method = "PUT"))]
pub(crate) async fn request_token<B: RoomBackend>(
    State(mut ctx): State<B>,
    path: Path<RoomId>,
    Json(body): Json<TokenRequestBody>,
) -> Result<Json<RoomServerAccess>, ApiError> {
    let response = ctx
        .request_room_token(path.0, body.client_parameters, body.room_parameters)
        .await?;

    Ok(Json(response))
}

pub fn routes<B: RoomBackend + 'static>() -> Router<B> {
    Router::new().nest(
        "/rooms",
        Router::new()
            .route("/{room_id}", put(put_room::<B>))
            .route("/{room_id}", patch(patch_room::<B>))
            .route("/{room_id}", delete(delete_room::<B>))
            .route("/{room_id}/token", post(request_token::<B>)),
    )
}
