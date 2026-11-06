// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{HashMap, hash_map::Entry};

use chrono::{DateTime, Utc};
use opentalk_roomserver_types::{
    client_parameters::ClientKind, connection_id::ConnectionId, device_id::DeviceId,
    room_parameters::WaitingRoom,
};
use opentalk_types_common::users::DisplayName;
use opentalk_types_signaling::{ParticipantId, Role};

use crate::participant_state::ParticipantState;

/// Information associated with a participant that joined the waiting room.
#[derive(Debug, Clone)]
pub struct LobbyParticipant {
    /// The kind of the participant
    pub kind: ClientKind,

    /// The role that the participant assumes in the meeting.
    pub role: Role,

    /// All connections and their associated devices
    connections: HashMap<ConnectionId, LobbyConnection>,

    /// Whether the participant was accepted to enter the meeting
    pub accepted: bool,

    pub waiting_room_state: Option<WaitingRoomState>,
}

#[derive(Debug, Clone)]
pub struct WaitingRoomState {
    /// The display name of the participant
    pub display_name: DisplayName,

    /// The time when the participant joined the waiting room
    pub joined_at: DateTime<Utc>,
}

impl LobbyParticipant {
    pub fn new(
        kind: ClientKind,
        role: Role,
        connections: HashMap<ConnectionId, LobbyConnection>,
        accepted: bool,
        waiting_room_state: Option<WaitingRoomState>,
    ) -> Self {
        Self {
            kind,
            role,
            connections,
            accepted,
            waiting_room_state,
        }
    }

    /// Inserts a connection into the lobby
    ///
    /// Returns the previous connection registered under `connection_id`, if any.
    pub fn insert_lobby_connection(
        &mut self,
        connection_id: ConnectionId,
        device_id: DeviceId,
    ) -> Option<LobbyConnection> {
        self.connections.insert(
            connection_id,
            LobbyConnection {
                device_id,
                in_waiting_room: false,
            },
        )
    }

    /// Inserts the given connections into the waiting room.
    ///
    /// Existing connections registered under the same id are overwritten and, regardless of their
    /// previous location, end up in the waiting room.
    pub fn extend_waiting_connections(
        &mut self,
        connections: impl Iterator<Item = (ConnectionId, DeviceId)>,
    ) {
        self.connections
            .extend(connections.map(|(connection_id, device_id)| {
                (
                    connection_id,
                    LobbyConnection {
                        device_id,
                        in_waiting_room: true,
                    },
                )
            }));
    }

    /// Returns the ids of all connections of this participant.
    ///
    /// This includes connections in the waiting room **and** the lobby.
    pub fn all_connection_ids(&self) -> impl Iterator<Item = ConnectionId> {
        self.connections.keys().copied()
    }

    /// Returns the ids of the connections that are currently in the waiting room.
    pub fn waiting_room_connection_ids(&self) -> impl Iterator<Item = ConnectionId> {
        self.connections
            .iter()
            .filter_map(|(id, con)| if con.in_waiting_room { Some(*id) } else { None })
    }

    /// Returns the ids of the connections that are currently in the lobby.
    pub fn lobby_connection_ids(&self) -> impl Iterator<Item = ConnectionId> {
        self.connections
            .iter()
            .filter_map(|(id, con)| if con.in_waiting_room { None } else { Some(*id) })
    }

    /// Returns the [`Entry`] for the connection with the specified `connection_id`.
    ///
    /// This addresses connections in the waiting room **and** the lobby.
    pub fn connection_entry(
        &mut self,
        connection_id: ConnectionId,
    ) -> Entry<'_, ConnectionId, LobbyConnection> {
        self.connections.entry(connection_id)
    }

    /// Returns a mutable reference to the connection with the specified `connection_id`.
    ///
    /// This addresses connections in the waiting room **and** the lobby.
    pub fn connection_mut(&mut self, connection_id: ConnectionId) -> Option<&mut LobbyConnection> {
        self.connections.get_mut(&connection_id)
    }

    /// Removes all connections that are in the waiting room and returns them.
    ///
    /// Connections that are only in the lobby are left untouched.
    pub fn drain_waiting_connections(&mut self) -> impl Iterator<Item = (ConnectionId, DeviceId)> {
        self.connections
            .extract_if(|_, con| con.in_waiting_room)
            .map(|(id, con)| {
                let LobbyConnection {
                    device_id,
                    in_waiting_room: _,
                } = con;
                (id, device_id)
            })
    }

    /// Removes the connection with the specified `connection_id`.
    ///
    /// This affects connections in the waiting room **and** the lobby.
    pub fn remove_connection(&mut self, connection_id: ConnectionId) -> Option<LobbyConnection> {
        self.connections.remove(&connection_id)
    }

    /// Returns `true` if the participant has any connection in the lobby or waiting room.
    ///
    /// This accounts for connections in the waiting room **and** the lobby.
    pub fn is_connected(&self) -> bool {
        !self.connections.is_empty()
    }

    /// Returns `true` if the participant has any connection in the lobby or waiting room
    /// **and** is not a service.
    pub fn is_connected_user(&self) -> bool {
        self.is_connected() && !self.kind.is_service()
    }

    pub fn can_enter(
        &self,
        participant_id: ParticipantId,
        all_participants: &HashMap<ParticipantId, ParticipantState>,
        waiting_room: WaitingRoom,
    ) -> bool {
        can_enter_conference(
            participant_id,
            all_participants,
            waiting_room,
            &self.kind,
            self.role,
        )
    }
}

#[derive(Debug, Clone)]
pub struct LobbyConnection {
    pub device_id: DeviceId,
    pub in_waiting_room: bool,
}

