// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms/{room_id}/invites`.

mod get_rooms_invites_response_body;
mod invite_resource;
mod post_invite_request_body;
mod post_invite_verify_request_body;
mod post_invite_verify_response_body;
mod put_invite_request_body;

pub use get_rooms_invites_response_body::GetRoomsInvitesResponseBody;
pub use invite_resource::InviteResource;
pub use post_invite_request_body::PostInviteRequestBody;
pub use post_invite_verify_request_body::PostInviteVerifyRequestBody;
pub use post_invite_verify_response_body::PostInviteVerifyResponseBody;
pub use put_invite_request_body::PutInviteRequestBody;
