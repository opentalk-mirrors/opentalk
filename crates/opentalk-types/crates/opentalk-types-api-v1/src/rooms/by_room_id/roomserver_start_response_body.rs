// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{roomserver::Token, utils::ExampleData};

/// The JSON body returned from the roomserver start endpoint
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomserverStartResponseBody::example_data())))]
pub struct RoomserverStartResponseBody {
    /// The token to start a signaling session with the roomserver
    pub token: Token,
    /// The address of the roomserver
    pub roomserver_address: String,
}

impl ExampleData for RoomserverStartResponseBody {
    fn example_data() -> Self {
        Self {
            token: Token::example_data(),
            roomserver_address: "http://0.0.0.0:9000".into(),
        }
    }
}
