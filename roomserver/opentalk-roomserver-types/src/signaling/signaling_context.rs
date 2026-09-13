// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::rooms::RoomId;

use crate::client_parameters::ClientParameters;

/// The context to start a signaling session
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalingClientContext {
    pub room_id: RoomId,
    pub client_parameters: ClientParameters,
}

impl SignalingClientContext {
    pub fn new(room_id: RoomId, client_parameters: ClientParameters) -> Self {
        Self {
            room_id,
            client_parameters,
        }
    }
}
