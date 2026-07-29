// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{HashMap, hash_map::Entry},
    future::pending,
};

use anyhow::{Context, Result, bail, ensure};
use opentalk_roomserver_client::api::{
    command::SignalingModuleCommand,
    event::{SignalingEvent, SignalingModuleEvent},
    signaling::SignalingConnection,
};
use opentalk_roomserver_types::{
    breakout::{event::BreakoutEvent, module_data::BreakoutModuleData},
    connection_id::ConnectionId,
    core::{CoreCommand, CoreEvent},
    join::event_info::EventInfo,
    room_kind::RoomKind,
};
use opentalk_roomserver_types_livekit::{Credentials, LiveKitState};
use opentalk_roomserver_types_moderation::event::ModerationEvent;
use opentalk_roomserver_types_raise_hands::{command::RaiseHandsCommand, event::RaiseHandsEvent};
use opentalk_roomserver_types_recording::{
    command::RecordingCommand, event::RecordingEvent, state::RecordingState,
};
use opentalk_roomserver_types_transcription::{
    event::TranscriptionEvent, service::command::TranscriptionServiceCommand,
    state::TranscriptionState,
};
use opentalk_types::{
    common::users::DisplayName,
    signaling::{ModuleData, ParticipantId},
};
use recording_service::RoomRecordingServiceState;

use crate::room::{
    livekit::RoomLiveKitState, recording::RoomRecordingState, transcription::RoomTranscriptionState,
};

mod breakout;
mod livekit;
mod participant;
mod recording;
mod transcription;

mod recording_service;
mod transcription_service;

pub use breakout::{BreakoutApi, BreakoutConfig, BreakoutState};
pub use livekit::{LiveKitApi, OpenTalkLiveKitEvent};
pub use participant::Participant;
pub use recording::{OpenTalkRecordingEvent, RecorderAction, RecordingApi};
pub use recording_service::{OpenTalkRecordingServiceEvent, RecordingServiceApi};
pub use transcription::TranscriptionApi;
pub use transcription_service::TranscriptionServiceApi;

#[derive(Debug)]
pub struct Room {
    pub(crate) signaling: SignalingConnection,

    id: ParticipantId,

    waiting_room_state: WaitingRoomState,
    livekit_state: Option<RoomLiveKitState>,
    recording_state: Option<RoomRecordingState>,
    breakout_state: BreakoutState,
    transcription_state: Option<RoomTranscriptionState>,

    event_info: Option<EventInfo>,

    participants: HashMap<ParticipantId, Participant>,

    recording_service_state: Option<RoomRecordingServiceState>,
}

#[derive(Debug, PartialEq, Eq)]
enum WaitingRoomState {
    InLobby,
    InWaitingRoom,
    Accepted,
    None,
}

pub struct JoinedLobby {
    pub room: Room,
    /// The participant is allowed to join the conference when `true`. Otherwise they have to go trough the waiting
    /// room.
    pub can_enter: bool,
    /// The display name of the participant in the conference.
    pub display_name: Option<DisplayName>,
}

/// Event originating from the OpenTalk signaling
#[derive(Debug)]
pub enum OpenTalkEvent {
    /// The permission to enter the conference changed while this connection was in the lobby.
    EntryPermissionChanged {
        /// `true` when the participant can enter the conference without going through the waiting room.
        can_enter: bool,
    },

    /// The display name changed in the conference while this connection was in the lobby
    DisplayNameAssigned(Option<DisplayName>),

    /// A participant joined the room
    ParticipantJoined(Participant),

    /// A participant's state has been updated
    ///
    /// The event contains the previous and updated state
    ParticipantUpdated {
        previous: Participant,
        updated: Participant,
    },

    /// A participant left the room
    ParticipantLeft(ParticipantId, ConnectionId),

    /// The participant was moved to the waiting room
    MovedToWaitingRoom,

    /// The participant was accepted from the waiting room and may now join the conference with [`Room::enter_room`].
    WaitingRoomAccepted,

    /// All events from the livekit module
    LiveKit(OpenTalkLiveKitEvent),

    /// All Breakout events
    Breakout(BreakoutEvent),

