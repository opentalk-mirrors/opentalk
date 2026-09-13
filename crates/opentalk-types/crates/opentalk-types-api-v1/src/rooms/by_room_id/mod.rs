// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms/{room_id}`.

pub mod assets;
pub mod invites;
pub mod sip;
pub mod streaming_targets;

mod delete_room_query;
mod get_room_event_response_body;
mod patch_rooms_request_body;
mod post_rooms_roomserver_start_request_body;
mod post_rooms_start_request_body;
mod rooms_start_response_body;
mod roomserver_start_response_body;

pub use delete_room_query::DeleteRoomQuery;
pub use get_room_event_response_body::GetRoomEventResponseBody;
pub use patch_rooms_request_body::PatchRoomsRequestBody;
pub use post_rooms_roomserver_start_request_body::PostRoomsRoomserverStartRequestBody;
pub use post_rooms_start_request_body::PostRoomsStartRequestBody;
pub use rooms_start_response_body::RoomsStartResponseBody;
pub use roomserver_start_response_body::RoomserverStartResponseBody;
