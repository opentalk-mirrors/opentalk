// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/rooms/{room_id}/start_invited`
//!
//! Starting a session with an invitation code has been removed. This endpoint
//! is kept for backwards compatibility and permanently redirects to
//! `v1/rooms/{room_id}/start`.

use actix_web::{HttpResponse, http::header, post, web::Path};
use opentalk_types_common::rooms::{RoomId, RoomIdOrAlias};

/// Start a signaling session with the roomserver with an invitation code
///
/// This endpoint has to be called in order to get a signaling token for the roomserver. This endpoint returns the token
/// and the corresponding roomserver address. Call the  *GET `<roomserver_address>/signaling/<token>`* endpoint to
/// establish the websocket connection with the roomserver.
#[utoipa::path(
    operation_id = "start_invited",
    tag = "api::v1::rooms",
    params(
        ("room_id_or_alias" = RoomId, description = "Either the id or the alias of the room"),
    ),
    request_body = (),
    responses(
        (
            status = StatusCode::PERMANENT_REDIRECT,
            description = "Redirects to *POST `/v1/rooms/{room_id_or_alias}/start`*",
            headers(
             ("location", description = "Target URL: `/v1/rooms/{room_id_or_alias}/start`"),
            ),
        ),
    ),
    security(),
)]
#[post("/rooms/{room_id_or_alias}/start_invited")]
pub async fn post(room_id_or_alias: Path<RoomIdOrAlias>) -> HttpResponse {
    let room_id_or_alias = room_id_or_alias.into_inner();
    // This endpoint is currently public; if re-implementing, please adjust the permissions.
    HttpResponse::PermanentRedirect()
        .insert_header((
            header::LOCATION,
            format!("/v1/rooms/{}/start", room_id_or_alias),
        ))
        .finish()
}
