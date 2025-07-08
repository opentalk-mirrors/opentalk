use std::{collections::BTreeSet, future::pending};

use ::livekit::{Room as LiveKitRoom, RoomEvent as LiveKitRoomEvent};
use tokio::sync::mpsc::UnboundedReceiver;
use types::signaling::{
    ParticipantId,
    livekit::{Credentials, MicrophoneRestrictionState, event::LiveKitEvent, state::LiveKitState},
};

use crate::{Room, room::OpenTalkEvent};

#[derive(Debug)]
pub enum OpenTalkLiveKitEvent {
    MicrophoneRestrictionEnabled {
        unrestricted_participants: BTreeSet<ParticipantId>,
    },
    MicrophoneRestrictionDisabled,
    MutedBy(ParticipantId),
}

pub struct LiveKitApi<'a> {
    pub(super) room: &'a mut Room,
}

impl LiveKitApi<'_> {
    fn state(&self) -> &RoomLiveKitState {
        self.room.livekit_state.as_ref().unwrap()
    }

    pub fn credentials(&self) -> &Credentials {
        &self.state().credentials
    }

    pub fn microphone_restriction_state(&self) -> &MicrophoneRestrictionState {
        &self.state().microphone_restriction_state
    }
}

#[derive(Debug)]
pub(super) struct RoomLiveKitState {
    pub(super) room: Option<LiveKitRoom>,
    events: Option<UnboundedReceiver<LiveKitRoomEvent>>,

    pub(super) credentials: Credentials,

    // Set of participants that are allowed to talk, if the value is set
    microphone_restriction_state: MicrophoneRestrictionState,
}

impl RoomLiveKitState {
    pub(super) fn new(
        room: Option<LiveKitRoom>,
        events: Option<UnboundedReceiver<LiveKitRoomEvent>>,
        state: LiveKitState,
    ) -> Self {
        Self {
            room,
            events,
            credentials: state.credentials,
            microphone_restriction_state: state.microphone_restriction_state,
        }
    }

    pub(super) fn handle_websocket_event(&mut self, event: LiveKitEvent) -> Option<OpenTalkEvent> {
        match event {
            LiveKitEvent::MicrophoneRestrictionsEnabled(unrestricted_participants) => {
                self.microphone_restriction_state = MicrophoneRestrictionState::Enabled {
                    unrestricted_participants: unrestricted_participants
                        .unrestricted_participants
                        .clone(),
                };

                Some(OpenTalkEvent::LiveKit(
                    OpenTalkLiveKitEvent::MicrophoneRestrictionEnabled {
                        unrestricted_participants: unrestricted_participants
                            .unrestricted_participants,
                    },
                ))
            }
            LiveKitEvent::MicrophoneRestrictionsDisabled => {
                self.microphone_restriction_state = MicrophoneRestrictionState::Disabled;

                Some(OpenTalkEvent::LiveKit(
                    OpenTalkLiveKitEvent::MicrophoneRestrictionDisabled,
                ))
            }
            LiveKitEvent::ForceMuted { moderator } => Some(OpenTalkEvent::LiveKit(
                OpenTalkLiveKitEvent::MutedBy(moderator),
            )),
            _ => None,
        }
    }

    pub(super) fn can_publish(&self, id: ParticipantId) -> bool {
        match &self.microphone_restriction_state {
            MicrophoneRestrictionState::Disabled => true,
            MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } => unrestricted_participants.contains(&id),
        }
    }

    pub(super) async fn recv(&mut self) -> Option<LiveKitRoomEvent> {
        match &mut self.events {
            Some(events) => events.recv().await,
            None => pending().await,
        }
    }
}
