// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use actix_web::{
    post,
    web::{Data, Path, Payload, Query},
};
use futures::TryStreamExt as _;
use opentalk_asset_storage::{NewAssetFileName, ObjectStorageError, StorageNotifier};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_internal::module_assets::PostAssetResponseBody;
use opentalk_types_api_v1::{error::ApiError, rooms::by_room_id::assets::PostAssetQuery};
use opentalk_types_common::{rooms::RoomId, time::Timestamp};

use crate::api::{
    responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
    v1::{ApiResponse, DefaultApiResult},
};

/// Upload an asset. This endpoint requires the client to provide the RoomServer
/// credentials.
#[utoipa::path(
    context_path = "/internal",
    operation_id = "internal_upload_asset",
    request_body(
        content = String,
        content_type = "application/octet-stream",
        description = "The contents of the file",
    ),
    params(
        ("room_id" = RoomId, description = "The id of the room"),
        PostAssetQuery,
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The asset has been created successfully",
            body = PostAssetResponseBody,
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
#[post("/room/{room_id}/asset")]
pub async fn post_asset(
    service: Data<dyn OpenTalkControllerService>,
    notifier: Data<dyn StorageNotifier>,
    path: Path<RoomId>,
    query: Query<PostAssetQuery>,
    data: Payload,
) -> DefaultApiResult<PostAssetResponseBody, ApiError> {
    let room_id = path.into_inner();
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

    let (asset_resource, asset_saved) = service
        .create_room_asset(
            notifier.as_ref(),
            room_id.into(),
            filename,
            query.namespace,
            Box::new(data),
        )
        .await?;

    Ok(ApiResponse::new(PostAssetResponseBody {
        asset_resource,
        quota: asset_saved.quota,
    }))
}
