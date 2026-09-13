// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{auth::ResumptionToken, rooms::BreakoutRoomId, utils::ExampleData};

/// The JSON body expected when making a *POST /rooms/{room_id}/start*
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema (example = json!(PostRoomsStartRequestBody::example_data()))
)]
pub struct PostRoomsStartRequestBody {
    /// Optional breakout room ID
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub breakout_room: Option<BreakoutRoomId>,

    /// The resumption token for the room
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    // Field is non-required already, utoipa adds a `nullable: true` entry
    // by default which creates a false positive in the spectral linter when
    // combined with example data.
    #[cfg_attr(feature = "utoipa", schema(nullable = false))]
    pub resumption: Option<ResumptionToken>,
}

impl ExampleData for PostRoomsStartRequestBody {
    fn example_data() -> Self {
        Self {
            breakout_room: Some(BreakoutRoomId::from_u128(0xbadcafe)),
            resumption: Some(ResumptionToken::example_data()),
        }
    }
}
