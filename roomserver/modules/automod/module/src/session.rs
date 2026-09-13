// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types_automod::config::Parameter;
use opentalk_types_signaling::ParticipantId;

use crate::history_entry::{HistoryEntry, HistoryEntryKind};

/// The state of an automod session.
#[derive(Debug)]
pub struct Session {
    pub issued_by: ParticipantId,
    pub parameter: Parameter,
    pub remaining: Vec<ParticipantId>,
    pub history: Vec<HistoryEntry>,
    pub speaker: Option<ParticipantId>,
}

impl Session {
    pub fn new(
        issued_by: ParticipantId,
        parameter: Parameter,
        remaining: Vec<ParticipantId>,
    ) -> Self {
        Self {
            issued_by,
            parameter,
            remaining,
            history: Vec::new(),
            speaker: None,
        }
    }

    /// Returns all participants that have started speaking in this session.
    pub fn participant_history(&self) -> impl Iterator<Item = ParticipantId> {
        self.history.iter().filter_map(|entry| {
            if entry.kind == HistoryEntryKind::Start {
                Some(entry.participant)
            } else {
                None
            }
        })
    }
}
