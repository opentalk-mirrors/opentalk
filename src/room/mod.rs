use crate::{
    room::{
        livekit::{OpenTalkLiveKitEvent, RoomLiveKitState},
        recording::RoomRecordingState,
    },
    signaling::{Signaling, incoming},
};
use ::livekit::{
    Room as LiveKitRoom, RoomEvent as LiveKitRoomEvent, RoomOptions as LiveKitRoomOptions,
};
use anyhow::{Context, Result, bail};
use std::{collections::HashMap, future::pending};
use tokio::select;
use types::signaling::{
    Participant as OpenTalkParticipant, ParticipantId,
    control::{
        command::ControlCommand,
        event::{ControlEvent, JoinSuccess, Left},
        state::ControlState,
    },
    livekit::state::LiveKitState,
    moderation::event::ModerationEvent,
    recording::{
        command::{RecordingCommand, SetConsent},
        state::RecordingState,
    },
};

pub mod livekit;
mod recording;

#[derive(Debug)]
pub struct Room {
    use_livekit: bool,
    signaling: Signaling,

    id: ParticipantId,

    waiting_room_state: WaitingRoomState,
    livekit_state: Option<RoomLiveKitState>,
    recording_state: Option<RoomRecordingState>,

    participants: HashMap<ParticipantId, Participant>,
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
    LiveKit(LiveKitRoomEvent),
}

#[derive(Debug)]
pub enum OpenTalkEvent {
    ParticipantJoined(Participant),
    ParticipantUpdated(Participant),
    ParticipantLeft(ParticipantId),

    MovedToWaitingRoom,
    AcceptedFromWaitingRoom,

    LiveKit(OpenTalkLiveKitEvent),
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: ParticipantId,
    pub hand_is_up: bool,
}

impl Room {
    pub(crate) fn new(use_livekit: bool, signaling: Signaling) -> Self {
        Self {
            use_livekit,
            signaling,
            id: ParticipantId::nil(),
            waiting_room_state: WaitingRoomState::None,
            livekit_state: None,
            recording_state: None,
            participants: HashMap::new(),
        }
    }

    pub async fn recv(&mut self) -> Result<Event> {
        match self.waiting_room_state {
            WaitingRoomState::InWaitingRoom => {
                self.wait_until_accepted().await?;
                return Ok(Event::OpenTalk(OpenTalkEvent::AcceptedFromWaitingRoom));
            }
            WaitingRoomState::Accepted => pending().await,
            WaitingRoomState::None => {}
        }

        loop {
            let event = match &mut self.livekit_state {
                Some(livekit_state) => {
                    select! {
                        payload = self.signaling.recv() => self.parse_opentalk_event(payload?)?.map(Event::OpenTalk),
                        Some(room_event) = livekit_state.recv() => Some(Event::LiveKit(room_event)),
                    }
                }
                None => {
                    let payload = self.signaling.recv().await;
                    self.parse_opentalk_event(payload?)?.map(Event::OpenTalk)
                }
            };

            if let Some(event) = event {
                return Ok(event);
            }
        }
    }

