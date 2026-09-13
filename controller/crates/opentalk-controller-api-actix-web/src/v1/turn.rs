// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/turn`

//! TURN related API structs and Endpoints
#![allow(deprecated)]

use actix_web::get;

use crate::response::NoContent;

/// Deprecated endpoint, only available for backwards compatibility.
///
/// This endpoint is deprecated and will be removed in the future.
/// It returns an empty answer regardless any configuration.
#[utoipa::path(
    operation_id = "get_turn",
    tag="api::v1::turn",
    responses(
        (
            status = StatusCode::NO_CONTENT,
            description = "No TURN servers have been configured",
        ),
    ),
)]
#[get("/turn")]
#[deprecated = "This endpoint and related turn settings will be removed in the future"]
pub async fn get() -> NoContent {
    NoContent {}
}
