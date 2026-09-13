// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # DEPRECATED
//!
//! API endpoints under `v1/rooms/{room_id_or_alias}/roomserver/start_invited`
//!
//! This endpoint has been permanently moved to `/v1/rooms/{room_id_or_alias}/start`.

use actix_web::{HttpResponse, http::header, post, web::Path};
use opentalk_types_common::rooms::RoomIdOrAlias;

#[post("/rooms/{room_id_or_alias}/roomserver/start_invited")]
#[deprecated]
pub async fn post(room_id_or_alias: Path<RoomIdOrAlias>) -> HttpResponse {
    let room_id = room_id_or_alias.into_inner();
    HttpResponse::MovedPermanently()
        .insert_header((header::LOCATION, format!("/v1/rooms/{}/start", room_id)))
        .finish()
}
