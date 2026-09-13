// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{events::EventId, users::UserId};

/// Path parameters for the `DELETE /events/{event_id}/invites/{user_id}` endpoint
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct DeleteEventInvitePath {
    /// ID of the event to delete the invite for
    pub event_id: EventId,
    /// ID of the user to delete the invite for
    pub user_id: UserId,
}
