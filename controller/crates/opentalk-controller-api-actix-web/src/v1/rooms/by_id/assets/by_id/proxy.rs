// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}/assets/{asset_id}/proxy`

use actix_web::{
    HttpRequest, HttpResponse, get,
    web::{Data, Path, Query},
};
use futures::TryStreamExt as _;
use opentalk_controller_service_facade::{AssetDownloadProxyStream, OpenTalkControllerService};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::{assets::AssetId, rooms::RoomIdOrAlias};
use serde::{Deserialize, Serialize};

use crate::utoipa::responses::{
    BinaryData, Forbidden, InternalServerError, NotFound, Unauthorized,
};

/// Query parameters for *GET /rooms/{room_id_or_alias}/assets/{asset_id}/proxy*
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AssetProxyDownloadQuery {
    /// The token required for the asset download
    token: String,
}

/// Stream an asset via the controller proxy.
///
/// This endpoint forwards the request to object storage using the object storage
/// query parameters supplied by the client.
#[utoipa::path(
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
        ("asset_id" = AssetId, description = "The id of the asset"),
    ),
    responses(
        (
            status = StatusCode::OK,
            response = BinaryData,
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
    security(("BearerAuth" = [])),
)]
#[get("/rooms/{room_id_or_alias}/assets/{asset_id}/proxy")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    path: Path<(RoomIdOrAlias, AssetId)>,
    req: HttpRequest,
    query: Query<AssetProxyDownloadQuery>,
) -> Result<HttpResponse, ApiError> {
    let (_room_id_or_alias, asset_id) = path.into_inner();
    let query = query.into_inner();

    let range_header = req
        .headers()
        .get(actix_web::http::header::RANGE)
        .map(|v| v.to_str())
        .transpose()
        .unwrap()
        .map(ToString::to_string);

    let AssetDownloadProxyStream {
        status,
        headers,
        stream,
    } = service
        .get_asset_proxy_download_stream(asset_id, query.token, range_header)
        .await?;

    let mut response = HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status).unwrap_or(actix_web::http::StatusCode::OK),
    );

    for (k, v) in headers {
        _ = response.append_header((k, v));
    }

    Ok(response.streaming(stream.map_err(actix_web::error::ErrorInternalServerError)))
}
