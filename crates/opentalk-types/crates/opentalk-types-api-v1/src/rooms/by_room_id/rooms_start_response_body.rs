// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    auth::{ResumptionToken, TicketToken},
    utils::ExampleData,
};

/// The JSON body returned from the start endpoints supporting session resumption
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomsStartResponseBody::example_data())))]
pub struct RoomsStartResponseBody {
    /// The ticket token for the room
    pub ticket: TicketToken,

    /// The resumption token for the room
    pub resumption: ResumptionToken,
}

impl ExampleData for RoomsStartResponseBody {
    fn example_data() -> Self {
        Self {
            ticket: TicketToken::example_data(),
            resumption: ResumptionToken::example_data(),
        }
    }
}