    /// All events from the recording module
    Recording(OpenTalkRecordingEvent),

    /// All events from the recording-service module
    RecordingService(OpenTalkRecordingServiceEvent),

    /// All events from the transcription module
    Transcription(TranscriptionEvent),

    /// All events from the transcription service module
    TranscriptionService(TranscriptionServiceCommand),

    /// The participant was disconnected from the room. The [`Room`] must no longer be used.
    Disconnected(DisconnectReason),
}

#[derive(Debug)]
pub enum DisconnectReason {
    WebsocketClosed,
    Kicked,
    Banned,
    DebriefingStarted,
}

impl Room {
    pub(crate) fn new(signaling: SignalingConnection) -> Self {
        Self {
            signaling,
            id: ParticipantId::nil(),
            waiting_room_state: WaitingRoomState::None,
            livekit_state: None,
            recording_state: None,
            breakout_state: BreakoutState::default(),
            transcription_state: None,

            event_info: None,
            participants: HashMap::new(),

            recording_service_state: None,
        }
    }

    /// Receive the next event from the room
    ///
    /// This function is cancellation safe and can be called in any state the room is in,
    /// except after being disconnected after which it may wait forever or return an error.
    pub async fn recv(&mut self) -> Result<OpenTalkEvent> {
        match self.waiting_room_state {
            WaitingRoomState::InWaitingRoom => {
                self.wait_until_accepted().await?;
                return Ok(OpenTalkEvent::WaitingRoomAccepted);
            }
            WaitingRoomState::Accepted => pending().await,
            WaitingRoomState::None | WaitingRoomState::InLobby => {}
        }

        loop {
            let event = {
                let payload = self.signaling.receive().await?;
                self.parse_opentalk_event(payload)?
            };

            if let Some(event) = event {
                return Ok(event);
            }
        }
    }

    fn parse_opentalk_event(
        &mut self,
        event: Option<SignalingEvent>,
    ) -> Result<Option<OpenTalkEvent>> {
        let Some(event) = event else {
            return Ok(Some(OpenTalkEvent::Disconnected(
                DisconnectReason::WebsocketClosed,
            )));
        };

        match event.payload {
            SignalingModuleEvent::Core(core_event) => match core_event {
                CoreEvent::JoinedLobby { .. } => {
                    bail!("Got unexpected JoinedLobby");
                }
                CoreEvent::JoinSuccess(..) => {
                    bail!("Got unexpected JoinSuccess");
                }
                CoreEvent::JoinBlocked { .. } => {
                    bail!("Got unexpected JoinBlocked");
                }
                CoreEvent::InWaitingRoom => {
                    bail!("Got unexpected InWaitingRoom");
                }
                CoreEvent::ParticipantConnected {
                    participant_id,
                    connection_id,
                    peer_data,
                } => {
                    let participant = match self.participants.entry(participant_id) {
                        Entry::Occupied(mut occupied_entry) => {
                            let participant = occupied_entry.get_mut();
                            participant.connections.insert(connection_id);
                            participant.clone()
                        }
                        Entry::Vacant(vacant_entry) => {
                            let participant =
                                Participant::new(participant_id, [connection_id], &peer_data)?;
                            vacant_entry.insert(participant.clone());
                            participant
                        }
                    };

                    return Ok(Some(OpenTalkEvent::ParticipantJoined(participant)));
                }
                CoreEvent::ParticipantDisconnected {
                    participant_id,
                    connection_id,
                    reason: _,
                } => {
                    if let Entry::Occupied(mut occupied_entry) =
                        self.participants.entry(participant_id)
                    {
                        let participant = occupied_entry.get_mut();

                        participant.connections.remove(&connection_id);

                        if participant.connections.is_empty() {
                            occupied_entry.remove();
                        }
                    }

                    return Ok(Some(OpenTalkEvent::ParticipantLeft(
                        participant_id,
                        connection_id,
                    )));
                }
                CoreEvent::JoinedWaitingRoom { .. } => {}
                CoreEvent::LeftWaitingRoom(..) => {}
                CoreEvent::Closing { .. } => {}
                CoreEvent::Error(..) => {}
                CoreEvent::RoomParametersChanged { .. } => {}
                CoreEvent::StorageQuotaChanged { .. } => {}
            },
            SignalingModuleEvent::Breakout(breakout_event) => {
                return self.handle_breakout_event(breakout_event);
            }
            SignalingModuleEvent::Moderation(moderation_event) => {
                return self.handle_moderation_event(moderation_event);
            }
            SignalingModuleEvent::RaiseHands(event) => match event {
                RaiseHandsEvent::HandRaised { participant } => {
                    if let Some(participant) = self.participants.get_mut(&participant)
                        && !participant.hand_is_up
                    {
                        let previous = participant.clone();
                        participant.hand_is_up = true;
                        return Ok(Some(OpenTalkEvent::ParticipantUpdated {
                            previous,
                            updated: participant.clone(),
                        }));
                    }
                }
                RaiseHandsEvent::HandLowered { participant } => {
                    if let Some(participant) = self.participants.get_mut(&participant)
                        && participant.hand_is_up
                    {
                        let previous = participant.clone();
                        participant.hand_is_up = false;
                        return Ok(Some(OpenTalkEvent::ParticipantUpdated {
                            previous,
                            updated: participant.clone(),
                        }));
                    }
                }
                _ => {}
            },
            SignalingModuleEvent::Recording(event) => match event {
                RecordingEvent::Service { event } => {
                    if let Some(recording_service_state) = &mut self.recording_service_state {
                        return Ok(Some(recording_service_state.handle_websocket_event(event)));
                    }
                }
                event => {
                    if let Some(recording_state) = &mut self.recording_state {
                        return recording_state
                            .handle_websocket_event(&mut self.participants, event);
                    }
                }
            },
            SignalingModuleEvent::Transcription(transcription_event) => match transcription_event {
                TranscriptionEvent::ServiceCommand { command } => {
                    return Ok(Some(OpenTalkEvent::TranscriptionService(command)));
                }
                event => {
                    if let Some(transcription_state) = &mut self.transcription_state {
                        return transcription_state.handle_websocket_event(event);
                    }
                }
            },
            _ => {}
        };

        Ok(None)
    }

