// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}/streaming_targets`

use actix_web::{
    get, post,
    web::{Data, Json, Path, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    events::StreamingTargetOptionsQuery,
    pagination::PagePaginationQuery,
    rooms::by_room_id::streaming_targets::{
        GetRoomStreamingTargetsResponseBody, PostRoomStreamingTargetRequestBody,
        PostRoomStreamingTargetResponseBody,
    },
};
use opentalk_types_common::{pagination::ItemCount, rooms::RoomIdOrAlias};

use crate::{
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::ApiResponse,
};

pub mod by_id;

/// Lists the streaming targets of a room
///
/// Returns the streaming targets available for a room
#[utoipa::path(
    operation_id = "get_streaming_targets",
    tag = "api::v1::streaming_targets",
    params(
        PagePaginationQuery,
        (
            "room_id_or_alias" = RoomIdOrAlias,
            description = "Either the id or the alias of the room"
        ),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "List of streaming targets successfully returned",
            body = GetRoomStreamingTargetsResponseBody,
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
#[get("/rooms/{room_id_or_alias}/streaming_targets")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id_or_alias: Path<RoomIdOrAlias>,
    pagination: Query<PagePaginationQuery>,
) -> Result<ApiResponse<GetRoomStreamingTargetsResponseBody>, ApiError> {
    let response = service
        .get_streaming_targets(current_user.id, room_id_or_alias.into_inner(), &pagination)
        .await?;
    let length = ItemCount::try_from(response.0.len())
        .expect("looks like we got more items than can be represented in the ItemCount type");

    Ok(ApiResponse::new(response).with_page_pagination(
        pagination.per_page,
        pagination.page,
        length,
    ))
}

/// Creates a new streaming target
///
/// Creates a new streaming target for the given room
#[utoipa::path(
    operation_id = "post_streaming_target",
    tag = "api::v1::streaming_targets",
    params(
        StreamingTargetOptionsQuery,
        (
            "room_id_or_alias" = RoomIdOrAlias,
            description = "Either the id or the alias of the room"
        ),
    ),
    request_body = PostRoomStreamingTargetRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "Successfully create a new streaming target",
            body = PostRoomStreamingTargetResponseBody,
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
#[post("/rooms/{room_id_or_alias}/streaming_targets")]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    room_id_or_alias: Path<RoomIdOrAlias>,
    query: Query<StreamingTargetOptionsQuery>,
    data: Json<PostRoomStreamingTargetRequestBody>,
) -> Result<Json<PostRoomStreamingTargetResponseBody>, ApiError> {
    let response = service
        .post_streaming_target(
            current_user.into_inner(),
            room_id_or_alias.into_inner(),
            query.into_inner(),
            data.into_inner().0,
        )
        .await?;

    Ok(Json(response))
}
