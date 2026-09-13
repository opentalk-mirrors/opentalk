// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::BTreeSet;

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

#[cfg(feature = "actix")]
pub mod continuation_buffer;
pub mod module_error;
pub mod signaling_context;
pub mod websocket;

pub enum MessageTarget {
    AllParticipantsInRoom,
    Participant(ParticipantId),
    Participants(BTreeSet<ParticipantId>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingCommand {
    pub namespace: ModuleId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<u64>,
    pub payload: Box<serde_json::value::RawValue>,
}

impl SignalingCommand {
    pub fn new(
        namespace: ModuleId,
        transaction_id: Option<u64>,
        payload: Box<serde_json::value::RawValue>,
    ) -> Self {
        Self {
            namespace,
            transaction_id,
            payload,
        }
    }
}
