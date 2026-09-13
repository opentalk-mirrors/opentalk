// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::auth::{ResumptionToken, TicketToken};

/// Response body for `POST /**/**/start` endpoints
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostServiceStartResponseBody {
    /// The ticket token
    pub ticket: TicketToken,
    /// The resumption token
    pub resumption: ResumptionToken,
}
