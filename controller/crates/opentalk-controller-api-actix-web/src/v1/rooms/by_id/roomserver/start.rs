// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! # DEPRECATED
//!
//! Permanently redirected to `POST /v1/rooms/{room_id_or_alias}/start`
//!
//! This endpoint has been deprecated and permanently moved. Clients should update
//! their requests to use the new endpoint path.

use actix_web::{HttpResponse, http::header, post, web::Path};
use opentalk_types_common::rooms::RoomIdOrAlias;

#[post("/rooms/{room_id_or_alias}/roomserver/start")]
#[deprecated]
pub async fn post(room_id_or_alias: Path<RoomIdOrAlias>) -> HttpResponse {
    let room_id = room_id_or_alias.into_inner();
    HttpResponse::MovedPermanently()
        .insert_header((header::LOCATION, format!("/v1/rooms/{}/start", room_id)))
        .finish()
}
