// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::state::ChatChunk;

/// Private chat history
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateHistory {
    /// Private chat correspondent
    pub correspondent: ParticipantId,

    /// Private chat history
    pub history: ChatChunk,
}
