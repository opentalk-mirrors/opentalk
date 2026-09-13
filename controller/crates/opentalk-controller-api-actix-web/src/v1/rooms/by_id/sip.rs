// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}/sip`

use actix_web::{
    HttpResponse, delete, get, put,
    web::{Data, Json, Path},
};
use opentalk_controller_service_facade::OpenTalkControllerService;
use opentalk_types_api_v1::{
    error::ApiError,
    rooms::by_room_id::sip::{PutSipConfigRequestBody, SipConfigResource},
};
use opentalk_types_common::rooms::{RoomId, RoomIdOrAlias};

use crate::{
    response::NoContent,
    utoipa::responses::{Forbidden, InternalServerError, NotFound, Unauthorized},
};

/// Get the sip config for the specified room.
///
/// Returns the sip config if available for the room, otherwise `404 NOT_FOUND`
/// is returned.
#[utoipa::path(
    operation_id = "get_room_sip",
    tag = "api::v1::sip_configs",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "The SIP config is successfully returned",
            body = SipConfigResource,
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
#[get("/rooms/{room_id_or_alias}/sip")]
pub async fn get(
    service: Data<dyn OpenTalkControllerService>,
    room_id_or_alias: Path<RoomIdOrAlias>,
) -> Result<Json<SipConfigResource>, ApiError> {
    Ok(Json(
        service
            .get_sip_config(room_id_or_alias.into_inner())
            .await?,
    ))
}

/// Modify the sip configuration of a room. A new sip configuration is created
/// if none was set before.
///
/// Returns the new modified sip configuration.
#[utoipa::path(
    tag = "api::v1::sip_configs",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
    ),
    request_body = PutSipConfigRequestBody,
    responses(
        (
            status = StatusCode::OK,
            description = "The SIP configuration was updated",
            body = SipConfigResource,
        ),
        (
            status = StatusCode::CREATED,
            description = "A new SIP configuration was created",
            body = SipConfigResource,
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
#[put("/rooms/{room_id_or_alias}/sip")]
pub async fn put(
    service: Data<dyn OpenTalkControllerService>,
    room_id_or_alias: Path<RoomIdOrAlias>,
    modify_sip_config: Json<PutSipConfigRequestBody>,
) -> Result<HttpResponse, ApiError> {
    let (sip_config_resource, newly_created) = service
        .set_sip_config(
            room_id_or_alias.into_inner(),
            modify_sip_config.into_inner(),
        )
        .await?;

    let mut response = if newly_created {
        HttpResponse::Created()
    } else {
        HttpResponse::Ok()
    };

    Ok(response.json(sip_config_resource))
}

/// Delete the SIP configuration of a room.
///
/// This removes the dial-in functionality from the room.
#[utoipa::path(
    operation_id = "delete_room_sip",
    tag = "api::v1::sip_configs",
    params(
        ("room_id_or_alias" = RoomId, description = "Either the id or the alias of the room"),
    ),
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "The SIP configuration was successfully deleted",
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
#[delete("/rooms/{room_id_or_alias}/sip")]
pub async fn delete(
    service: Data<dyn OpenTalkControllerService>,
    room_id_or_alias: Path<RoomIdOrAlias>,
) -> Result<NoContent, ApiError> {
    service
        .delete_sip_config(room_id_or_alias.into_inner())
        .await?;

    Ok(NoContent)
}
