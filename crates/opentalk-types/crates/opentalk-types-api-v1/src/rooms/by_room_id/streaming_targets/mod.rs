// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms/{room_id}/streaming_targets`.

mod get_room_streaming_target_response_body;
mod get_room_streaming_targets_response_body;
mod patch_room_streaming_target_request_body;
mod patch_room_streaming_target_response_body;
mod post_room_streaming_target_request_body;
mod post_room_streaming_target_response_body;
mod room_and_streaming_target_id;

pub use get_room_streaming_target_response_body::GetRoomStreamingTargetResponseBody;
pub use get_room_streaming_targets_response_body::GetRoomStreamingTargetsResponseBody;
pub use patch_room_streaming_target_request_body::PatchRoomStreamingTargetRequestBody;
pub use patch_room_streaming_target_response_body::PatchRoomStreamingTargetResponseBody;
pub use post_room_streaming_target_request_body::PostRoomStreamingTargetRequestBody;
pub use post_room_streaming_target_response_body::PostRoomStreamingTargetResponseBody;
pub use room_and_streaming_target_id::RoomAndStreamingTargetId;
