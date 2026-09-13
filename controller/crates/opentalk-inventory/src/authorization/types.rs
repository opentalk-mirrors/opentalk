// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::events::invites::InviteRole;

/// The role of an authorized user relating to an event.
#[derive(Clone, Debug)]
pub enum AuthorizationUserRole {
    /// Owner of the event.
    Owner,
    /// Invited to the event.
    Invited(InviteRole),
    /// No relation to the event.
    Unrelated {
        /// Is guest access is enabled.
        guest_access: bool,
    },
}
