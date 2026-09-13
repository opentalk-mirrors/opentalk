// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/users/{user_id}`

use actix_web::{
    get,
    web::{Data, Json, Path, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{error::ApiError, users::PublicUserProfile};
use opentalk_types_common::users::UserId;

use crate::utoipa::responses::{Forbidden, InternalServerError, Unauthorized};

/// Get a user's public profile
///
/// Returns the public profile of a user.
#[utoipa::path(
    tag = "api::v1::users",
    operation_id = "get_user",
    params(
        ("user_id" = UserId, description = "The id of the user"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Information about the user",
            body = PublicUserProfile,
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
#[get("/users/{user_id}")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
    user_id: Path<UserId>,
) -> Result<Json<PublicUserProfile>, ApiError> {
    let user_profile = Json(
        service
            .get_user(current_user.into_inner(), user_id.into_inner())
            .await?,
    );
    Ok(user_profile)
}