    fn parse_opentalk_event(&mut self, payload: incoming::Event) -> Result<Option<OpenTalkEvent>> {
        let event = match payload {
            incoming::Event::Control(ControlEvent::Joined(opentalk_participant)) => {
                let participant = parse_participant(&opentalk_participant)?;
                self.participants
                    .insert(participant.id, participant.clone());

                Some(OpenTalkEvent::ParticipantJoined(participant))
            }
            incoming::Event::Control(ControlEvent::Update(opentalk_participant)) => {
                let participant = parse_participant(&opentalk_participant)?;
                self.participants
                    .insert(participant.id, participant.clone());

                Some(OpenTalkEvent::ParticipantUpdated(participant))
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
            incoming::Event::LiveKit(livekit_event) => {
                if let Some(livekit_state) = &mut self.livekit_state {
                    livekit_state.handle_websocket_event(livekit_event)
                } else {
                    None
                }
            }
            incoming::Event::Recording(recording_event) => {
                if let Some(recording_state) = &mut self.recording_state {
                    recording_state.handle_websocket_event(recording_event)
                } else {
                    None
                }
            }
            _ => None,
        };

        Ok(event)
    }

    pub async fn wait_until_accepted(&mut self) -> Result<()> {
        log::debug!("wait until accepted");
        loop {
            match self.signaling.recv().await? {
                incoming::Event::Moderation(ModerationEvent::Accepted) => {
                    self.waiting_room_state = WaitingRoomState::Accepted;

                    return Ok(());
                }
                event => log::warn!("Received unexpected event {event:?} while in waiting room"),
            }
        }
    }

    pub async fn enter_from_waiting_room(&mut self) -> Result<()> {
        if self.waiting_room_state != WaitingRoomState::Accepted {
            bail!("Not accepted yet")
        }

        self.signaling.send(ControlCommand::EnterRoom).await?;
        self.join().await
    }

    pub(crate) async fn join(&mut self) -> Result<()> {
        let first_message = self.signaling.recv().await;

        match first_message {
            Ok(incoming::Event::Control(ControlEvent::JoinSuccess(join_success))) => {
                self.id = join_success.id;

                let mut participants = HashMap::new();
                for opentalk_participant in &join_success.participants {
                    let participant = parse_participant(opentalk_participant)?;
                    participants.insert(participant.id, participant);
                }

                if self.use_livekit {
                    self.connect_livekit(&join_success).await?;
                }

                // Set or update recording state after joining from waiting room
                if let Ok(Some(recording_state)) = join_success.get_module::<RecordingState>() {
                    if let Some(room_recording_state) = &mut self.recording_state {
                        room_recording_state.update(recording_state);
                    } else {
                        self.recording_state = Some(RoomRecordingState::new(recording_state))
                    }
                } else {
                    self.recording_state = None;
                }

                self.waiting_room_state = WaitingRoomState::None;
            }
            Ok(incoming::Event::Moderation(ModerationEvent::InWaitingRoom)) => {
                self.waiting_room_state = WaitingRoomState::InWaitingRoom;
            }
            message => {
                bail!("received neither JoinSuccess, nor InWaitingRoom, got: {message:?}")
            }
        }

        Ok(())
    }

    async fn connect_livekit(&mut self, join_success: &JoinSuccess) -> Result<()> {
        let state = join_success
            .get_module::<LiveKitState>()?
            .context("no livekit context")?;

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

        self.livekit_state = Some(RoomLiveKitState::new(room, events, state));

        Ok(())
    }

    pub fn is_in_waiting_room(&self) -> bool {
        self.waiting_room_state != WaitingRoomState::None
    }

    pub fn participants(&self) -> &HashMap<ParticipantId, Participant> {
        &self.participants
    }

    /// Returns the livekit room handle if available
    ///
    /// This is always available if `use_livekit` has been set to true
    pub fn livekit_room(&self) -> Option<&LiveKitRoom> {
        self.livekit_state
            .as_ref()
            .map(|livekit_state| &livekit_state.room)
    }

    /// Set the current consent to be recording in any recording or stream
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
            .is_none_or(|s| s.can_publish());

        livekit_state_allows_publish && recording_state_allows_publish
    }

    /// Closes all connection to the room
    pub async fn close(mut self) -> Result<()> {
        if let Some(livekit_state) = self.livekit_state.take() {
            livekit_state
                .room
                .close()
                .await
                .context("Failed to close livekit")?;
        }

        self.signaling
            .close()
            .await
            .context("Failed to close websocket")
    }
}

impl Drop for Room {
    fn drop(&mut self) {
        if let Some(livekit_state) = self.livekit_state.take() {
            tokio::spawn(async move {
                let _ = livekit_state.room.close().await;
            });
        }
    }
}

fn parse_participant(opentalk_participant: &OpenTalkParticipant) -> Result<Participant> {
    let control_state = opentalk_participant
        .get_module::<ControlState>()?
        .context("control state is missing")?;

    Ok(Participant {
        id: opentalk_participant.id,
        hand_is_up: control_state.hand_is_up,
    })
}
