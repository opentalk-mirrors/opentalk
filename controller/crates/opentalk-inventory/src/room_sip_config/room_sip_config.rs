// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    call_in::{CallInId, CallInPassword},
    rooms::RoomId,
};

/// The representation of a room SIP config in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomSipConfig {
    /// The id of the config.
    pub id: i64,

    /// The id of the room to which the SIP config belongs.
    pub room: RoomId,

    /// The call-in id for the room.
    pub sip_id: CallInId,

    /// The call-in password for the room.
    pub password: CallInPassword,

    /// A flag indicating whether SIP participants should be sent to the lobby first.
    pub lobby: bool,
}
