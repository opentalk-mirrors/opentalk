use std::collections::BTreeMap;
use types::{
    common::streaming::StreamingTargetId,
    signaling::recording::{
        StreamStatus, StreamTarget, StreamUpdated, event::RecordingEvent, state::RecordingState,
    },
};

use crate::{Room, room::OpenTalkEvent};

/// OpenTalk recording module events
#[derive(Debug)]
pub enum OpenTalkRecordingEvent {
    /// Stream targets have been updated. This can be due to a stream starting, pausing or stopping.
    ///
    /// The events provides both the previous and updated state
    StreamTargetsUpdated {
        stream_target: StreamTarget,
        previous_status: StreamStatus,
    },
}

pub struct RecordingApi<'a> {
    pub(super) room: &'a mut Room,
}

impl RecordingApi<'_> {
    pub fn streams(&self) -> &BTreeMap<StreamingTargetId, StreamTarget> {
        &self.room.recording_state.as_ref().unwrap().streams
    }
}

#[derive(Debug)]
pub(super) struct RoomRecordingState {
    pub(super) streams: BTreeMap<StreamingTargetId, StreamTarget>,
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
                let stream_target = self.streams.get_mut(&target_id)?;

                let previous_status = stream_target.status.clone();
                stream_target.status = status;

                Some(OpenTalkEvent::Recording(
                    OpenTalkRecordingEvent::StreamTargetsUpdated {
                        previous_status,
                        stream_target: stream_target.clone(),
                    },
                ))
            }
            RecordingEvent::Error(..) => None,
            RecordingEvent::RecorderError(..) => None,
        }
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
