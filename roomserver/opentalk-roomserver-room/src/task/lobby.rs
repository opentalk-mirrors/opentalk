// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    mem,
};

use chrono::Utc;
use opentalk_roomserver_signaling::{
    event_origin::EventOrigin,
    lobby_participant::{
        LobbyConnection, LobbyParticipant, WaitingRoomState, can_enter_conference,
    },
};
use opentalk_roomserver_types::{
    client_parameters::ClientParameters,
    connection_id::ConnectionId,
    core::{CORE_MODULE_ID, CoreEvent, LeftWaitingRoom},
    device_id::DeviceId,
    disconnect_reason::DisconnectReason,
    signaling::{module_error::FatalError, websocket::SignalingSocket},
};
use opentalk_types_signaling::{ParticipantId, Role};

use crate::{signaling::DynBroadcastEvent, task::RoomTask};

impl<Socket: SignalingSocket> RoomTask<Socket> {
    #[tracing::instrument(level = "debug", skip(self))]
    pub(super) fn disconnect_lobby_participant(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) {
        let Entry::Occupied(mut participant) = self.lobby_participants.entry(participant_id) else {
            tracing::error!("Attempted to disconnect waiting participant who does not exist");
            return;
        };

        self.message_router.lobby.remove_connection(connection_id);

        let connection = participant.get_mut().remove_connection(connection_id);
        if !participant.get().is_connected() {
            participant.remove();
            tracing::debug!("Lobby connection disconnected, participant removed");
        } else {
            tracing::debug!("Lobby connection disconnected");
        }

        // Only notify moderators in case the participant was in the waiting room.
        if connection.is_some_and(|con| con.in_waiting_room) {
            let moderator_ids = self.participants.connected().moderators().connection_ids();
            let res = self.message_router.conference.serialize_and_send(
                moderator_ids,
                CORE_MODULE_ID,
                None,
                CoreEvent::LeftWaitingRoom(LeftWaitingRoom {
                    id: participant_id,
                    connection_ids: HashSet::from_iter([connection_id]),
                }),
            );
            if let Err(e) = res {
                tracing::warn!("Failed to send disconnect message to moderator: {e:?}");
            }
        }

        self.update_idle_timeout();
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn join_lobby(
        &mut self,
        connection_id: ConnectionId,
        participant_id: ParticipantId,
        device_id: DeviceId,
        client_parameters: ClientParameters,
    ) -> Result<(), FatalError> {
        // The participants role might have changed when they were already in the conference, so
        // determine the role in the following order: conference, lobby, client parameters.
        let role = self
            .participants
            .all_unfiltered
            .get(&participant_id)
            .map(|state| state.role)
            .or_else(|| {
                self.lobby_participants
                    .get(&participant_id)
                    .map(|state| state.role)
            })
            .unwrap_or(client_parameters.role);

        let can_enter = can_enter_conference(
            participant_id,
            &self.participants.all_unfiltered,
            self.info.room.waiting_room,
            &client_parameters.kind,
            role,
        );

        match self.lobby_participants.entry(participant_id) {
            Entry::Occupied(mut occupied_entry) => {
                // The user joins with a second device. The client parameter (e.g. username, role)
                // could have changed since the last connect. If the participant is
                // already connected we ignore the new client parameter.
                let state = occupied_entry.get_mut();
                state.insert_lobby_connection(connection_id, device_id);
            }
            Entry::Vacant(vacant_entry) => {
                let connections = HashMap::from_iter([(
                    connection_id,
                    LobbyConnection {
                        device_id,
                        in_waiting_room: false,
                    },
                )]);
                // The client kind received on the first join is kept for the whole meeting
                let kind = self
                    .participants
                    .all_unfiltered
                    .get(&participant_id)
                    .map(|state| state.kind.clone())
                    .unwrap_or(client_parameters.kind);
                vacant_entry.insert(LobbyParticipant::new(
                    kind,
                    role,
                    connections,
                    can_enter,
                    None,
                ));
            }
        };

        let display_name = Self::derive_display_name(
            self.participants.connected().get(&participant_id),
            self.lobby_participants.get(&participant_id),
        );

        self.message_router.lobby.serialize_and_send(
            [connection_id],
            CORE_MODULE_ID,
            None,
            CoreEvent::JoinedLobby {
                connection_id,
                participant_id,
                device_id,
                can_enter,
                display_name,
            },
        )?;

        tracing::debug!("Participant entered lobby");
        Ok(())
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub fn move_to_waiting_room(
        &mut self,
        participant_id: ParticipantId,
    ) -> Result<(), FatalError> {
        // Moderators that are moved to the waiting room get demoted to user and their ability to
        // skip the waiting room is removed.
        self.revoke_waiting_room_bypass(participant_id)?;

        let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) else {
            tracing::error!("Failed to move unknown participant {participant_id} to waiting room");
            return Ok(());
        };

        let connections = mem::take(&mut state.connections);
        let connection_ids = connections.keys();
        self.message_router
            .move_to_waiting_room(connection_ids.clone().copied());

        match self.lobby_participants.entry(participant_id) {
            Entry::Occupied(mut occupied) => occupied.get_mut().extend_waiting_connections(
                connections
                    .iter()
                    .map(|(connection_id, device_id)| (*connection_id, *device_id)),
            ),
            Entry::Vacant(vacant) => {
                let connections = connections
                    .iter()
                    .map(|(&connection_id, &device_id)| {
                        (
                            connection_id,
                            LobbyConnection {
                                device_id,
                                in_waiting_room: true,
                            },
                        )
                    })
                    .collect();
                vacant.insert(LobbyParticipant::new(
                    state.kind.clone(),
                    state.role,
                    connections,
                    false,
                    Some(WaitingRoomState {
                        display_name: state.display_name.clone(),
                        joined_at: Utc::now(),
                    }),
                ));
            }
        }

        let connection_ids: Vec<ConnectionId> = connection_ids.copied().collect();
        let display_name = state.display_name.clone();
        let avatar_url = state.kind.avatar_url().map(str::to_owned);
        let room = state.room;

        for connection_id in &connection_ids {
            self.participant_disconnected(
                EventOrigin::Internal,
                participant_id,
                *connection_id,
                room,
                DisconnectReason::SentToWaitingRoom,
            )?;
        }

        self.message_router.conference.serialize_and_send(
            self.participants.connected().moderators().connection_ids(),
            CORE_MODULE_ID,
            None,
            CoreEvent::JoinedWaitingRoom {
                participant_id,
                connection_ids,
                joined_at: Utc::now(),
                display_name,
                avatar_url,
            },
        )?;

        tracing::debug!("Participant left lobby and entered the waiting room");
        Ok(())
    }

    /// Removes moderator rights from a participant, assigning them the [`Role::User`], sets their
    /// `in_waiting_room` flag to `true` and their `accepted` flag to `false`.
    #[tracing::instrument(skip(self), level = "debug")]
    fn revoke_waiting_room_bypass(
        &mut self,
        participant_id: ParticipantId,
    ) -> Result<(), FatalError> {
        let mut did_change = false;

        if let Some(state) = self.lobby_participants.get_mut(&participant_id) {
            did_change |= state.accepted;

            state.accepted = false;
            state.role = Role::User;
        }

        let Some(state) = self.participants.all_unfiltered.get_mut(&participant_id) else {
            return Ok(());
        };

        state.in_waiting_room = true;
        did_change |= state.role.is_moderator();
        state.role = Role::User;

        if did_change {
            let room = state.room;
            self.broadcast_event_to_modules(
                EventOrigin::Internal,
                room,
                DynBroadcastEvent::ModerationRoleChanged { participant_id },
            )
            .handle_requested_messages(self)?;
        }

        Ok(())
    }
}
