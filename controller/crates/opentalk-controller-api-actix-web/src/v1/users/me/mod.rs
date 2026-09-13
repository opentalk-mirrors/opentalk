// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/users/me`

pub mod assets;
pub mod event_favorites;
pub mod pending_invites;
pub mod tariff;

use actix_web::{
    Either, get, patch,
    web::{Data, Json, ReqData},
};
use oauth2::AccessToken;
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::ApiError,
    users::{PrivateUserProfile, me::PatchMeRequestBody},
};

use crate::{
    response::NoContent,
    utoipa::responses::{InternalServerError, Unauthorized},
};

/// Patch the current user's profile
///
/// Fields that are not provided in the request body will remain unchanged.
#[utoipa::path(
    request_body = PatchMeRequestBody,
    tag = "api::v1::users",
    operation_id = "patch_users_me",
    responses(
        (
            status = StatusCode::OK,
            description = "User profile was successfully updated",
            body = PrivateUserProfile
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = r"Could not modify the user's profile due to wrong
                syntax or bad values",
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
#[patch("/users/me")]
pub async fn patch(
    service: Data<dyn OpenTalkControllerService>,
    access_token: ReqData<AccessToken>,
    current_user: ReqData<RequestUser>,
    patch: Json<PatchMeRequestBody>,
) -> Result<Either<Json<PrivateUserProfile>, NoContent>, ApiError> {
    let current_user = current_user.into_inner();

    let user_profile = service
        .patch_me(current_user.clone(), patch.into_inner(), &access_token)
        .await?;

    match user_profile {
        Some(user_profile) => Ok(Either::Left(Json(user_profile))),
        _ => Ok(Either::Right(NoContent)),
    }
}

/// Get the current user's profile
///
/// Returns the private user profile of the currently logged-in user. This
/// private profile contains information that is not visible in the public
/// profile, such as tariff status or the used storage.
#[utoipa::path(
    operation_id = "get_users_me",
    tag = "api::v1::users",
    responses(
        (
            status = StatusCode::OK,
            description = "Information about the logged in user",
            body = PrivateUserProfile,
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
#[get("/users/me")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
) -> Result<Json<PrivateUserProfile>, ApiError> {
    Ok(Json(service.get_me(current_user.into_inner()).await?))
}
