// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/users/find`

use actix_web::{
    get,
    web::{Data, Json, Query, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    users::{GetFindQuery, GetFindResponseBody},
};

use crate::utoipa::responses::{InternalServerError, Unauthorized};

/// Find users
///
/// Query users for autocomplete fields
#[utoipa::path(
    params(GetFindQuery),
    operation_id = "find",
    tag = "api::v1::users",
    responses(
        (
            status = StatusCode::OK,
            description = "Search results",
            body = GetFindResponseBody,
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
#[get("/users/find")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    query: Query<GetFindQuery>,
) -> Result<Json<GetFindResponseBody>, ApiError> {
    let result = Json(
        service
            .find_users(current_user.into_inner(), query.into_inner())
            .await?,
    );
    Ok(result)
}
