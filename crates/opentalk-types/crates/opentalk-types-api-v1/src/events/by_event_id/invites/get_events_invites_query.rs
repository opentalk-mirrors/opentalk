// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! This module contains types that are used for OpenTalk API V1 event invite endpoints.

use opentalk_types_common::events::invites::EventInviteStatus;

use crate::pagination::PagePaginationQuery;

/// The query passed to the `GET /events/{event_id}/invites` endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct GetEventsInvitesQuery {
    /// Results will be paginated by this pagination specification
    #[cfg_attr(feature = "serde", serde(flatten))]
    // TODO: This might not be working correctly for now, upstream fix is needed.
    // Upstream issue: https://github.com/juhaku/utoipa/issues/841
    #[cfg_attr(feature = "utoipa", param(inline))]
    pub pagination: PagePaginationQuery,

    /// If present, the results will be filtered by that state
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub status: Option<EventInviteStatus>,
}
