// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling event invites.

mod email_invite_role;
mod event_invite_status;
mod invite_role;

pub use email_invite_role::{EmailInviteRole, EmailInviteRoleType};
pub use event_invite_status::{EventInviteStatus, EventInviteStatusType};
pub use invite_role::{InviteRole, InviteRoleType};
