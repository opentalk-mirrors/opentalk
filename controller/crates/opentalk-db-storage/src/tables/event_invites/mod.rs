// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains event invites table structs

mod event_invite;
mod event_invite_id;
mod new_event_invite;
mod update_event_invite;

pub use event_invite::EventInvite;
pub use event_invite_id::EventInviteId;
pub use new_event_invite::NewEventInvite;
pub use update_event_invite::UpdateEventInvite;
