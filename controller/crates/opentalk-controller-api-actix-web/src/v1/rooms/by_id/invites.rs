// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Removed API endpoints under `v1/rooms/{room_id}/invites`
//!
//! The invite-code API has been removed. To signal to clients that these
//! endpoints are permanently gone instead of merely missing, all requests to
//! them respond with `410 Gone` rather than `404 Not Found`.

use actix_web::{Resource, web};

use crate::response::Gone;

/// Handler that responds with `410 Gone` for any removed invite endpoint.
async fn gone() -> Gone {
    Gone
}

/// `410 Gone` resource for the removed `v1/rooms/{room_id}/invites` endpoints.
pub fn collection() -> Resource {
    web::resource("/rooms/{room_id}/invites").to(gone)
}

/// `410 Gone` resource for the removed `v1/rooms/{room_id}/invites/{invite_code}` endpoints.
pub fn by_code() -> Resource {
    web::resource("/rooms/{room_id}/invites/{invite_code}").to(gone)
}
