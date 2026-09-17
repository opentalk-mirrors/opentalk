// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API endpoints under `v1/turn`

//! TURN related API structs and Endpoints
#![allow(deprecated)]

use actix_web::get;
use opentalk_types_api_v1::error::ErrorBody;

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
        (
            status = StatusCode::UNAUTHORIZED,
            description = "An `Authorization` header was sent, but the contained access token is \
                           invalid or expired. The endpoint itself requires no authentication, a \
                           request without an `Authorization` header always succeeds.",
            body = ErrorBody,
        ),
    ),
    security(),
)]
#[get("/turn")]
#[deprecated = "This endpoint and related turn settings will be removed in the future"]
pub async fn get() -> NoContent {
    NoContent {}
}
