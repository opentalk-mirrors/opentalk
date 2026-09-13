// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    call_in::{CallInId, CallInPassword},
    rooms::RoomId,
    utils::ExampleData,
};

/// Response for the `GET /rooms/{room_id}/sip` endpoint
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(SipConfigResource::example_data())))]
pub struct SipConfigResource {
    /// The room id
    pub room: RoomId,

    /// The SIP ID
    pub sip_id: CallInId,

    /// The SIP password
    pub password: CallInPassword,
    /// Flag if the room is a lobby
    pub lobby: bool,
}

impl ExampleData for SipConfigResource {
    fn example_data() -> Self {
        Self {
            room: RoomId::example_data(),
            sip_id: CallInId::example_data(),
            password: CallInPassword::example_data(),
            lobby: false,
        }
    }
}
