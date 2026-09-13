// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    mem::replace,
};

use anyhow::{Context, Result};
use opentalk_roomserver_client::api::{command::SignalingModuleCommand, signaling::SignalingError};
use opentalk_roomserver_types_recording::{
    RecordingStatus, StreamStatus, StreamingTarget,
    command::RecordingCommand,
    event::{RecordingError, RecordingEvent},
    state::RecordingState,
};
use opentalk_types::{common::streaming::StreamingTargetId, signaling::ParticipantId};

use crate::{
    Room,
    room::{OpenTalkEvent, participant::Participant},
};

/// OpenTalk recording module events
#[derive(Debug)]
pub enum OpenTalkRecordingEvent {
    /// Recording state has been updated. This can be due to the recording starting, pausing or
    /// stopping.
    ///
    /// The events provides both the previous and updated state
    RecordingStateUpdate {
        previous: RecordingStatus,
        new: RecordingStatus,
    },

    /// Stream targets have been updated. This can be due to a stream starting, pausing or stopping.
    ///
    /// The events provides both the previous and updated state
    StreamTargetsUpdated {
        stream_target: StreamingTargetId,
        previous_status: StreamStatus,
        new_status: StreamStatus,
    },

    /// Some error has occurred
    Error(RecordingError),
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
    SignalingCommandFailed(#[source] SignalingError),
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
    pub fn streams(&self) -> &BTreeMap<StreamingTargetId, StreamingTarget> {
        &self.room.recording_state.as_ref().unwrap().stream_states
    }

    #[must_use]
    pub fn recording_status(&self) -> &RecordingStatus {
        &self.room.recording_state.as_ref().unwrap().recording_state
    }

    pub async fn update_stream(
        &mut self,
        action: RecorderAction,
        target_ids: BTreeSet<StreamingTargetId>,
    ) -> Result<(), UpdateStreamingError> {
        let recorder_command = match action {
            RecorderAction::Start => RecordingCommand::StartStream { target_ids },
            RecorderAction::Stop => RecordingCommand::StopStream { target_ids },
            RecorderAction::Pause => RecordingCommand::PauseStream { target_ids },
        };

        self.room
            .signaling
            .send(&SignalingModuleCommand::Recording(recorder_command).into())
            .await
            .map_err(UpdateStreamingError::SignalingCommandFailed)
    }
}

#[derive(Debug)]
pub(super) struct RoomRecordingState {
    pub(super) recording_state: RecordingStatus,
    pub(super) stream_states: BTreeMap<StreamingTargetId, StreamingTarget>,
    pub(super) current_consent: bool,
}

impl RoomRecordingState {
    pub(super) fn new(state: RecordingState) -> Self {
        Self {
            recording_state: state.recording_state,
            stream_states: state.stream_states,
            current_consent: false,
        }
    }

    pub(super) fn handle_websocket_event(
        &mut self,
        participants: &mut HashMap<ParticipantId, Participant>,
        event: RecordingEvent,
    ) -> Result<Option<OpenTalkEvent>> {
        match event {
            RecordingEvent::RecordingUpdated(status) => {
                let previous = replace(&mut self.recording_state, status.clone());

                Ok(Some(OpenTalkEvent::Recording(
                    OpenTalkRecordingEvent::RecordingStateUpdate {
                        previous,
                        new: status,
                    },
                )))
            }
            RecordingEvent::StreamUpdated { target_id, status } => {
                let stream_target = self
                    .stream_states
                    .get_mut(&target_id)
                    .context("Got stream updated event for unknown streaming target")?;

                let previous_status = stream_target.status.clone();
                stream_target.status = status.clone();

                Ok(Some(OpenTalkEvent::Recording(
                    OpenTalkRecordingEvent::StreamTargetsUpdated {
                        stream_target: target_id,
                        previous_status,
                        new_status: status,
                    },
                )))
            }
            RecordingEvent::ConsentUpdated {
                participant,
                consents,
            } => {
                let participant = participants
                    .get_mut(&participant)
                    .context("Got ConsentUpdated for unknown participant")?;

                let previous = participant.clone();
                participant.consents_recording = consents;

                Ok(Some(OpenTalkEvent::ParticipantUpdated {
                    previous,
                    updated: participant.clone(),
                }))
            }
            RecordingEvent::Service { .. } => {
                // Not passed in
                Ok(None)
            }
            RecordingEvent::Error(err) => Ok(Some(OpenTalkEvent::Recording(
                OpenTalkRecordingEvent::Error(err),
            ))),
        }
    }

    pub(super) fn can_publish(&self) -> bool {
        let no_active_stream = self
            .stream_states
            .values()
            .all(|s| matches!(s.status, StreamStatus::Inactive));

        self.current_consent || no_active_stream
    }

    // Update the recording state from a join-success state
    pub(super) fn update(&mut self, recording_state: RecordingState) {
        self.stream_states = recording_state.stream_states;
        self.recording_state = recording_state.recording_state;
    }
}
