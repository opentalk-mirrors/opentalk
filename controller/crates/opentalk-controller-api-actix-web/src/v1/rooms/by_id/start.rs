// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id_or_alias}/start`

use actix_web::{
    post,
    web::{Data, Json, Path, ReqData},
};
use opentalk_controller_service_facade::{OpenTalkControllerService, RequestUser};
use opentalk_types_api_v1::{
    error::{ApiError, ErrorBody},
    rooms::by_room_id::{PostRoomsRoomserverStartRequestBody, RoomserverStartResponseBody},
};
use opentalk_types_common::rooms::RoomIdOrAlias;

use crate::{host::Host, utoipa::responses::InternalServerError};

/// Start a signaling session with the roomserver as a registered user
///
/// This endpoint has to be called in order to get a signaling token for the roomserver. This endpoint returns the token
/// and the corresponding roomserver address. Call the  *GET `<roomserver_address>/signaling/<token>`* endpoint to
/// establish the websocket connection with the roomserver.
#[utoipa::path(
    operation_id = "start",
    tag = "api::v1::rooms",
    params(
        ("room_id_or_alias" = RoomIdOrAlias, description = "Either the id or the alias of the room"),
    ),
    responses(
        (
            status = StatusCode::OK,
            description = "Returns the roomserver token and roomserver address",
            body = RoomserverStartResponseBody,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = r"The provided AccessToken is expired or the
                provided ID- or Access-Token is invalid. The WWW-Authenticate
                header will contain a error description 'session expired' to
                distinguish between an invalid and an expired token.",
            body = ErrorBody,
            headers(
                (
                    "www-authenticate",
                    description = "Will contain 'session expired' to distinguish between an invalid and an expired token"
                ),
            ),
        ),
        (
            status = StatusCode::FORBIDDEN,
            description = "The user has not been invited to join the room or has been banned from entering this room",
            body = ErrorBody,
            examples(
                ("UserNotInvited" = (summary = "User has not been invited to join the room", value = json!(ApiError::forbidden().body))),
            ),
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "The specified room could not be found or it has no event associated with it",
            body = ErrorBody,
            example = json!(ApiError::not_found().body),
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
#[post("/rooms/{room_id_or_alias}/start")]
pub async fn post(
    service: Data<dyn OpenTalkControllerService>,
    current_user: Option<ReqData<RequestUser>>,
    room_id_or_alias: Path<RoomIdOrAlias>,
    request: Json<PostRoomsRoomserverStartRequestBody>,
    host: Host,
) -> Result<Json<RoomserverStartResponseBody>, ApiError> {
    let user = current_user.map(|user| user.into_inner());
    let response = Json(
        service
            .start_room_session(
                user,
                room_id_or_alias.into_inner(),
                request.into_inner(),
                host.into_inner(),
            )
            .await?,
    );
    Ok(response)
}
