// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

use crate::{
    state::{WhisperGroup, WhisperState},
    whisper_id::WhisperId,
};

/// Frontend representation of a [`WhisperGroup`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhisperGroupOutgoing {
    /// Unique id for the whisper group
    pub whisper_id: WhisperId,
    /// A list of participants in the whisper group
    pub participants: Vec<WhisperParticipant>,
}

/// Representation of a whisper participant
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhisperParticipant {
    /// The participant id
    pub participant_id: ParticipantId,
    /// The participants whisper state
    pub state: WhisperState,
}

impl From<WhisperGroup> for WhisperGroupOutgoing {
    fn from(value: WhisperGroup) -> Self {
        let participants = value
            .participants
            .into_iter()
            .map(|(participant_id, state)| WhisperParticipant {
                participant_id,
                state,
            })
            .collect();

        Self {
            whisper_id: value.whisper_id,
            participants,
        }
    }
}
