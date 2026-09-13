use std::collections::BTreeMap;

// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use super::WhisperState;
use crate::whisper_id::WhisperId;

/// Representation of an existing whisper group
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhisperGroup {
    /// Unique id for the whisper group
    pub whisper_id: WhisperId,
    /// Participants of the whisper group
    pub participants: BTreeMap<ParticipantId, WhisperState>,
}

impl Default for WhisperGroup {
    fn default() -> Self {
        Self {
            whisper_id: WhisperId::generate(),
            participants: BTreeMap::new(),
        }
    }
}

impl WhisperGroup {
    /// Check if the given participant is contained in this group
    pub fn contains(&self, participant_id: &ParticipantId) -> bool {
        self.participants.contains_key(participant_id)
    }

    /// Check if the given participant has accepted the invite to this group
    pub fn has_accepted(&self, participant_id: &ParticipantId) -> bool {
        let Some(state) = self.participants.get(participant_id) else {
            return false;
        };

        matches!(state, WhisperState::Accepted | WhisperState::Creator)
    }
}
