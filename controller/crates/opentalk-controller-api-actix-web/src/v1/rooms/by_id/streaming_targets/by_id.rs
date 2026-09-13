// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}/streaming_targets/{streaming_target_id}`

use actix_web::{
    delete, get, patch,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::StreamingTargetOptionsQuery,
    rooms::by_room_id::streaming_targets::{
        GetRoomStreamingTargetResponseBody, PatchRoomStreamingTargetRequestBody,
        PatchRoomStreamingTargetResponseBody, RoomAndStreamingTargetId,
    },
};

use crate::{
    response::NoContent,
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
};

/// Gets a streaming target
///
/// Returns a single streaming target for a specific room.
#[utoipa::path(
    operation_id = "get_streaming_target",
    tag = "api::v1::streaming_targets",
    params(RoomAndStreamingTargetId),
    responses(
        (
            status = StatusCode::OK,
            description = "The streaming target has been successfully returned",
            body = GetRoomStreamingTargetResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/rooms/{room_id_or_alias}/streaming_targets/{streaming_target_id}")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<RoomAndStreamingTargetId>,
) -> Result<Json<GetRoomStreamingTargetResponseBody>, ApiError> {
    let response = service
        .get_streaming_target(current_user.id, path_params.into_inner())
        .await?;

    Ok(Json(response))
}

/// Updates a streaming target
///
/// Modifies and returns a single streaming target.
#[utoipa::path(
    operation_id = "patch_streaming_target",
    tag = "api::v1::streaming_targets",
    params(RoomAndStreamingTargetId),
    request_body = PatchRoomStreamingTargetRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Streaming target was successfully updated",
            body = PatchRoomStreamingTargetResponseBody
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the specified streaming target due to wrong
                syntax or bad values",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = HttpStatus::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[patch("/rooms/{room_id_or_alias}/streaming_targets/{streaming_target_id}")]
pub async fn patch(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<RoomAndStreamingTargetId>,
    query: Query<StreamingTargetOptionsQuery>,
    streaming_target: Json<PatchRoomStreamingTargetRequestBody>,
) -> Result<Json<PatchRoomStreamingTargetResponseBody>, ApiError> {
    let response = service
        .patch_streaming_target(
            current_user.into_inner(),
            path_params.into_inner(),
            query.into_inner(),
            streaming_target.into_inner(),
        )
        .await?;

    Ok(Json(response))
}

/// Deletes a streaming target
///
/// The streaming target is deleted from the room
#[utoipa::path(
    operation_id = "delete_streaming_target",
    tag = "api::v1::streaming_targets",
    params(
        RoomAndStreamingTargetId,
        StreamingTargetOptionsQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The streaming target has been deleted",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
        ),
        (
            status = StatusCode::FORBIDDEN,
            response = Forbidden,
        ),
        (
            status = StatusCode::NOT_FOUND,
            response = NotFound,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[delete("/rooms/{room_id_or_alias}/streaming_targets/{streaming_target_id}")]
pub async fn delete(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    path_params: Path<RoomAndStreamingTargetId>,
    query: Query<StreamingTargetOptionsQuery>,
) -> Result<NoContent, ApiError> {
    service
        .delete_streaming_target(
            current_user.into_inner(),
            path_params.into_inner(),
            query.into_inner(),
        )
        .await?;

    Ok(NoContent)
}
