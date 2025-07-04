use std::collections::BTreeSet;

use livekit::webrtc::audio_source::native::NativeAudioSource;
use ::livekit::{Room as LiveKitRoom, RoomEvent as LiveKitRoomEvent};
use tokio::sync::mpsc::UnboundedReceiver;
use types::signaling::{
    ParticipantId,
    livekit::{MicrophoneRestrictionState, event::LiveKitEvent, state::LiveKitState},
};

use crate::room::OpenTalkEvent;

#[derive(Debug)]
pub(super) struct RoomLiveKitState {
    pub(super) room: LiveKitRoom,
    events: UnboundedReceiver<LiveKitRoomEvent>,

    // Set of participants that are allowed to talk, if the value is set
    unrestricted_participants: Option<BTreeSet<ParticipantId>>,
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
            unrestricted_participants: match state.microphone_restriction_state {
                MicrophoneRestrictionState::Disabled => None,
                MicrophoneRestrictionState::Enabled {
                    unrestricted_participants,
                } => Some(unrestricted_participants),
            },
        }
    }

    pub(super) fn handle_websocket_event(&mut self, event: LiveKitEvent) -> Option<OpenTalkEvent> {
        match event {
            LiveKitEvent::MicrophoneRestrictionsEnabled(unrestricted_participants) => {
                self.unrestricted_participants =
                    Some(unrestricted_participants.unrestricted_participants);
            }
            LiveKitEvent::MicrophoneRestrictionsDisabled => {
                self.unrestricted_participants = None;
            }
            _ => {}
        }

        None
    }

    pub(super) fn can_publish(&self, id: ParticipantId) -> bool {
        self.unrestricted_participants
            .as_ref()
            .is_none_or(|unrestricted| unrestricted.contains(&id))
    }

    pub(super) async fn recv(&mut self) -> Option<LiveKitRoomEvent> {
        self.events.recv().await
    }
}
