// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains event email invites table structs

mod event_email_invite;
mod new_event_email_invite;
mod update_event_email_invite;

pub use event_email_invite::EventEmailInvite;
pub use new_event_email_invite::NewEventEmailInvite;
pub use update_event_email_invite::UpdateEventEmailInvite;
