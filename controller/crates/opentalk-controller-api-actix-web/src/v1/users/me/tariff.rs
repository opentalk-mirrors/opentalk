// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/users/me/tariff`

use actix_web::{
    get,
    web::{Data, Json, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::tariffs::TariffResource;

use crate::utoipa::responses::{InternalServerError, Unauthorized};

/// Get the current user tariff information.
///
/// Returns the tariff information for the currently logged in user.
#[utoipa::path(
    tag = "api::v1::users",
    operation_id = "get_me_tariff",
    responses(
        (
            status = StatusCode::OK,
            description = "Information about the tariff of the current user",
            body = TariffResource,
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
#[get("/users/me/tariff")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    current_user: ReqData<RequestUser>,
) -> Result<Json<TariffResource>, ApiError> {
    let resource = service.get_my_tariff(current_user.into_inner()).await?;
    Ok(Json(resource))
}
