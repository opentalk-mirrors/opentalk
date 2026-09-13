// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_automod::{
    config::{Parameter, SelectionStrategy},
    event::AutomodError,
};
use opentalk_types_signaling::ParticipantId;
use rand::Rng;

use crate::{Session, speaker_selection::SpeakerSelectionOutput};

/// Depending on the session parameters, inspect/change the `session`s state to select the next
/// user to be speaker.
#[tracing::instrument(skip(rng), level = "debug")]
pub fn select_next<R: Rng>(
    session: &mut Session,
    user_selected: Option<ParticipantId>,
    rng: &mut R,
) -> Result<SpeakerSelectionOutput, AutomodError> {
    let participant = match session.parameter {
        Parameter {
            selection_strategy: SelectionStrategy::None,
            allow_double_selection,
            ..
        } => select_next_nomination(session, user_selected, allow_double_selection)?,
        Parameter {
            selection_strategy: SelectionStrategy::Playlist,
            ..
        } => {
            if session.remaining.is_empty() {
                super::select_unchecked(session, None);
                return Ok(SpeakerSelectionOutput::End);
            }
            Some(session.remaining.remove(0))
        }
        Parameter {
            selection_strategy: SelectionStrategy::Nomination,
            allow_double_selection,
            ..
        } => select_next_nomination(session, user_selected, allow_double_selection)?,
        Parameter {
            selection_strategy: SelectionStrategy::Random,
            ..
        } => {
            if user_selected.is_none() {
                return Ok(super::select_random(session, rng));
            }
            user_selected
        }
    };

    let update = super::select_unchecked(session, participant);
    Ok(SpeakerSelectionOutput::ContinueWith { update })
}

pub fn select_specific(
    session: &mut Session,
    user_selected: Option<ParticipantId>,
    allow_double_selection: bool,
) -> Result<SpeakerSelectionOutput, AutomodError> {
    let participant = select_next_nomination(session, user_selected, allow_double_selection)?;
    let update = super::select_unchecked(session, participant);

    Ok(SpeakerSelectionOutput::ContinueWith { update })
}

