// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::whisper_id::WhisperId;

/// Participants targeted in a whisper group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantTargets {
    /// The id of the targeted whisper group
    pub whisper_id: WhisperId,

    /// The participants that are affected
    pub participant_ids: BTreeSet<ParticipantId>,
}
