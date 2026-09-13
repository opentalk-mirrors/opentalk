// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_types::{connection_id::ConnectionId, room_kind::RoomKind};
use opentalk_types_signaling::{ParticipantId, ParticipationVisibility};

use crate::participant_state::{ParticipantState, Participants};

/// The filter methods used by [`ParticipantsFiltered`] and [`ParticipantsFilteredMut`]
macro_rules! impl_filter_functions {
    () => {
        /// Filter by connected participants
        pub fn connected(mut self) -> Self {
            self.filter.connected = Some(true);
            self
        }

        /// Filter by disconnected participants
        pub fn disconnected(mut self) -> Self {
            self.filter.connected = Some(false);
            self
        }

        /// Filter by the participants (breakout) room
        pub fn room(mut self, room: RoomKind) -> Self {
            self.filter.room = Some(room);
            self
        }

        pub fn visibility(mut self, visibility: ParticipationVisibility) -> Self {
            self.filter.visibility = Some(visibility);
            self
        }

        pub fn moderators(mut self) -> Self {
            self.filter.moderator_only = Some(true);
            self
        }

        pub fn non_moderators(mut self) -> Self {
            self.filter.moderator_only = Some(false);
            self
        }
    };
}

#[derive(Default, Clone, Copy)]
struct ParticipantStateFilter {
    /// Room participant filter
    room: Option<RoomKind>,
    connected: Option<bool>,
    visibility: Option<ParticipationVisibility>,
    moderator_only: Option<bool>,
}

impl ParticipantStateFilter {
    fn apply(&self, state: &ParticipantState) -> bool {
        if let Some(connected) = self.connected
            && state.is_connected() != connected
        {
            return false;
        }

        if let Some(room) = self.room
            && state.room != room
        {
            return false;
        }

        if let Some(visibility) = self.visibility
            && state.kind.visibility() != visibility
        {
            return false;
        }

        if let Some(only_moderator) = self.moderator_only
            && only_moderator != state.is_moderator()
        {
            return false;
        }

        true
    }
}

/// Holds a mutable reference to [Participants`] and provides various filter functions to access it
pub struct ParticipantsFilteredMut<'a> {
    inner: &'a mut Participants,
    filter: ParticipantStateFilter,
}

impl<'a> ParticipantsFilteredMut<'a> {
    pub(crate) fn new(participants: &'a mut Participants) -> Self {
        Self {
            inner: participants,
            filter: ParticipantStateFilter::default(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ParticipantId, &ParticipantState)> + use<'_> {
        let filter = self.filter;

        self.inner
            .all_unfiltered
            .iter()
            .filter(move |(_, s)| filter.apply(s))
    }

    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&ParticipantId, &mut ParticipantState)> + use<'_> {
        let filter = self.filter;

        self.inner
            .all_unfiltered
            .iter_mut()
            .filter(move |(_, s)| filter.apply(s))
    }

    pub fn get(&self, participant_id: &ParticipantId) -> Option<&ParticipantState> {
        self.inner
            .all_unfiltered
            .get(participant_id)
            .filter(|s| self.filter.apply(s))
    }

    pub fn get_mut(&mut self, participant_id: &ParticipantId) -> Option<&mut ParticipantState> {
        let filter = self.filter;

        self.inner
            .all_unfiltered
            .get_mut(participant_id)
            .filter(|s| filter.apply(s))
    }

    impl_filter_functions!();
}

/// Holds a reference to [Participants`] and provides various filter functions to access it
pub struct ParticipantsFiltered<'a> {
    inner: &'a Participants,
    filter: ParticipantStateFilter,
}

impl<'a> ParticipantsFiltered<'a> {
    pub(crate) fn new(participants: &'a Participants) -> Self {
        Self {
            inner: participants,
            filter: ParticipantStateFilter::default(),
        }
    }

    pub fn ids(self) -> impl Iterator<Item = ParticipantId> {
        let filter = self.filter;

        self.inner
            .all_unfiltered
            .iter()
            .filter(move |(_, s)| filter.apply(s))
            .map(|(k, _)| *k)
    }

    pub fn connection_ids(self) -> impl Iterator<Item = ConnectionId> {
        let filter = self.filter;

        self.inner
            .all_unfiltered
            .iter()
            .filter(move |(_, s)| filter.apply(s))
            .flat_map(|(_, s)| s.connections())
    }

    pub fn iter(self) -> impl Iterator<Item = (&'a ParticipantId, &'a ParticipantState)> + use<'a> {
        let filter = self.filter;

        self.inner
            .all_unfiltered
            .iter()
            .filter(move |(_, s)| filter.apply(s))
    }

    pub fn get(&self, participant_id: &ParticipantId) -> Option<&'a ParticipantState> {
        self.inner
            .all_unfiltered
            .get(participant_id)
            .filter(|s| self.filter.apply(s))
    }

    pub fn contains(&self, participant_id: &ParticipantId) -> bool {
        self.inner
            .all_unfiltered
            .get(participant_id)
            .filter(|s| self.filter.apply(s))
            .is_some()
    }

    impl_filter_functions!();
}
