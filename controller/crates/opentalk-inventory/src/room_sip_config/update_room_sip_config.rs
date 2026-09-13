// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::call_in::CallInPassword;

/// Representation of an update to a room SIP config in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRoomSipConfig {
    /// The call-in password for the room.
    pub password: Option<CallInPassword>,

    /// A flag indicating whether SIP participants should be sent to the lobby first.
    pub enable_lobby: Option<bool>,
}
