use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use livekit::{Room as LiveKitRoom, RoomEvent as LiveKitEvent, RoomOptions as LiveKitRoomOptions};
use signaling::{Signaling, incoming};
use tokio::{select, sync::mpsc::UnboundedReceiver};
use types_control::{
    command::ControlCommand,
    event::{ControlEvent, JoinSuccess, Left},
    state::ControlState,
};
use types_moderation::event::ModerationEvent;
use types_signaling::{Participant as OpenTalkParticipant, ParticipantId};
use types_signaling_livekit::state::LiveKitState;

pub mod client;
pub mod config;
pub mod http;
pub mod signaling;

pub use client::OpenTalkClient;
pub use config::{AuthConfig, ControllerConfig};

#[derive(Debug)]
pub struct Room {
    use_livekit: bool,
    signaling: Signaling,
    waiting_room_state: WaitingRoomState,
    livekit_room: Option<LiveKitRoom>,
    livekit_events: Option<UnboundedReceiver<LiveKitEvent>>,
    participants: HashMap<ParticipantId, Participant>,
}

#[derive(Debug, PartialEq, Eq)]
enum WaitingRoomState {
    InWaitingRoom,
    Accepted,
    EnterRoomSent,
    None,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Event {
    OpenTalk(OpenTalkEvent),
    LiveKit(LiveKitEvent),
}

#[derive(Debug)]
pub enum OpenTalkEvent {
    ParticipantJoined(Participant),
    ParticipantUpdated(Participant),
    ParticipantLeft(ParticipantId),
    MovedToWaitingRoom,
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: ParticipantId,
    pub hand_is_up: bool,
}

impl Room {
    pub async fn recv(&mut self) -> Result<Event> {
        if self.waiting_room_state == WaitingRoomState::InWaitingRoom {
            self.wait_until_accepted().await?;
        }

        if self.waiting_room_state == WaitingRoomState::Accepted {
            self.signaling.send(ControlCommand::EnterRoom).await?;
            self.waiting_room_state = WaitingRoomState::EnterRoomSent;
        }

        if self.waiting_room_state == WaitingRoomState::EnterRoomSent {
            self.join().await?;
        }

        loop {
            let Some(ref mut livekit_events) = self.livekit_events else {
                let payload = self.signaling.recv().await;
                let Some(event) = self.parse_opentalk_event(payload?)?.map(Event::OpenTalk) else {
                    continue;
                };
                return Ok(event);
            };

            let event = select! {
                payload = self.signaling.recv() => self.parse_opentalk_event(payload?)?.map(Event::OpenTalk),
                Some(room_event) = livekit_events.recv() => Some(Event::LiveKit(room_event)),
            };
            let Some(event) = event else {
                continue;
            };

            return Ok(event);
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
                self.livekit_room = None;
                self.livekit_events = None;

                Some(OpenTalkEvent::MovedToWaitingRoom)
            }
            _ => None,
        };

        Ok(event)
    }

    pub async fn wait_until_accepted(&mut self) -> Result<()> {
        log::debug!("wait until accpeted");
        loop {
            match self.signaling.recv().await? {
                incoming::Event::Moderation(ModerationEvent::Accepted) => {
                    self.waiting_room_state = WaitingRoomState::Accepted;

                    return Ok(());
                }
                _ => {}
            }
        }
    }

    async fn join(&mut self) -> Result<()> {
        let first_message = self.signaling.recv().await;

        match first_message {
            Ok(incoming::Event::Control(ControlEvent::JoinSuccess(join_success))) => {
                let mut participants = HashMap::new();
                for opentalk_participant in &join_success.participants {
                    let participant = parse_participant(opentalk_participant)?;
                    participants.insert(participant.id, participant);
                }

                if self.use_livekit {
                    self.connect_livekit(join_success).await?;
                }
            }
            Ok(incoming::Event::Moderation(ModerationEvent::Accepted)) => {
                self.waiting_room_state = WaitingRoomState::Accepted;
            }
            Ok(incoming::Event::Moderation(ModerationEvent::InWaitingRoom)) => {
                self.waiting_room_state = WaitingRoomState::InWaitingRoom;
            }
            message => {
                bail!("received neither JoinSuccess nor SentToWaitingRoom, got: {message:?}")
            }
        }

        Ok(())
    }

    async fn connect_livekit(&mut self, join_success: JoinSuccess) -> Result<()> {
        let livekit = join_success
            .get_module::<LiveKitState>()?
            .context("no livekit context")?;

        let (livekit_room, livekit_room_events) = LiveKitRoom::connect(
            &livekit
                .credentials
                .service_url
                .unwrap_or(livekit.credentials.public_url),
            &livekit.credentials.token,
            LiveKitRoomOptions::default(),
        )
        .await?;

        self.livekit_room = Some(livekit_room);
        self.livekit_events = Some(livekit_room_events);

        Ok(())
    }

    pub fn is_in_waiting_room(&self) -> bool {
        self.waiting_room_state != WaitingRoomState::None
    }

    pub fn participants(&self) -> Vec<Participant> {
        self.participants.values().cloned().collect::<Vec<_>>()
    }

    pub async fn close(self) {}
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
