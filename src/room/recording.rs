// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use types::{
    common::streaming::StreamingTargetId,
    signaling::recording::{
        StreamStatus, StreamTarget, StreamUpdated,
        command::{PauseStreaming, RecordingCommand, StartStreaming, StopStreaming},
        event::RecordingEvent,
        state::RecordingState,
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

#[derive(Debug)]
pub enum RecorderAction {
    Start,
    Stop,
    Pause,
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateStreamingError {
    #[error("Sending recorder command failed: {0}")]
    SignalingCommandFailed(#[source] anyhow::Error),
}

pub struct RecordingApi<'a> {
    pub(super) room: &'a mut Room,
}

impl RecordingApi<'_> {
    /// Returns all known streams
    ///
    /// # Panics
    /// When the `recording_state` was somehow invalidated
    #[must_use]
    pub fn streams(&self) -> &BTreeMap<StreamingTargetId, StreamTarget> {
        &self.room.recording_state.as_ref().unwrap().streams
    }

    pub async fn update_stream(
        &mut self,
        action: RecorderAction,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<(), UpdateStreamingError> {
        let recorder_command = match action {
            RecorderAction::Start => RecordingCommand::StartStream(StartStreaming { target_ids }),
            RecorderAction::Stop => RecordingCommand::StopStream(StopStreaming { target_ids }),
            RecorderAction::Pause => RecordingCommand::PauseStream(PauseStreaming { target_ids }),
        };

        self.room
            .signaling
            .send(recorder_command)
            .await
            .map_err(UpdateStreamingError::SignalingCommandFailed)
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
            RecordingEvent::Error(..) | RecordingEvent::RecorderError(..) => None,
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