    fn handle_moderation_event(
        &mut self,
        moderation_event: ModerationEvent,
    ) -> Result<Option<OpenTalkEvent>> {
        match moderation_event {
            ModerationEvent::EntryPermissionChanged { can_enter } => {
                Ok(Some(OpenTalkEvent::EntryPermissionChanged { can_enter }))
            }
            ModerationEvent::DisplayNameAssigned { new_name } => {
                Ok(Some(OpenTalkEvent::DisplayNameAssigned(new_name)))
            }
            ModerationEvent::Kicked { .. } => {
                Ok(Some(OpenTalkEvent::Disconnected(DisconnectReason::Kicked)))
            }
            ModerationEvent::Banned => {
                Ok(Some(OpenTalkEvent::Disconnected(DisconnectReason::Banned)))
            }
            ModerationEvent::DebriefingStarted { .. } => Ok(Some(OpenTalkEvent::Disconnected(
                DisconnectReason::DebriefingStarted,
            ))),
            ModerationEvent::WaitingRoomUpdated { .. } => Ok(None),
            ModerationEvent::SentToWaitingRoom => {
                self.waiting_room_state = WaitingRoomState::InWaitingRoom;
                Ok(Some(OpenTalkEvent::MovedToWaitingRoom))
            }
            ModerationEvent::Accepted => Ok(Some(OpenTalkEvent::WaitingRoomAccepted)),
            ModerationEvent::ParticipantAccepted { .. } => Ok(None),
            ModerationEvent::DisplayNameChanged {
                target,
                issued_by: _,
                old_name: _,
                new_name,
            } => {
                if let Some(participant) = self.participants.get_mut(&target) {
                    let previous = participant.clone();
                    participant.display_name = new_name;

                    return Ok(Some(OpenTalkEvent::ParticipantUpdated {
                        previous,
                        updated: participant.clone(),
                    }));
                }

                Ok(None)
            }
            event @ (ModerationEvent::MicrophoneRestrictionsEnabled { .. }
            | ModerationEvent::MicrophoneRestrictionsDisabled
            | ModerationEvent::Muted { .. }) => {
                if let Some(livekit) = &mut self.livekit_state {
                    return Ok(livekit.handle_moderation_event(event));
                }

                Ok(None)
            }
            // Not implemented yet
            ModerationEvent::RoleUpdated { .. }
            | ModerationEvent::ParticipantBanned(..)
            | ModerationEvent::ParticipantUnbanned { .. }
            | ModerationEvent::Error(..)
            | ModerationEvent::DisplayNameChangeRestrictionsEnabled { .. }
            | ModerationEvent::DisplayNameChangeRestrictionsDisabled => Ok(None),
        }
    }

