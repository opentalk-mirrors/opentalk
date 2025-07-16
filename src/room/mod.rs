// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, future::pending};

#[cfg(feature = "livekit")]
use ::livekit::{
    Room as LiveKitRoom, RoomEvent as LiveKitRoomEvent, RoomOptions as LiveKitRoomOptions,
};
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use types::{
    common::{events::EventInfo, users::DisplayName},
    signaling::{
        Participant as OpenTalkParticipant, ParticipantId,
        control::{
            command::ControlCommand,
            event::{ControlEvent, JoinSuccess, Left},
            state::ControlState,
        },
        livekit::{Credentials, state::LiveKitState},
        moderation::event::{ModerationEvent, SessionEnded},
        recording::{
            command::{RecordingCommand, SetConsent},
            peer_state::RecordingPeerState,
            state::RecordingState,
        },
    },
};

use crate::{
    room::{livekit::RoomLiveKitState, recording::RoomRecordingState},
    signaling::{Signaling, incoming},
};

mod livekit;
mod recording;

#[cfg(feature = "chat")]
mod chat;
#[cfg(feature = "recording-service")]
mod recording_service;

#[cfg(feature = "chat")]
use chat::RoomChatState;
#[cfg(feature = "chat")]
pub use chat::{ChatApi, Message, OpenTalkChatEvent};
pub use livekit::{LiveKitApi, OpenTalkLiveKitEvent};
pub use recording::{OpenTalkRecordingEvent, RecordingApi};
#[cfg(feature = "recording-service")]
use recording_service::RoomRecordingServiceState;
#[cfg(feature = "recording-service")]
pub use recording_service::{OpenTalkRecordingServiceEvent, RecordingServiceApi};
#[cfg(feature = "chat")]
use types::signaling::chat::state::ChatState;
#[cfg(feature = "recording-service")]
use types::signaling::recording_service::state::RecordingServiceState;

#[derive(Debug)]
pub struct Room {
    pub(crate) signaling: Signaling,

    id: ParticipantId,

    waiting_room_state: WaitingRoomState,
    livekit_state: Option<RoomLiveKitState>,
    recording_state: Option<RoomRecordingState>,

    event_info: Option<EventInfo>,

    participants: HashMap<ParticipantId, Participant>,

    #[cfg(feature = "chat")]
    chat_state: Option<RoomChatState>,
    #[cfg(feature = "livekit")]
    use_livekit: bool,
    #[cfg(feature = "recording-service")]
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
    ParticipantLeft(ParticipantId),

    /// The participant was moved to the waiting room
    MovedToWaitingRoom,

    /// The participant was accepted from the waiting room and may now join the conference with
    /// [`Room::enter_from_waiting_room`].
    WaitingRoomAccepted,

    /// All events from the livekit module
    LiveKit(OpenTalkLiveKitEvent),

    /// All events from the recording module
    Recording(OpenTalkRecordingEvent),

    #[cfg(feature = "chat")]
    Chat(OpenTalkChatEvent),

    /// All events from the recording-service module
    #[cfg(feature = "recording-service")]
    RecordingService(OpenTalkRecordingServiceEvent),

    /// The participant was disconnected from the room. The [`Room`] must no longer be used.
    Disconnected(DisconnectReason),
}

#[derive(Debug)]
pub enum DisconnectReason {
    WebsocketClosed,
    SessionEnded(SessionEnded),
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: ParticipantId,
    pub hand_is_up: bool,
    pub display_name: DisplayName,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub consents_recording: bool,
}