/// Returns the next (if any) participant to be selected inside a `Nomination` selection strategy.
#[tracing::instrument(level = "debug")]
fn select_next_nomination(
    session: &mut Session,
    user_selected: Option<ParticipantId>,
    allow_double_selection: bool,
) -> Result<Option<ParticipantId>, AutomodError> {
    // get user selection
    let Some(participant) = user_selected else {
        // No next user nominated, unset current speaker
        return Ok(None);
    };

    // Different approaches depending on `allow_double_selection`
    if allow_double_selection {
        // Double selection is allowed:
        // Just check if the given participant is inside the allow_list
        if session.remaining.contains(&participant) {
            Ok(Some(participant))
        } else {
            Err(AutomodError::InvalidSelection)
        }
    } else {
        // Double selection is disallowed:
        // Try to remove the participant from the allow_list.
        // If the participant wasn't inside the list invalid selection was made
        let Some(index) = session
            .remaining
            .iter()
            .position(|participant_id| *participant_id == participant)
        else {
            return Err(AutomodError::InvalidSelection);
        };

        session.remaining.remove(index);
        Ok(Some(participant))
    }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use pretty_assertions::assert_eq;
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;
    use crate::{
        history_entry::HistoryEntry,
        speaker_selection::{self, SpeakerUpdate},
    };

    fn rng() -> StdRng {
        StdRng::seed_from_u64(0)
    }

    fn assert_history_without_timestamp(lhs: &[HistoryEntry], rhs: &[HistoryEntry]) {
        assert_eq!(lhs.len(), rhs.len());
        lhs.iter().zip(rhs.iter()).for_each(|(lhs, rhs)| {
            assert_eq!(lhs.kind, rhs.kind);
            assert_eq!(lhs.participant, rhs.participant);
        })
    }

    /// Test next when selection_strategy is Nomination and reselection is allowed
    #[test_log::test(tokio::test)]
    async fn nomination_reselection_allowed() {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);

        let mut session = Session::new(
            p1,
            Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: true,
                auto_append_on_join: false,
            },
            vec![p1],
        );
        session.history.push(HistoryEntry::start(p1));
        // Add current speaker
        session.speaker = Some(p1);

        // Check with nominee in history
        let next = select_next(&mut session, Some(p1), &mut rng).unwrap();

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p1, p1]);

        assert_history_without_timestamp(
            &session.history,
            &[
                HistoryEntry::start(p1),
                HistoryEntry::stop(p1),
                HistoryEntry::start(p1),
            ],
        );

        assert_eq!(session.speaker, Some(p1));

        assert_eq!(
            next,
            SpeakerSelectionOutput::ContinueWith {
                update: Some(SpeakerUpdate {
                    speaker: Some(p1),
                    history: Some(vec![p1, p1]),
                    remaining: Some(vec![p1])
                })
            }
        );

        session.parameter = Parameter {
            selection_strategy: SelectionStrategy::Nomination,
            show_remaining: false,
            time_limit: None,
            allow_double_selection: false,
            auto_append_on_join: false,
        };

        // Check with nominee in history
        let next = select_next(&mut session, Some(p1), &mut rng).unwrap();
        assert_eq!(
            next,
            SpeakerSelectionOutput::ContinueWith {
                update: Some(SpeakerUpdate {
                    speaker: Some(p1),
                    history: Some(vec![p1, p1, p1]),
                    remaining: Some(vec![])
                })
            }
        );

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p1, p1, p1]);
        assert_history_without_timestamp(
            &session.history,
            &[
                HistoryEntry::start(p1),
                HistoryEntry::stop(p1),
                HistoryEntry::start(p1),
                HistoryEntry::stop(p1),
                HistoryEntry::start(p1),
            ],
        );

        assert_eq!(session.speaker, Some(p1));
    }

    /// Test next when selection_strategy is Nomination and an allow_list containing only 2 of 3
    /// possible participants
    #[test_log::test(tokio::test)]
    async fn nomination() {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        let mut session = Session::new(
            p1,
            Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            vec![p1, p2],
        );

        // Check allowed participant
        let next = select_next(&mut session, Some(p1), &mut rng).unwrap();
        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p1]);

        assert_history_without_timestamp(&session.history, &[HistoryEntry::start(p1)]);

        assert_eq!(session.speaker, Some(p1));

        assert_eq!(
            next,
            SpeakerSelectionOutput::ContinueWith {
                update: Some(SpeakerUpdate {
                    speaker: Some(p1),
                    history: Some(vec![p1]),
                    remaining: Some(vec![p2])
                })
            }
        );

        // Check non-allowed participant
        let next = select_next(&mut session, Some(p3), &mut rng);
        assert_matches!(next, Err(AutomodError::InvalidSelection));

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p1]);

        assert_history_without_timestamp(&session.history, &[HistoryEntry::start(p1)]);

        assert_eq!(session.speaker, Some(p1));

        // Check with nominee in history
        let next = select_next(&mut session, Some(p1), &mut rng);
        assert_matches!(next, Err(AutomodError::InvalidSelection));

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p1]);

        assert_history_without_timestamp(&session.history, &[HistoryEntry::start(p1)]);

        assert_eq!(session.speaker, Some(p1));
    }

    /// Test next when selection_strategy is Nomination but no nomination was given and the
    /// allow_list is empty
    #[test_log::test(tokio::test)]
    async fn nomination_without_nomination_empty_allow_list() {
        let mut rng = rng();

        let mut session = Session::new(
            ParticipantId::nil(),
            Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            Vec::new(),
        );

        let next = select_next(&mut session, None, &mut rng).unwrap();

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, Vec::new());

        assert!(session.history.is_empty());

        assert_eq!(session.speaker, None);

        assert_eq!(next, SpeakerSelectionOutput::ContinueWith { update: None });
    }

    /// Test next when selection_strategy is Nomination but no nomination was given but the
    /// allow_list contains possible participants
    #[test_log::test(tokio::test)]
    async fn nomination_without_nomination_with_allow_list() {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        let mut session = Session::new(
            p1,
            Parameter {
                selection_strategy: SelectionStrategy::Nomination,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            vec![p1, p2, p3],
        );

        // select_next with empty history
        let next = select_next(&mut session, None, &mut rng).unwrap();
        assert_eq!(next, SpeakerSelectionOutput::ContinueWith { update: None });

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, Vec::new());

        assert!(session.history.is_empty());

        assert_eq!(session.speaker, None);

        // Add current speaker
        speaker_selection::select_unchecked(&mut session, Some(p1)).unwrap();

        // select_next with non-empty history
        let update = match select_next(&mut session, None, &mut rng).unwrap() {
            SpeakerSelectionOutput::ContinueWith { update } => update.unwrap(),
            SpeakerSelectionOutput::End => panic!("Expected ContinueWith output, got End"),
        };

        assert!(update.speaker.is_none());
        assert_eq!(update.history.unwrap(), vec![p1]);
        let mut remaining = update.remaining.unwrap();
        remaining.sort();
        assert_eq!(remaining, vec![p1, p2, p3]);

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p1]);

        assert_history_without_timestamp(
            &session.history,
            &[HistoryEntry::start(p1), HistoryEntry::stop(p1)],
        );

        assert_eq!(session.speaker, None);
    }

    /// Test next when selection_strategy is None
    #[test_log::test(tokio::test)]
    async fn select_next_with_none() {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);

        let mut session = Session::new(
            p1,
            Parameter {
                selection_strategy: SelectionStrategy::None,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            Vec::new(),
        );

        let next = select_next(&mut session, None, &mut rng).unwrap();

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![]);

        assert_history_without_timestamp(&session.history, &[]);

        assert_eq!(session.speaker, None);

        assert_eq!(next, SpeakerSelectionOutput::ContinueWith { update: None });

        // Add current speaker
        session.history.push(HistoryEntry::start(p1));
        session.speaker = Some(p1);

        // select_next with non-empty history
        let next = select_next(&mut session, None, &mut rng).unwrap();

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p1]);

        assert_history_without_timestamp(
            &session.history,
            &[HistoryEntry::start(p1), HistoryEntry::stop(p1)],
        );

        assert_eq!(session.speaker, None);

        assert_eq!(
            next,
            SpeakerSelectionOutput::ContinueWith {
                update: Some(SpeakerUpdate {
                    speaker: None,
                    history: Some(vec![p1]),
                    remaining: Some(vec![])
                })
            }
        );
    }

    /// Test next when selection_strategy is Playlist
    #[test_log::test(tokio::test)]
    async fn select_next_with_playlist() {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        let mut session = Session::new(
            p1,
            Parameter {
                selection_strategy: SelectionStrategy::Playlist,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            Vec::new(),
        );

        // Test with empty playlist
        let next = select_next(&mut session, None, &mut rng).unwrap();

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![]);

        assert_history_without_timestamp(&session.history, &[]);

        assert_eq!(session.speaker, None);

        assert_eq!(next, SpeakerSelectionOutput::End);

        // Create playlist
        session.remaining = vec![p2, p1, p3];

        // select_next with empty history and playlist
        let next = select_next(&mut session, None, &mut rng).unwrap();

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p2]);

        assert_history_without_timestamp(&session.history, &[HistoryEntry::start(p2)]);

        assert_eq!(session.speaker, Some(p2));

        assert_eq!(
            next,
            SpeakerSelectionOutput::ContinueWith {
                update: Some(SpeakerUpdate {
                    speaker: Some(p2),
                    history: Some(vec![p2]),
                    remaining: Some(vec![p1, p3])
                })
            }
        );

        // select_next with non-empty history and playlist
        let next = select_next(&mut session, None, &mut rng).unwrap();

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p2, p1]);

        assert_history_without_timestamp(
            &session.history,
            &[
                HistoryEntry::start(p2),
                HistoryEntry::stop(p2),
                HistoryEntry::start(p1),
            ],
        );

        assert_eq!(session.speaker, Some(p1));

        assert_eq!(
            next,
            SpeakerSelectionOutput::ContinueWith {
                update: Some(SpeakerUpdate {
                    speaker: Some(p1),
                    history: Some(vec![p2, p1]),
                    remaining: Some(vec![p3])
                })
            }
        );

        // select_next with non-empty history and playlist, to drain playlist
        let next = select_next(&mut session, None, &mut rng).unwrap();
        assert_eq!(
            next,
            SpeakerSelectionOutput::ContinueWith {
                update: Some(SpeakerUpdate {
                    speaker: Some(p3),
                    history: Some(vec![p2, p1, p3]),
                    remaining: Some(vec![])
                })
            }
        );

        assert_history_without_timestamp(
            &session.history,
            &[
                HistoryEntry::start(p2),
                HistoryEntry::stop(p2),
                HistoryEntry::start(p1),
                HistoryEntry::stop(p1),
                HistoryEntry::start(p3),
            ],
        );

        // select_next with non-empty history and empty-playlist
        let next = select_next(&mut session, None, &mut rng).unwrap();

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![p2, p1, p3]);

        assert_history_without_timestamp(
            &session.history,
            &[
                HistoryEntry::start(p2),
                HistoryEntry::stop(p2),
                HistoryEntry::start(p1),
                HistoryEntry::stop(p1),
                HistoryEntry::start(p3),
                HistoryEntry::stop(p3),
            ],
        );

        assert_eq!(session.speaker, None);

        assert_eq!(next, SpeakerSelectionOutput::End);
    }
}
