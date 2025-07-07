use std::collections::BTreeSet;

use ::livekit::{Room as LiveKitRoom, RoomEvent as LiveKitRoomEvent};
use tokio::sync::mpsc::UnboundedReceiver;
use types::signaling::{
    ParticipantId,
    livekit::{MicrophoneRestrictionState, event::LiveKitEvent, state::LiveKitState},
};

use crate::room::OpenTalkEvent;

#[derive(Debug)]
pub enum OpenTalkLiveKitEvent {
    MicrophoneRestrictionEnabled {
        unrestricted_participants: BTreeSet<ParticipantId>,
    },
    MicrophoneRestrictionDisabled,
    MutedBy(ParticipantId),
}

#[derive(Debug)]
pub(super) struct RoomLiveKitState {
    pub(super) room: LiveKitRoom,
    events: UnboundedReceiver<LiveKitRoomEvent>,

    // Set of participants that are allowed to talk, if the value is set
    microphone_restriction_state: MicrophoneRestrictionState,
}

impl RoomLiveKitState {
    pub(super) fn new(
        room: LiveKitRoom,
        events: UnboundedReceiver<LiveKitRoomEvent>,
        state: LiveKitState,
    ) -> Self {
        Self {
            room,
            events,
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
        self.events.recv().await
    }
}