/// Determines whether a participant may skip the waiting room and enter the conference directly
/// from the lobby.
///
/// `already_in_conference` must be `true` when the participant already has a connection in the
/// conference (i.e. they were accepted earlier and are reconnecting); such participants are always
/// allowed back in.
pub fn can_enter_conference(
    participant_id: ParticipantId,
    all_participants: &HashMap<ParticipantId, ParticipantState>,
    waiting_room: WaitingRoom,
    kind: &ClientKind,
    role: Role,
) -> bool {
    role.is_moderator()
        || kind.is_service()
        || !waiting_room.applies_to(kind, role)
        || all_participants
            .get(&participant_id)
            .is_some_and(|state| !state.in_waiting_room)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashMap};

    use chrono::DateTime;
    use opentalk_roomserver_types::{
        client_parameters::ClientKind, public_user_profile::PublicUserProfile, room_kind::RoomKind,
        room_parameters::WaitingRoom,
    };
    use opentalk_types_common::{users::DisplayName, utils::ExampleData};
    use opentalk_types_signaling::{ParticipantId, Role};

    use super::*;
    use crate::participant_state::ParticipantState;

    /// Builds an empty participant that the individual tests populate with connections.
    fn participant() -> LobbyParticipant {
        LobbyParticipant {
            kind: ClientKind::Unregistered,
            role: Role::Guest,
            connections: HashMap::new(),
            accepted: false,
            waiting_room_state: None,
        }
    }

    #[test]
    fn connection_ids_are_partitioned_by_location() {
        let mut participant = participant();
        participant.insert_lobby_connection(ConnectionId::from_u128(1), DeviceId::from_u128(1));
        participant.extend_waiting_connections(
            [(ConnectionId::from_u128(2), DeviceId::from_u128(2))].into_iter(),
        );

        assert_eq!(
            BTreeSet::from_iter([ConnectionId::from_u128(1), ConnectionId::from_u128(2)]),
            participant.all_connection_ids().collect()
        );
        assert_eq!(
            BTreeSet::from_iter([ConnectionId::from_u128(1)]),
            participant.lobby_connection_ids().collect()
        );
        assert_eq!(
            BTreeSet::from_iter([ConnectionId::from_u128(2)]),
            participant.waiting_room_connection_ids().collect(),
        );
    }

    #[test]
    fn extend_waiting_connections_moves_existing_lobby_connection_into_waiting_room() {
        let mut participant = participant();
        participant.insert_lobby_connection(ConnectionId::from_u128(1), DeviceId::from_u128(1));

        participant.extend_waiting_connections(
            [(ConnectionId::from_u128(1), DeviceId::from_u128(1))].into_iter(),
        );

        assert!(
            participant
                .connection_mut(ConnectionId::from_u128(1))
                .expect("connection must exist")
                .in_waiting_room
        );
        assert_eq!(
            BTreeSet::from_iter([ConnectionId::from_u128(1)]),
            participant.waiting_room_connection_ids().collect()
        );
        assert!(
            participant
                .lobby_connection_ids()
                .collect::<Vec<_>>()
                .is_empty(),
        );
    }

    fn registered() -> ClientKind {
        ClientKind::Registered {
            profile: PublicUserProfile::example_data(),
        }
    }

    #[test]
    fn moderators_skip_the_waiting_room() {
        // Even with the strictest setting, a moderator may enter the conference directly.
        assert!(can_enter_conference(
            ParticipantId::nil(),
            &HashMap::new(),
            WaitingRoom::ForEveryone,
            &registered(),
            Role::Moderator,
        ));
    }

    #[test]
    fn services_skip_the_waiting_room() {
        assert!(can_enter_conference(
            ParticipantId::nil(),
            &HashMap::new(),
            WaitingRoom::ForEveryone,
            &ClientKind::Recorder {
                room: RoomKind::Main
            },
            Role::User,
        ));
    }

    #[test]
    fn entry_allowed_when_the_waiting_room_does_not_apply() {
        // `ForGuests` does not apply to registered users, so they may enter directly.
        assert!(can_enter_conference(
            ParticipantId::nil(),
            &HashMap::new(),
            WaitingRoom::ForGuests,
            &registered(),
            Role::User,
        ));
    }

    #[test]
    fn entry_denied_when_the_waiting_room_applies() {
        // A regular guest with no prior conference presence has to wait.
        assert!(!can_enter_conference(
            ParticipantId::nil(),
            &HashMap::new(),
            WaitingRoom::ForEveryone,
            &ClientKind::Unregistered,
            Role::Guest,
        ));
    }

    #[test]
    fn participants_already_in_the_conference_can_re_enter() {
        let id = ParticipantId::from_u128(1);

        // A participant that is connected and not in the waiting room (i.e. previously accepted)
        // may enter even though the waiting room would otherwise apply.
        let participants = HashMap::from_iter([(
            id,
            ParticipantState::new(
                ClientKind::Unregistered,
                DisplayName::from_str_lossy("Guest"),
                Role::Guest,
                DateTime::UNIX_EPOCH,
                false,
            ),
        )]);
        assert!(can_enter_conference(
            id,
            &participants,
            WaitingRoom::ForEveryone,
            &ClientKind::Unregistered,
            Role::Guest
        ));

        // A participant that is currently in the waiting room is not allowed to skip it.
        let participants = HashMap::from_iter([(
            id,
            ParticipantState::new(
                ClientKind::Unregistered,
                DisplayName::from_str_lossy("Guest"),
                Role::Guest,
                DateTime::UNIX_EPOCH,
                true,
            ),
        )]);
        assert!(!can_enter_conference(
            id,
            &participants,
            WaitingRoom::ForEveryone,
            &ClientKind::Unregistered,
            Role::Guest,
        ));
    }
}