impl Room {
    pub(crate) fn new(signaling: Signaling, #[cfg(feature = "livekit")] use_livekit: bool) -> Self {
        Self {
            signaling,
            id: ParticipantId::nil(),
            waiting_room_state: WaitingRoomState::None,
            livekit_state: None,
            recording_state: None,

            event_info: None,
            participants: HashMap::new(),

            #[cfg(feature = "chat")]
            chat_state: None,
            #[cfg(feature = "livekit")]
            use_livekit,
            #[cfg(feature = "recording-service")]
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
                payload = self.signaling.recv() => self.parse_opentalk_event(payload?)?.map(Event::OpenTalk),
                room_event = recv_livekit(&mut self.livekit_state) => room_event.map(Event::LiveKit),
            };

            if let Some(event) = event {
                return Ok(event);
            }
        }
    }

    fn parse_opentalk_event(
        &mut self,
        event: Option<incoming::Event>,
    ) -> Result<Option<OpenTalkEvent>> {
        let Some(event) = event else {
            return Ok(Some(OpenTalkEvent::Disconnected(
                DisconnectReason::WebsocketClosed,
            )));
        };

        let event = match event {
            incoming::Event::Control(ControlEvent::Joined(opentalk_participant)) => {
                let participant = parse_participant(&opentalk_participant)?;
                self.participants
                    .insert(participant.id, participant.clone());

                Some(OpenTalkEvent::ParticipantJoined(participant))
            }
            incoming::Event::Control(ControlEvent::Update(opentalk_participant)) => {
                let previous = self.participants[&opentalk_participant.id].clone();

                let participant = parse_participant(&opentalk_participant)?;
                self.participants
                    .insert(participant.id, participant.clone());

                Some(OpenTalkEvent::ParticipantUpdated {
                    previous,
                    updated: participant,
                })
            }
            incoming::Event::Control(ControlEvent::Left(Left {
                id: participant,
                reason: _,
            })) => {
                self.participants.remove(&participant.id);

                Some(OpenTalkEvent::ParticipantLeft(participant.id))
            }
            incoming::Event::Moderation(ModerationEvent::SentToWaitingRoom) => {
                self.waiting_room_state = WaitingRoomState::InWaitingRoom;

                self.livekit_state = None;

                Some(OpenTalkEvent::MovedToWaitingRoom)
            }
            incoming::Event::Moderation(ModerationEvent::SessionEnded(session_ended)) => Some(
                OpenTalkEvent::Disconnected(DisconnectReason::SessionEnded(session_ended)),
            ),
            incoming::Event::LiveKit(livekit_event) => self
                .livekit_state
                .as_mut()
                .and_then(|state| state.handle_websocket_event(livekit_event)),
            incoming::Event::Recording(recording_event) => self
                .recording_state
                .as_mut()
                .and_then(|state| state.handle_websocket_event(recording_event)),
            #[cfg(feature = "chat")]
            incoming::Event::Chat(chat_event) => self
                .chat_state
                .as_mut()
                .and_then(|state| state.handle_websocket_event(chat_event)),
            #[cfg(feature = "recording-service")]
            incoming::Event::RecordingService(recording_service_event) => {
                self.recording_service_state.as_mut().map(|_| {
                    RoomRecordingServiceState::handle_websocket_event(recording_service_event)
                })
            }
            _ => None,
        };

        Ok(event)
    }

    /// Wait until accepted into the room, while inside the waiting room
    ///
    /// If the function completes, the [`OpenTalkEvent::WaitingRoomAccepted`] will not be returned by the call to [`recv`](Self::recv).
    pub async fn wait_until_accepted(&mut self) -> Result<()> {
        log::debug!("wait until accepted");
        loop {
            match self.signaling.recv().await? {
                Some(incoming::Event::Moderation(ModerationEvent::Accepted)) => {
                    self.waiting_room_state = WaitingRoomState::Accepted;

                    return Ok(());
                }
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

        self.signaling.send(ControlCommand::EnterRoom).await?;
        self.join().await
    }

    pub(crate) async fn join(&mut self) -> Result<()> {
        let first_message = self.signaling.recv().await;

        match first_message {
            Ok(Some(incoming::Event::Control(ControlEvent::JoinSuccess(join_success)))) => {
                self.id = join_success.id;

                self.participants.clear();
                for opentalk_participant in &join_success.participants {
                    let participant = parse_participant(opentalk_participant)?;
                    self.participants.insert(participant.id, participant);
                }

                self.set_livekit_state(&join_success).await?;

                // Set or update recording state after joining from waiting room
                if let Ok(Some(recording_state)) = join_success.get_module::<RecordingState>() {
                    if let Some(room_recording_state) = &mut self.recording_state {
                        room_recording_state.update(recording_state);
                    } else {
                        self.recording_state = Some(RoomRecordingState::new(recording_state));
                    }
                } else {
                    self.recording_state = None;
                }

                #[cfg(feature = "chat")]
                {
                    self.chat_state = join_success
                        .get_module::<ChatState>()
                        .context("failed to parse ChatState")?
                        .map(RoomChatState::new);
                }

                #[cfg(feature = "recording-service")]
                {
                    self.recording_service_state = join_success
                        .get_module::<RecordingServiceState>()
                        .context("failed to parse RecordingServiceState")?
                        .map(RoomRecordingServiceState::new);
                }

                self.event_info = join_success.event_info;

                self.waiting_room_state = WaitingRoomState::None;
            }
            Ok(Some(incoming::Event::Moderation(ModerationEvent::InWaitingRoom))) => {
                self.waiting_room_state = WaitingRoomState::InWaitingRoom;
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
            ControlCommand::RaiseHand
        } else {
            ControlCommand::LowerHand
        };

        self.signaling
            .send(cmd)
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
            .send(RecordingCommand::SetConsent(SetConsent { consent }))
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
    pub async fn close(&mut self) -> Result<()> {
        #[cfg(feature = "livekit")]
        if let Some(livekit_room) = self
            .livekit_state
            .as_mut()
            .and_then(|state| state.room.take())
        {
            livekit_room
                .close()
                .await
                .context("Failed to close livekit")?;
        }

        self.signaling
            .close()
            .await
            .context("Failed to close websocket")
    }

    /// Access the [`LiveKitApi`] which contains functions to interact with the livekit module
    ///
    /// Returns `None` if the livekit module is not present or while inside the waiting room
    pub fn livekit_api(&mut self) -> Option<LiveKitApi> {
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

    /// Access the [`ChatApi`] which contains functions to interact with the chat module
    #[cfg(feature = "chat")]
    pub fn chat_api(&mut self) -> Option<ChatApi<'_>> {
        self.chat_state.is_some().then_some(ChatApi { room: self })
    }

    /// Access the [`RecordingServiceApi`] which contains functions to interact with the recording service module
    #[cfg(feature = "recording-service")]
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

fn parse_participant(opentalk_participant: &OpenTalkParticipant) -> Result<Participant> {
    let control_state = opentalk_participant
        .get_module::<ControlState>()?
        .context("control state is missing")?;

    let consents_recording = if let Ok(Some(recording_peer_state)) =
        opentalk_participant.get_module::<RecordingPeerState>()
    {
        recording_peer_state.consents_recording
    } else {
        false
    };

    Ok(Participant {
        id: opentalk_participant.id,
        hand_is_up: control_state.hand_is_up,
        display_name: control_state.display_name,
        joined_at: control_state.joined_at.into(),
        left_at: control_state.left_at.map(Into::into),
        consents_recording,
    })
}
