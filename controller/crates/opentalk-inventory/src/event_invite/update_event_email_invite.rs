// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::events::invites::EmailInviteRole;

/// Representation of an update to a room in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEventEmailInvite {
    /// The role of the invited person.
    pub role: Option<EmailInviteRole>,
}
