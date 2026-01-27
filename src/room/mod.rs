// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    any::type_name,
    collections::{BTreeMap, BTreeSet, HashMap, hash_map::Entry},
    future::pending,
};

#[cfg(feature = "livekit")]
use ::livekit::{
    Room as LiveKitRoom, RoomEvent as LiveKitRoomEvent, RoomOptions as LiveKitRoomOptions,
};
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use opentalk_roomserver_client::api::{
    command::SignalingModuleCommand,
    event::{SignalingEvent, SignalingModuleEvent},
    signaling::SignalingConnection,
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    core::{CoreCommand, CoreEvent, state::CoreState},
    join::{event_info::EventInfo, join_success::JoinSuccess},
    shared_json::SharedJson,
};
use opentalk_roomserver_types_livekit::{Credentials, LiveKitState};
use opentalk_roomserver_types_moderation::event::ModerationEvent;
use opentalk_roomserver_types_raise_hands::{
    command::RaiseHandsCommand, event::RaiseHandsEvent, state::RaisedHandPeerState,
};
use opentalk_roomserver_types_recording::{
    command::RecordingCommand, event::RecordingEvent, peer_state::RecordingPeerState,
    state::RecordingState,
};
use serde::Deserialize;
use types::{
    common::{modules::ModuleId, users::DisplayName},
    signaling::{ParticipantId, SignalingModulePeerFrontendData},
};

use crate::room::{livekit::RoomLiveKitState, recording::RoomRecordingState};

mod livekit;
mod recording;

mod recording_service;

pub use livekit::{LiveKitApi, OpenTalkLiveKitEvent};
pub use recording::{OpenTalkRecordingEvent, RecorderAction, RecordingApi};
use recording_service::RoomRecordingServiceState;
pub use recording_service::{OpenTalkRecordingServiceEvent, RecordingServiceApi};

#[derive(Debug)]
pub struct Room {
    pub(crate) signaling: SignalingConnection,

    id: ParticipantId,

    waiting_room_state: WaitingRoomState,
    livekit_state: Option<RoomLiveKitState>,
    recording_state: Option<RoomRecordingState>,

    event_info: Option<EventInfo>,

    participants: HashMap<ParticipantId, Participant>,

    #[cfg(feature = "livekit")]
    use_livekit: bool,
    recording_service_state: Option<RoomRecordingServiceState>,
}

#[derive(Debug, PartialEq, Eq)]
enum WaitingRoomState {
    InWaitingRoom,
    Accepted,
    None,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Event {
    OpenTalk(OpenTalkEvent),
    #[cfg(feature = "livekit")]
    LiveKit(LiveKitRoomEvent),
}

/// Event originating from the OpenTalk signaling
#[derive(Debug)]
pub enum OpenTalkEvent {
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

    /// The participant was accepted from the waiting room and may now join the conference with
    /// [`Room::enter_from_waiting_room`].
    WaitingRoomAccepted,

    /// All events from the livekit module
    LiveKit(OpenTalkLiveKitEvent),

    /// All events from the recording module
    Recording(OpenTalkRecordingEvent),

    /// All events from the recording-service module
    RecordingService(OpenTalkRecordingServiceEvent),

    /// The participant was disconnected from the room. The [`Room`] must no longer be used.
    Disconnected(DisconnectReason),
}

#[derive(Debug)]
pub enum DisconnectReason {
    WebsocketClosed,
    Kicked,
    Banned,
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: ParticipantId,
    pub hand_is_up: bool,
    pub display_name: DisplayName,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub consents_recording: bool,
    pub connections: BTreeSet<ConnectionId>,
}

impl Room {
    pub(crate) fn new(
        signaling: SignalingConnection,
        #[cfg(feature = "livekit")] use_livekit: bool,
    ) -> Self {
        Self {
            signaling,
            id: ParticipantId::nil(),
            waiting_room_state: WaitingRoomState::None,
            livekit_state: None,
            recording_state: None,

            event_info: None,
            participants: HashMap::new(),

            #[cfg(feature = "livekit")]
            use_livekit,
            recording_service_state: None,
        }
    }