    pub(crate) fn handle_breakout_event(
        &mut self,
        breakout_event: BreakoutEvent,
    ) -> Result<Option<OpenTalkEvent>> {
        match &breakout_event {
            BreakoutEvent::Started {
                started_by: _,
                rooms,
                expires_at,
                assignment,
            } => {
                self.breakout_state.set_config(BreakoutConfig {
                    rooms: rooms.clone(),
                    expires_at: *expires_at,
                    assignment: *assignment,
                });
            }
            BreakoutEvent::ParticipantSwitchedRoom {
                participant_id,
                old_room: _,
                new_room,
                module_data,
            } => {
                if let Some(participant) = self.participants.get_mut(participant_id) {
                    participant.handle_breakout_switch(*new_room, module_data)?;
                } else {
                    log::error!(
                        "Received breakout switch event for unknown participant {participant_id}"
                    );
                    return Ok(None);
                }
            }
            BreakoutEvent::SwitchedRoom {
                own_data,
                old_room: _,
                new_room,
                peer_data,
            } => {
                self.breakout_state.current_room = *new_room;
                self.handle_module_data(own_data)?;

                for (participant, peer_module_data) in peer_data {
                    if let Some(participant) = self.participants.get_mut(participant) {
                        participant.handle_breakout_switch(*new_room, peer_module_data)?;
                    }
                }
            }
            BreakoutEvent::Closed => {
                self.breakout_state.current_room = RoomKind::Main;
                self.breakout_state.config = None;
            }
            // these events are forwarded to the client but don't require any state changes in the Room struct
            BreakoutEvent::CloseNotice { .. } | BreakoutEvent::Closing { .. } => (),

            BreakoutEvent::Error(breakout_error) => {
                log::error!("Received breakout error event: {breakout_error:?}");
            }
        }

        Ok(Some(OpenTalkEvent::Breakout(breakout_event)))
    }

    /// Wait until accepted into the room, while inside the waiting room
    ///
    /// If the function completes, the [`OpenTalkEvent::WaitingRoomAccepted`] will not be returned by the call to [`recv`](Self::recv).
    pub async fn wait_until_accepted(&mut self) -> Result<()> {
        log::debug!("wait until accepted");
        loop {
            match self
                .signaling
                .receive()
                .await
                .map(|m| m.map(|event| event.payload))?
            {
                Some(SignalingModuleEvent::Moderation(ModerationEvent::Accepted)) => {
                    self.waiting_room_state = WaitingRoomState::Accepted;

                    return Ok(());
                }
                None => bail!("Unexpected end of websocket stream"),
                event => log::warn!("Received unexpected event {event:?} while in waiting room"),
            }
        }
    }

    pub(crate) async fn join_lobby(mut self) -> Result<JoinedLobby> {
        let event = self.recv_event().await?;
        match event {
            SignalingModuleEvent::Core(CoreEvent::JoinedLobby {
                connection_id: _,
                participant_id,
                device_id: _,
                can_enter,
                display_name,
            }) => {
                self.id = participant_id;
                self.waiting_room_state = WaitingRoomState::InLobby;

                Ok(JoinedLobby {
                    room: self,
                    can_enter,
                    display_name,
                })
            }
            SignalingModuleEvent::Core(CoreEvent::JoinBlocked { reason }) => {
                bail!("Join Blocked, {reason:?}");
            }
            message => {
                bail!("Received unexpected event: {message:?}")
            }
        }
    }

