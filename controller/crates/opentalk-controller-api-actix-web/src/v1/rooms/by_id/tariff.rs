// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}/tariff`

use actix_web::{
    get,
    web::{Data, Json, Path},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::error::ApiError;
use opentalk_types_common::{rooms::RoomIdOrAlias, tariffs::TariffResource};

use crate::utoipa::responses::{Forbidden, InternalServerError, Unauthorized};

/// Get a room's tariff
///
/// This returns the tariff that applies to the room, typically the tariff of
/// the room creator.
#[utoipa::path(
    operation_id = "get_room_tariff",
    tag = "api::v1::rooms",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The room's tariff was successfully retrieved",
            body = TariffResource
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
#[get("/rooms/{room_id_or_alias}/tariff")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    room_id_or_alias: Path<RoomIdOrAlias>,
) -> Result<Json<TariffResource>, ApiError> {
    Ok(Json(
        service
            .get_room_tariff(room_id_or_alias.into_inner())
            .await?,
    ))
}