    /// Receive the next event from the room
    ///
    /// This function is cancellation safe and can be called in any state the room is in,
    /// except after being disconnected after which it may wait forever or return an error.
    pub async fn recv(&mut self) -> Result<Event> {
        #[cfg(feature = "livekit")]
        async fn recv_livekit(state: &mut Option<RoomLiveKitState>) -> Option<LiveKitRoomEvent> {
            match state {
                Some(state) => state.recv().await,
                None => pending().await,
            }
        }

        match self.waiting_room_state {
            WaitingRoomState::InWaitingRoom => {
                self.wait_until_accepted().await?;
                return Ok(Event::OpenTalk(OpenTalkEvent::WaitingRoomAccepted));
            }
            WaitingRoomState::Accepted => pending().await,
            WaitingRoomState::None => {}
        }

        loop {
            #[cfg(not(feature = "livekit"))]
            let event = {
                let payload = self.signaling.recv().await?;
                self.parse_opentalk_event(payload)?.map(Event::OpenTalk)
            };
            #[cfg(feature = "livekit")]
            let event = tokio::select! {
                payload = self.signaling.receive() => self.parse_opentalk_event(payload?)?.map(Event::OpenTalk),
                room_event = recv_livekit(&mut self.livekit_state) => room_event.map(Event::LiveKit),
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
                CoreEvent::JoinSuccess(..) => {
                    bail!("Got unexpected JoinSuccess");
                }
                CoreEvent::JoinBlocked { .. } => {
                    bail!("Got unexpected JoinBlocked");
                }
                CoreEvent::InWaitingRoom { .. } => {
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
                                parse_participant(participant_id, [connection_id], &peer_data)?;
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
            },
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
            _ => {}
        };

        Ok(None)
    }

    fn handle_moderation_event(
        &mut self,
        moderation_event: ModerationEvent,
    ) -> Result<Option<OpenTalkEvent>> {
        match moderation_event {
            ModerationEvent::Kicked => {
                Ok(Some(OpenTalkEvent::Disconnected(DisconnectReason::Kicked)))
            }
            ModerationEvent::Banned => {
                Ok(Some(OpenTalkEvent::Disconnected(DisconnectReason::Banned)))
            }
            ModerationEvent::ParticipantBanned(..) => Ok(None),
            ModerationEvent::ParticipantUnbanned { .. } => Ok(None),
            ModerationEvent::RoleUpdated { .. } => Ok(None),
            ModerationEvent::DebriefingStarted { .. } => Ok(None),
            ModerationEvent::WaitingRoomEnabled => Ok(None),
            ModerationEvent::WaitingRoomDisabled => Ok(None),
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
            ModerationEvent::Error(..) => Ok(None),
            ModerationEvent::DisplayNameChangeRestrictionsEnabled { .. } => Ok(None),
            ModerationEvent::DisplayNameChangeRestrictionsDisabled => Ok(None),
        }
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

    /// Enter the room from inside the waiting room.
    ///
    /// This function may only be called after receiving the [`OpenTalkEvent::WaitingRoomAccepted`] or completing the
    /// future returned by [`wait_until_accepted`](Self::wait_until_accepted).
    ///
    /// # Errors
    /// When not in waiting room
    /// When the signaling message couldn't be sent
    pub async fn enter_from_waiting_room(&mut self) -> Result<()> {
        if self.waiting_room_state != WaitingRoomState::Accepted {
            bail!("Not in waiting room, or not yet accepted")
        }

        self.signaling
            .send(&SignalingModuleCommand::Core(CoreCommand::EnterRoom).into())
            .await?;

        self.join().await
    }

    pub(crate) async fn join(&mut self) -> Result<()> {
        let first_message = self
            .signaling
            .receive()
            .await
            .context("Failed to receive message")?
            .context("Unexpected end of websocket stream")?;

        match first_message.payload {
            SignalingModuleEvent::Core(CoreEvent::JoinSuccess(join_success)) => {
                self.id = join_success.id;

                self.participants.clear();
                for opentalk_participant in &join_success.participants {
                    let participant = parse_participant(
                        opentalk_participant.id,
                        opentalk_participant
                            .connections
                            .iter()
                            .map(|i| i.connection_id),
                        &opentalk_participant.module_data,
                    )?;

                    self.participants.insert(participant.id, participant);
                }

                self.set_livekit_state(&join_success).await?;

                // Set or update recording state after joining from waiting room
                if let Some(mut recording_state) = join_success
                    .get_module::<RecordingState>()
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

                self.event_info = join_success.event_info;

                self.waiting_room_state = WaitingRoomState::None;
            }
            SignalingModuleEvent::Core(CoreEvent::InWaitingRoom {
                connection_id: _,
                participant_id,
            }) => {
                self.id = participant_id;
                self.waiting_room_state = WaitingRoomState::InWaitingRoom;
            }
            SignalingModuleEvent::Core(CoreEvent::JoinBlocked { reason }) => {
                bail!("Join Blocked, {reason:?}");
            }
            message => {
                bail!("received neither JoinSuccess, nor InWaitingRoom, got: {message:?}")
            }
        }

        Ok(())
    }

    async fn set_livekit_state(&mut self, join_success: &JoinSuccess) -> Result<()> {
        let state = join_success
            .get_module::<LiveKitState>()?
            .context("JoinSuccess does not contain livekit module")?;

        #[cfg(feature = "livekit")]
        {
            let (room, events) = if self.use_livekit {
                let (room, events) = LiveKitRoom::connect(
                    state
                        .credentials
                        .service_url
                        .as_ref()
                        .unwrap_or(&state.credentials.public_url),
                    &state.credentials.token,
                    LiveKitRoomOptions::default(),
                )
                .await?;

                (Some(room), Some(events))
            } else {
                (None, None)
            };

            self.livekit_state = Some(RoomLiveKitState::new(room, events, state));
        }
        #[cfg(not(feature = "livekit"))]
        {
            self.livekit_state = Some(RoomLiveKitState::new(state));
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

    /// Returns the livekit room handle if available
    ///
    /// This is always available if `use_livekit` has been set to true
    ///
    /// Returns `None`` when inside the waiting room
    #[cfg(feature = "livekit")]
    pub fn livekit_room(&self) -> Option<&LiveKitRoom> {
        self.livekit_state
            .as_ref()
            .and_then(|livekit_state| livekit_state.room.as_ref())
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
        #[cfg(feature = "livekit")]
        if let Some(livekit_room) = self
            .livekit_state
            .as_mut()
            .and_then(|state| state.room.take())
        {
            livekit_room
                .close()
                .await
                .context("Failed to close livekit room")?;
        }

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

    /// Access the [`RecordingServiceApi`] which contains functions to interact with the recording service module
    pub fn recording_service_api(&mut self) -> Option<RecordingServiceApi<'_>> {
        self.recording_service_state
            .is_some()
            .then_some(RecordingServiceApi { room: self })
    }
}

impl Drop for Room {
    fn drop(&mut self) {
        #[cfg(feature = "livekit")]
        if let Some(livekit_room) = self
            .livekit_state
            .as_mut()
            .and_then(|state| state.room.take())
        {
            tokio::spawn(async move {
                let _ = livekit_room.close().await;
            });
        }
    }
}

fn parse_participant(
    participant_id: ParticipantId,
    connection_ids: impl IntoIterator<Item = ConnectionId>,
    module_data: &BTreeMap<ModuleId, SharedJson>,
) -> Result<Participant> {
    let core_state: CoreState = get_module(module_data)?.context("Missing core module")?;
    let raise_hands_state: Option<RaisedHandPeerState> = get_module(module_data)?;

    let consents_recording =
        if let Ok(Some(recording_peer_state)) = get_module::<RecordingPeerState>(module_data) {
            recording_peer_state.consents_recording
        } else {
            false
        };

    Ok(Participant {
        id: participant_id,
        hand_is_up: raise_hands_state.is_some(),
        display_name: core_state.display_name,
        joined_at: core_state.joined_at.into(),
        left_at: core_state.left_at.map(Into::into),
        consents_recording,
        connections: BTreeSet::from_iter(connection_ids),
    })
}

fn get_module<T: SignalingModulePeerFrontendData>(
    module_data: &BTreeMap<ModuleId, SharedJson>,
) -> Result<Option<T>> {
    module_data
        .get(&T::NAMESPACE.unwrap())
        .map(parse_module)
        .transpose()
}

fn parse_module<T: for<'a> Deserialize<'a>>(json: &SharedJson) -> Result<T> {
    serde_json::from_value(json.clone_inner()).with_context(|| {
        format!(
            "Failed to parse {} from participant peer data",
            type_name::<T>()
        )
    })
}
