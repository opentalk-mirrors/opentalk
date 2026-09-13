// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}/assets`

use actix_web::{
    get, post,
    web::{Data, Json, Path, Payload, Query},
};
use futures::TryStreamExt as _;
use opentalk_controller_service_facade::{
    NewAssetFileName, ObjectStorageError, OpenTalkControllerService, StorageNotifier,
};
use opentalk_types_api_v1::{
    error::ApiError,
    pagination::PagePaginationQuery,
    rooms::by_room_id::assets::{
        PostAssetQuery, PostAssetResponseBody, RoomsByRoomIdAssetsGetResponseBody,
    },
};
use opentalk_types_common::{rooms::RoomIdOrAlias, time::Timestamp};

use crate::{
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::response::ApiResponse,
};

pub mod by_id;

/// Get the assets associated with a room.
///
/// This returns assets that are available for a room. If no
/// pagination query is added, the default page size is used.
#[utoipa::path(
    operation_id = "room_assets",
    tag = "api::v1::assets",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
        PagePaginationQuery,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The assets have been returned successfully",
            body = RoomsByRoomIdAssetsGetResponseBody,
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
#[get("/rooms/{room_id_or_alias}/assets")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    room_id_or_alias: Path<RoomIdOrAlias>,
    pagination: Query<PagePaginationQuery>,
) -> Result<ApiResponse<RoomsByRoomIdAssetsGetResponseBody>, ApiError> {
    let pagination = pagination.into_inner();

    let (assets, asset_count) = service
        .get_room_assets(room_id_or_alias.into_inner(), &pagination)
        .await?;

    Ok(ApiResponse::new(assets).with_page_pagination(
        pagination.per_page,
        pagination.page,
        asset_count,
    ))
}

/// Create an asset for a room from an uploaded file
///
/// The asset is attached to the room and saved in the storage.
#[utoipa::path(
    operation_id = "create_room_asset",
    tag = "api::v1::assets",
    request_body(
        content = String,
        content_type = "application/octet-stream",
        description = "The contents of the file",
    ),
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
        PostAssetQuery,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The asset has been created successfully",
            body = PostAssetResponseBody,
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "Storage quota has been exceeded",
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "The associated room was not found",
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
#[post("/rooms/{room_id_or_alias}/assets")]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    notifier: Data<dyn StorageNotifier>,
    path: Path<RoomIdOrAlias>,
    query: Query<PostAssetQuery>,
    data: Payload,
) -> Result<Json<PostAssetResponseBody>, ApiError> {
    let room_id_or_alias = path.into_inner();
    let query = query.into_inner();

    let filename = NewAssetFileName::new_with_event_title(
        query.event_title,
        query.kind,
        Timestamp::now(),
        query.file_extension,
    );

    let data = data.map_err(|e| ObjectStorageError::Other {
        message: "Upload error".to_string(),
        source: Some(e.into()),
    });

    let (resource, _) = service
        .create_room_asset(
            notifier.as_ref(),
            room_id_or_alias,
            filename,
            query.namespace,
            Box::new(data),
        )
        .await?;

    let response = PostAssetResponseBody(resource);

    Ok(Json(response))
}
