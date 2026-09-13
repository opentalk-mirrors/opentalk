// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_automod::config::{Parameter, SelectionStrategy};
use rand::{Rng, seq::IndexedRandom};

use crate::{Session, speaker_selection::SpeakerSelectionOutput};

/// Depending on the config will select a random participant to be speaker. This may be used when
/// the `selection_strategy` ist `random` or a moderator issues a `Select::Random` command.
#[tracing::instrument(skip(rng), level = "debug")]
pub fn select_random<R: Rng>(session: &mut Session, rng: &mut R) -> SpeakerSelectionOutput {
    let participant = match &session.parameter {
        Parameter {
            selection_strategy:
                SelectionStrategy::None | SelectionStrategy::Random | SelectionStrategy::Nomination,
            allow_double_selection,
            ..
        } => {
            if *allow_double_selection {
                session.remaining.choose(rng).copied()
            } else {
                let participant_id = session.remaining.choose(rng).copied();
                if let Some(participant_id) = participant_id {
                    session.remaining.retain(|id| *id != participant_id);
                }
                participant_id
            }
        }
        Parameter {
            selection_strategy: SelectionStrategy::Playlist,
            ..
        } => {
            if let Some(participant) = session.remaining.choose(rng).copied() {
                if let Some(index) = session.remaining.iter().position(|id| *id == participant) {
                    session.remaining.remove(index);
                }

                Some(participant)
            } else {
                None
            }
        }
    };

    if participant.is_none() {
        // When determining a random speaker failed, the list of options is empty.
        // This means the session is over.
        return SpeakerSelectionOutput::End;
    }

    let update = super::select_unchecked(session, participant);
    SpeakerSelectionOutput::ContinueWith { update }
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use opentalk_types_signaling::ParticipantId;
    use pretty_assertions::{assert_eq, assert_ne};
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;
    use crate::history_entry::{HistoryEntry, HistoryEntryKind};

    fn rng() -> StdRng {
        StdRng::seed_from_u64(0)
    }

    /// Test that history works when selecting a random member
    /// 3 entries are added, assert that every time select_random returns an entry, it is also
    /// appended to the history.
    #[test_log::test(tokio::test)]
    async fn history_addition() {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        let mut session = Session::new(
            p1,
            Parameter {
                selection_strategy: SelectionStrategy::None,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            vec![p1, p2, p3],
        );

        // === SELECT FIRST
        assert_matches!(
            select_random(&mut session, &mut rng),
            SpeakerSelectionOutput::ContinueWith { .. }
        );

        let history: Vec<ParticipantId> = session.participant_history().collect();
        let first = session.speaker.unwrap();

        assert_eq!(history, vec![first]);

        // === SELECT SECOND
        assert_matches!(
            select_random(&mut session, &mut rng),
            SpeakerSelectionOutput::ContinueWith { .. }
        );

        let second = session.speaker.unwrap();

        assert_ne!(first, second);

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(
            history,
            vec![first, second],
            "History is: {:#?}",
            session.history
        );

        // === SELECT THIRD
        assert_matches!(
            select_random(&mut session, &mut rng),
            SpeakerSelectionOutput::ContinueWith { .. }
        );

        let third = session.speaker.unwrap();

        assert_ne!(first, third);
        assert_ne!(second, third);

        let history: Vec<ParticipantId> = session.participant_history().collect();
        assert_eq!(history, vec![first, second, third]);
    }

    /// Test random selection when selection_strategy is None and double selection is forbidden
    /// 3 entries are added to the allow_list, two entries are added to the history.
    /// Assert that the third entry is returned by select_random
    #[test_log::test(tokio::test)]
    async fn select_random_when_none() {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        let mut session = Session::new(
            p1,
            Parameter {
                selection_strategy: SelectionStrategy::None,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: false,
                auto_append_on_join: false,
            },
            vec![p1, p2, p3],
        );

        assert_matches!(
            select_random(&mut session, &mut rng),
            SpeakerSelectionOutput::ContinueWith { .. }
        );

        let speaker = session.speaker.unwrap();

        assert!([p1, p2, p3].contains(&speaker));
    }

    /// Test random selection when selection_strategy is Playlist
    /// 3 entries are added to the playlist, one entry is added to the history (stopped).
    /// Assert that select_random removes the entries from playlist and adds them to the history.
    #[test_log::test(tokio::test)]
    async fn select_random_when_playlist() {
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
            vec![p1, p2, p3],
        );
        session.history.push(HistoryEntry {
            participant: p1,
            kind: HistoryEntryKind::Start,
        });
        session.history.push(HistoryEntry {
            participant: p1,
            kind: HistoryEntryKind::Stop,
        });

        // === SELECT FIRST
        assert_matches!(
            select_random(&mut session, &mut rng),
            SpeakerSelectionOutput::ContinueWith { .. }
        );

        let speaker = session.speaker.unwrap();

        assert_eq!(speaker, p3);
        assert_eq!(session.remaining, vec![p1, p2]);

        // === SELECT SECOND
        assert_matches!(
            select_random(&mut session, &mut rng),
            SpeakerSelectionOutput::ContinueWith { .. }
        );

        let speaker = session.speaker.unwrap();

        assert_eq!(speaker, p2);
        assert_eq!(session.remaining, vec![p1]);

        // === SELECT THIRD
        assert_matches!(
            select_random(&mut session, &mut rng),
            SpeakerSelectionOutput::ContinueWith { .. }
        );

        let speaker = session.speaker.unwrap();

        assert_eq!(speaker, p1);
        assert_eq!(session.remaining, vec![]);

        // === SELECT LAST MUST BE NONE
        assert_matches!(
            select_random(&mut session, &mut rng),
            SpeakerSelectionOutput::End
        );
    }

    /// Test random selection when selection_strategy is Random and reselection is allowed
    /// 3 entries are added to the allow_list. Select 4 times. Assert that at least once a double
    /// selection was encountered
    #[test_log::test(tokio::test)]
    async fn select_random_when_random_allow_double_select() {
        let mut rng = rng();

        let p1 = ParticipantId::from_u128(1);
        let p2 = ParticipantId::from_u128(2);
        let p3 = ParticipantId::from_u128(3);

        let mut session = Session::new(
            p1,
            Parameter {
                selection_strategy: SelectionStrategy::Random,
                show_remaining: false,
                time_limit: None,
                allow_double_selection: true,
                auto_append_on_join: false,
            },
            vec![p1, p2, p3],
        );
        session.history.push(HistoryEntry {
            participant: p1,
            kind: HistoryEntryKind::Start,
        });
        session.history.push(HistoryEntry {
            participant: p1,
            kind: HistoryEntryKind::Stop,
        });

        // === SELECT FIRST
        let mut selected = Vec::new();

        for _ in 0..4 {
            assert_matches!(
                select_random(&mut session, &mut rng),
                SpeakerSelectionOutput::ContinueWith { .. }
            );

            let speaker = session.speaker.unwrap();

            if selected.contains(&speaker) {
                return;
            } else {
                selected.push(speaker);
            }
        }

        panic!("selected did not contain any duplicates ???")
    }
}
