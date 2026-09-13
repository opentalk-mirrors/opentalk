// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}`

use actix_web::{
    delete, get, patch,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::{
        RoomResource,
        by_room_id::{DeleteRoomQuery, PatchRoomsRequestBody},
    },
};
use opentalk_types_common::rooms::RoomIdOrAlias;

use crate::{
    response::NoContent,
    utoipa::responses::{Forbidden, InternalServerError, Unauthorized},
};

pub mod assets;
pub mod event;
pub mod invites;
pub mod roomserver;
pub mod sip;
pub mod start;
pub mod start_invited;
pub mod streaming_targets;
pub mod tariff;

/// Get a room
///
/// Returns the room resource including additional information such as the creator profile.
#[utoipa::path(
    operation_id = "get_room",
    tag = "api::v1::rooms",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Room was successfully retrieved",
            body = RoomResource
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[get("/rooms/{room_id_or_alias}")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    room_id_or_alias: Path<RoomIdOrAlias>,
) -> Result<Json<RoomResource>, ApiError> {
    Ok(Json(service.get_room(room_id_or_alias.into_inner()).await?))
}

/// Patch a room with the provided fields
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    request_body = PatchRoomsRequestBody,
    operation_id = "patch_room",
    tag = "api::v1::rooms",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Room was successfully updated",
            body = RoomResource
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the specified room due to wrong
                syntax or bad values, for example an invalid owner id",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            response = Unauthorized,
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
#[patch("/rooms/{room_id_or_alias}")]
pub async fn patch(
    service: Data<dyn OpenTalkControllerService>,
    room_id_or_alias: Path<RoomIdOrAlias>,
    body: Json<PatchRoomsRequestBody>,
) -> Result<Json<RoomResource>, ApiError> {
    let room_id_or_alias = room_id_or_alias.into_inner();
    let body = body.into_inner();

    let room_resource = service
        .patch_room(
            room_id_or_alias,
            body.name,
            body.password,
            body.waiting_room,
            body.guest_access,
            body.e2e_encryption,
        )
        .await?;

    Ok(Json(room_resource))
}

/// Delete a room and its owned resources.
///
/// Deletes the room by the id if found. See the query parameters for affecting
/// the behavior of this endpoint, such as succeding even if external resources
/// cannot be successfully deleted.
#[utoipa::path(
    tag = "api::v1::rooms",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
        DeleteRoomQuery,
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "Room was successfully deleted",
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
            status = StatusCode::INTERNAL_SERVER_ERROR,
            response = InternalServerError,
        ),
    ),
    security(
        ("BearerAuth" = []),
    ),
)]
#[delete("/rooms/{room_id_or_alias}")]
pub async fn delete(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id_or_alias: Path<RoomIdOrAlias>,
    query: Query<DeleteRoomQuery>,
) -> Result<NoContent, ApiError> {
    let query = query.into_inner();

    service
        .delete_room(
            current_user.into_inner(),
            room_id_or_alias.into_inner(),
            query.force_delete_reference_if_external_services_fail,
        )
        .await?;

    Ok(NoContent)
}