    pub async fn enter_waiting_room(&mut self, display_name: Option<DisplayName>) -> Result<()> {
        if self.waiting_room_state != WaitingRoomState::InLobby {
            bail!("Not in lobby");
        }

        self.signaling
            .send(
                &SignalingModuleCommand::Core(CoreCommand::EnterWaitingRoom { display_name })
                    .into(),
            )
            .await
            .context("Failed to send command")?;

        let event = self.recv_event().await?;
        ensure!(
            matches!(event, SignalingModuleEvent::Core(CoreEvent::InWaitingRoom)),
            "Received unexpected event: {event:?}"
        );

        self.waiting_room_state = WaitingRoomState::InWaitingRoom;

        Ok(())
    }

    pub async fn enter_room(&mut self, display_name: Option<DisplayName>) -> Result<()> {
        self.signaling
            .send(&SignalingModuleCommand::Core(CoreCommand::EnterRoom { display_name }).into())
            .await
            .context("Failed to send command")?;

        let event = self.recv_event().await?;
        let SignalingModuleEvent::Core(CoreEvent::JoinSuccess(join_success)) = event else {
            bail!("Received unexpected event: {event:?}");
        };

        self.participants.clear();
        for opentalk_participant in &join_success.participants {
            let participant = Participant::new(
                opentalk_participant.id,
                opentalk_participant
                    .connections
                    .iter()
                    .map(|i| i.connection_id),
                &opentalk_participant.module_data,
            )?;

            self.participants.insert(participant.id, participant);
        }

        self.handle_module_data(&join_success.module_data)?;

        self.event_info = join_success.event_info;
        self.waiting_room_state = WaitingRoomState::None;

        Ok(())
    }

    async fn recv_event(&mut self) -> Result<SignalingModuleEvent> {
        Ok(self
            .signaling
            .receive()
            .await
            .context("Failed to receive message")?
            .context("Unexpected end of websocket stream")?
            .payload)
    }

    fn handle_module_data(&mut self, module_data: &ModuleData) -> Result<()> {
        // livekit
        let state = module_data
            .get::<LiveKitState>()?
            .context("ModuleData does not contain livekit module")?;
        self.livekit_state = Some(RoomLiveKitState::new(state));

        // recording
        if let Some(mut recording_state) = module_data
            .get::<RecordingState>()
            .context("Failed to parse recording module state")?
        {
            if let Some(state) = recording_state.service.take() {
                self.recording_service_state = Some(RoomRecordingServiceState::new(state));
            }

            if let Some(room_recording_state) = &mut self.recording_state {
                room_recording_state.update(recording_state);
            } else {
                self.recording_state = Some(RoomRecordingState::new(recording_state));
            }
        } else {
            self.recording_state = None;
        }

        // transcription
        if let Some(transcription_state) = module_data
            .get::<TranscriptionState>()
            .context("Failed to parse transcription module state")?
        {
            self.transcription_state = Some(RoomTranscriptionState::new(transcription_state));
        } else {
            self.transcription_state = None;
        }

        // breakout
        if let Some(breakout_config) = module_data
            .get::<BreakoutModuleData>()
            .context("Failed to parse breakout module state")?
        {
            self.breakout_state.current_room = breakout_config.room;

            if breakout_config.rooms.is_empty() {
                self.breakout_state.config = None;
            } else {
                self.breakout_state.config = Some(BreakoutConfig {
                    rooms: breakout_config.rooms,
                    expires_at: breakout_config.expires,
                    assignment: None,
                });
            }
        }

        Ok(())
    }

    /// Returns the participant id of the local participant
    pub fn participant_id(&self) -> ParticipantId {
        self.id
    }

    /// Check if the participant is currently inside the waiting room
    pub fn is_in_waiting_room(&self) -> bool {
        self.waiting_room_state != WaitingRoomState::None
    }

    /// Returns a collection of all _other_ participants in the conference
    ///
    /// This is empty when inside the waiting room
    pub fn participants(&self) -> &HashMap<ParticipantId, Participant> {
        &self.participants
    }

