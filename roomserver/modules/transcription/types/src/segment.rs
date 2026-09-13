// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::time::Timestamp;
use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Transcription segment to receive and correctly display transcribed text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TranscriptionSegment {
    /// Participant which the segment belongs to
    pub participant_id: ParticipantId,
    /// LiveKit track id the segment belongs to
    pub track_id: String,
    /// Segment start timestamp
    pub starts_at: Timestamp,
    /// Segment end timestamp
    pub ends_at: Timestamp,
    /// Transcribed text for the segment
    pub text: String,
}
