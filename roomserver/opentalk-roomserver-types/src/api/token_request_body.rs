// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::utils::ExampleData;
use serde::{Deserialize, Serialize};

use crate::{client_parameters::ClientParameters, room_parameters::RoomParameters};

/// The request body for GET `/rooms/{room_id}/token`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(TokenRequestBody::example_data())))]
pub struct TokenRequestBody {
    /// Information regarding the requesting client
    pub client_parameters: ClientParameters,

    /// The room that the token is requested for.
    ///
    /// Once this was provided to the roomserver, further request do not need to include the room
    /// parameters
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub room_parameters: Option<RoomParameters>,
}

impl ExampleData for TokenRequestBody {
    fn example_data() -> Self {
        Self {
            client_parameters: ClientParameters::example_data(),
            room_parameters: Some(RoomParameters::example_data()),
        }
    }
}