    /// Returns a collection of all _other_ participants in the conference, that are active
    ///
    /// # Remarks
    /// This function clones the filtered HashMap
    ///
    /// # Returns
    /// This is empty when no participant is left or when inside the waiting room
    pub fn active_participants(&self) -> HashMap<ParticipantId, Participant> {
        self.participants
            .iter()
            .filter(|(_, p)| p.left_at.is_none())
            .map(|(id, p)| (*id, p.clone()))
            .collect()
    }

    /// Send a command to lower or raise the local participants hand
    pub async fn set_hand_raised(&mut self, raised: bool) -> Result<()> {
        let cmd = if raised {
            RaiseHandsCommand::RaiseHand
        } else {
            RaiseHandsCommand::LowerHand
        };

        self.signaling
            .send(&SignalingModuleCommand::RaiseHands(cmd).into())
            .await
            .context("Failed to send hand-raise/lower websocket message")
    }

    /// Returns the livekit credentials if the Room was joined and the livekit module was present
    ///
    /// Returns `None` when inside the waiting room
    pub fn livekit_credentials(&self) -> Option<&Credentials> {
        self.livekit_state.as_ref().map(|state| &state.credentials)
    }

    /// Set the current consent to be recording in any recording or stream
    ///
    /// # Errors
    /// When the recording state is missing
    /// When the signaling message couldn't be sent
    pub async fn set_recording_consent(&mut self, consent: bool) -> Result<()> {
        let Some(recording_state) = &mut self.recording_state else {
            bail!("no recording state")
        };

        self.signaling
            .send(
                &SignalingModuleCommand::Recording(RecordingCommand::SetConsent { consent }).into(),
            )
            .await
            .context("Failed to send set-consent websocket message")?;

        recording_state.current_consent = consent;

        Ok(())
    }

    /// Returns if publishing the microphone is currently allowed
    pub fn can_publish_microphone(&self) -> bool {
        let livekit_state_allows_publish = self
            .livekit_state
            .as_ref()
            .is_some_and(|s| s.can_publish(self.id));

        let recording_state_allows_publish = self
            .recording_state
            .as_ref()
            .is_none_or(recording::RoomRecordingState::can_publish);

        livekit_state_allows_publish && recording_state_allows_publish
    }

    /// Gets the event title
    ///
    /// Returns `None` if there is no `EventInfo`
    pub fn event_title(&self) -> Option<String> {
        self.event_info
            .as_ref()
            .map(|state| state.title.to_string())
    }

    /// Closes all connection to the room
    pub async fn close(mut self) -> Result<()> {
        self.signaling
            .close()
            .await
            .context("Failed to close signaling connection")
    }

    /// Access the [`LiveKitApi`] which contains functions to interact with the livekit module
    ///
    /// Returns `None` if the livekit module is not present or while inside the waiting room
    pub fn livekit_api(&mut self) -> Option<LiveKitApi<'_>> {
        self.livekit_state
            .is_some()
            .then_some(LiveKitApi { room: self })
    }

    /// Access the [`RecordingApi`] which contains functions to interact with the recording module
    ///
    /// Returns `None` if the recording module is not present or while inside the waiting room
    pub fn recording_api(&mut self) -> Option<RecordingApi<'_>> {
        self.recording_state
            .is_some()
            .then_some(RecordingApi { room: self })
    }

    /// Access the [`BreakoutApi`] which contains functions to interact with the breakout module
    pub fn breakout_api(&mut self) -> BreakoutApi<'_> {
        BreakoutApi { room: self }
    }

    /// Access the [`RecordingServiceApi`] which contains functions to interact with the recording service module
    pub fn recording_service_api(&mut self) -> Option<RecordingServiceApi<'_>> {
        self.recording_service_state
            .is_some()
            .then_some(RecordingServiceApi { room: self })
    }

    /// Access the [`TranscriptionApi`] which contains function to interact with the transcription module
    pub fn transcription_api(&mut self) -> Option<TranscriptionApi<'_>> {
        self.transcription_state
            .is_some()
            .then_some(TranscriptionApi { room: self })
    }

    /// Access the [`TranscriptionServiceApi`] which contains functions to interact with the transcription service module
    pub fn transcription_service_api(&mut self) -> Option<TranscriptionServiceApi<'_>> {
        self.transcription_state
            .is_some()
            .then_some(TranscriptionServiceApi { room: self })
    }
}
