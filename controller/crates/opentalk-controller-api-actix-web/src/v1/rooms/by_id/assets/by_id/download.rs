// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}/assets/{asset_id}/download`

use actix_web::{
    HttpResponse, get,
    http::StatusCode,
    web::{Data, Path, Query},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::by_room_id::assets::{AssetDownloadQuery, AssetDownloadResponseBody},
};
use opentalk_types_common::{assets::AssetId, rooms::RoomIdOrAlias};

use crate::utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized};

/// Get the controller download path for an asset.
///
/// Returns a path for downloading the asset via the controller.
#[utoipa::path(
    operation_id = "room_asset_download",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
        ("asset_id" = AssetId, description = "The id of the asset"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Returns JSON with the controller download path",
            body = AssetDownloadResponseBody,
        ),
        (
            status = StatusCode::FOUND,
            description = "Redirects to the controller download path",
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
#[get("/rooms/{room_id_or_alias}/assets/{asset_id}/download")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    path: Path<(RoomIdOrAlias, AssetId)>,
    query: Query<AssetDownloadQuery>,
) -> Result<HttpResponse, ApiError> {
    let (room_id_or_alias, asset_id) = path.into_inner();
    let query = query.into_inner();

    let token = service
        .get_room_asset_proxy_download_token(room_id_or_alias, asset_id)
        .await?;
    let url = format!("proxy?token={token}");

    let mut response = if query.redirect.unwrap_or(true) {
        let mut res = HttpResponse::build(StatusCode::FOUND);

        _ = res.insert_header((actix_web::http::header::LOCATION, url.clone()));

        res
    } else {
        HttpResponse::build(StatusCode::OK)
    };

    Ok(response.json(AssetDownloadResponseBody { url }))
}
