// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{email::EmailAddress, events::invites::EmailInviteRole};

/// Request body for the `PATCH /events/{event_id}/invites/email` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PatchEmailInviteBody {
    /// Email address of the user to modify the invite for
    pub email: EmailAddress,

    /// Invite role of the user
    pub role: Option<EmailInviteRole>,
}
