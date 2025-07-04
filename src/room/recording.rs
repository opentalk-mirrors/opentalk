use std::collections::BTreeMap;
use types::{
    common::streaming::StreamingTargetId,
    signaling::recording::{
        StreamStatus, StreamTarget, StreamUpdated, event::RecordingEvent, state::RecordingState,
    },
};

use crate::room::OpenTalkEvent;

#[derive(Debug)]
pub(super) struct RoomRecordingState {
    streams: BTreeMap<StreamingTargetId, StreamTarget>,
    pub(super) current_consent: bool,
}

impl RoomRecordingState {
    pub(super) fn new(state: RecordingState) -> Self {
        Self {
            streams: state.targets,
            current_consent: false,
        }
    }

    pub(super) fn handle_websocket_event(
        &mut self,
        event: RecordingEvent,
    ) -> Option<OpenTalkEvent> {
        match event {
            RecordingEvent::StreamUpdated(StreamUpdated { target_id, status }) => {
                let stream = self.streams.get_mut(&target_id)?;
                stream.status = status
            }
            RecordingEvent::Error(..) => {}
            RecordingEvent::RecorderError(..) => {}
        }

        None
    }

    pub(super) fn can_publish(&self) -> bool {
        let no_active_stream = self
            .streams
            .values()
            .all(|s| matches!(s.status, StreamStatus::Inactive));

        self.current_consent || no_active_stream
    }

    // Update the recording state from a join-success state
    pub(super) fn update(&mut self, recording_state: RecordingState) {
        self.streams = recording_state.targets;
    }
}
