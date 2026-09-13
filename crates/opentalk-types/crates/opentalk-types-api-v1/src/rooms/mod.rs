// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms`.

pub mod by_room_id;
pub mod name;
pub mod streaming_targets;

mod get_rooms_response_body;
mod post_rooms_request_body;
mod room_resource;

pub use get_rooms_response_body::GetRoomsResponseBody;
pub use post_rooms_request_body::PostRoomsRequestBody;
pub use room_resource::RoomResource;
