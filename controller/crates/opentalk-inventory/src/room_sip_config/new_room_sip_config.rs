// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    call_in::{CallInId, CallInPassword},
    rooms::RoomId,
};

/// The representation of a new room SIP config that should be stored into the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRoomSipConfig {
    /// The id of the room to which the SIP config belongs.
    pub room: RoomId,

    /// The call-in id for the room.
    pub sip_id: CallInId,

    /// The call-in password for the room.
    pub password: CallInPassword,

    /// A flag indicating whether SIP participants should be sent to the lobby first.
    pub enable_lobby: bool,
}

impl NewRoomSipConfig {
    /// Create an new [`NewRoomSipConfig`].
    pub fn new(room_id: RoomId, enable_lobby: bool) -> Self {
        Self {
            room: room_id,
            sip_id: CallInId::generate(),
            password: CallInPassword::generate(),
            enable_lobby,
        }
    }
}
