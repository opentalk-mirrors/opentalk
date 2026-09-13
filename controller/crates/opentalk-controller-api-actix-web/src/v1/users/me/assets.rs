// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/users/me/assets`

use actix_web::{
    get,
    web::{Data, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    assets::AssetSortingQuery, error::ApiError, pagination::PagePaginationQuery,
    users::GetUserAssetsResponseBody,
};

use crate::{
    utoipa::responses::{InternalServerError, Unauthorized},
    v1::response::ApiResponse,
};

/// Get the assets associated with the user.
///
/// All assets associated to the requesting user are returned in a list. If no
/// pagination query is added, the default page size is used.
#[utoipa::path(
    params(PagePaginationQuery, AssetSortingQuery),
    operation_id = "get_me_assets",
    tag = "api::v1::users",
    responses(
        (
            status = StatusCode::OK,
            description = "List of accessible assets successfully returned",
            body = GetUserAssetsResponseBody,
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
#[get("/users/me/assets")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    sorting: Query<AssetSortingQuery>,
    pagination: Query<PagePaginationQuery>,
) -> Result<ApiResponse<GetUserAssetsResponseBody>, ApiError> {
    let (assets_response, asset_count) = service
        .get_my_assets(current_user.into_inner(), sorting.into_inner(), &pagination)
        .await?;

    Ok(ApiResponse::new(assets_response).with_page_pagination(
        pagination.per_page,
        pagination.page,
        asset_count,
    ))
}
