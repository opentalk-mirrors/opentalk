use opentalk_types_signaling::ParticipantId;

// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
#[derive(Debug, PartialEq, Eq)]
pub struct HistoryEntry {
    pub participant: ParticipantId,
    pub kind: HistoryEntryKind,
}

/// The kind of history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HistoryEntryKind {
    /// Participant gained its speaker status.
    Start,

    /// Participant lost its speaker status.
    Stop,
}

impl HistoryEntry {
    /// Creates a new Start-Entry with the current timestamp and the given participant.
    pub fn start(participant: ParticipantId) -> Self {
        Self {
            participant,
            kind: HistoryEntryKind::Start,
        }
    }

    /// Creates a new Stop-Entry with the current timestamp and the given participant.
    pub fn stop(participant: ParticipantId) -> Self {
        Self {
            participant,
            kind: HistoryEntryKind::Stop,
        }
    }
}
