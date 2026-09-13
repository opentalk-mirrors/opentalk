// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains and reexports all functions required to select a speaker from the state machine.

use opentalk_types_signaling::ParticipantId;

mod next;
mod random;

pub use next::{select_next, select_specific};
pub use random::select_random;

use crate::{
    Session,
    history_entry::{HistoryEntry, HistoryEntryKind},
};

#[derive(Debug, PartialEq, Eq)]
pub enum SpeakerSelectionOutput {
    ContinueWith { update: Option<SpeakerUpdate> },
    End,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SpeakerUpdate {
    pub speaker: Option<ParticipantId>,
    pub history: Option<Vec<ParticipantId>>,
    pub remaining: Option<Vec<ParticipantId>>,
}

/// Selects the given participant (or None) as the current speaker and generates the appropriate
/// [`SpeakerUpdate`] if necessary.
/// Does not check if the participant exists or is even eligible to be speaker.
pub fn select_unchecked(
    session: &mut Session,
    participant: Option<ParticipantId>,
) -> Option<SpeakerUpdate> {
    let previous = std::mem::replace(&mut session.speaker, participant);

    if previous.is_none() && participant.is_none() {
        // nothing changed, return early
        return None;
    }

    // If there was a previous speaker add stop event to history
    if let Some(previous) = previous {
        session.history.push(HistoryEntry::stop(previous));
    }

    // If there is a new speaker add start event to history
    if let Some(participant) = participant {
        session.history.push(HistoryEntry::start(participant));
    }

    let history: Vec<ParticipantId> = session
        .history
        .iter()
        .filter_map(|entry| {
            if entry.kind == HistoryEntryKind::Start {
                Some(entry.participant)
            } else {
                None
            }
        })
        .collect();

    Some(SpeakerUpdate {
        speaker: participant,
        history: Some(history).filter(|history| !history.is_empty()),
        remaining: Some(session.remaining.clone()),
    })
}
