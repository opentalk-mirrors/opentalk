// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use opentalk_roomserver_client::api::command::SignalingModuleCommand;
use opentalk_roomserver_types_recording::{
    RecordingStatus, StreamStatus,
    command::RecordingCommand,
    service::{
        command::RecordingServiceCommand,
        event::RecordingServiceEvent,
        state::{RecordingServiceState, ServiceStreamingTarget},
    },
};
use opentalk_types::common::streaming::StreamingTargetId;

use super::{OpenTalkEvent, Room};

#[derive(Debug)]
pub enum OpenTalkRecordingServiceEvent {
    StartRecording,
    PauseRecording,
    StopRecording,

    /// Start Streams
    StartStreams {
        /// The ids of the streams that should be started.
        target_ids: BTreeSet<StreamingTargetId>,
    },
    /// Pause Streams
    PauseStreams {
        /// The ids of the streams that should be paused.
        target_ids: BTreeSet<StreamingTargetId>,
    },
    /// Stop Streams
    StopStreams {
        /// The ids of the streams that should be stopped.
        target_ids: BTreeSet<StreamingTargetId>,
    },
}

#[derive(Debug)]
pub(super) struct RoomRecordingServiceState {
    pub(super) streams: BTreeMap<StreamingTargetId, ServiceStreamingTarget>,
}

impl RoomRecordingServiceState {
    pub(super) fn new(state: RecordingServiceState) -> Self {
        Self {
            streams: state.streaming_targets,
        }
    }

    pub(super) fn handle_websocket_event(&self, event: RecordingServiceCommand) -> OpenTalkEvent {
        OpenTalkEvent::RecordingService(match event {
            RecordingServiceCommand::StartRecording => {
                OpenTalkRecordingServiceEvent::StartRecording
            }
            RecordingServiceCommand::PauseRecording => {
                OpenTalkRecordingServiceEvent::PauseRecording
            }
            RecordingServiceCommand::StopRecording => OpenTalkRecordingServiceEvent::StopRecording,
            RecordingServiceCommand::StartStreams { target_ids } => {
                OpenTalkRecordingServiceEvent::StartStreams { target_ids }
            }
            RecordingServiceCommand::PauseStreams { target_ids } => {
                OpenTalkRecordingServiceEvent::PauseStreams { target_ids }
            }
            RecordingServiceCommand::StopStreams { target_ids } => {
                OpenTalkRecordingServiceEvent::StopStreams { target_ids }
            }
        })
    }
}

pub struct RecordingServiceApi<'a> {
    pub(super) room: &'a mut Room,
}

impl RecordingServiceApi<'_> {
    pub fn streaming_targets(&self) -> &BTreeMap<StreamingTargetId, ServiceStreamingTarget> {
        &self.room.recording_service_state.as_ref().unwrap().streams
    }

    /// Send an update for a stream
    pub async fn send_recording_update(&mut self, status: RecordingStatus) -> Result<()> {
        self.room
            .signaling
            .send(
                &SignalingModuleCommand::Recording(RecordingCommand::Service {
                    command: RecordingServiceEvent::RecordingUpdated(status),
                })
                .into(),
            )
            .await
            .context("Failed to send recording status update")
    }

    /// Send an update for a stream
    pub async fn send_stream_update(
        &mut self,
        target_id: StreamingTargetId,
        status: StreamStatus,
    ) -> Result<()> {
        self.room
            .signaling
            .send(
                &SignalingModuleCommand::Recording(RecordingCommand::Service {
                    command: RecordingServiceEvent::StreamUpdated { status, target_id },
                })
                .into(),
            )
            .await
            .with_context(|| format!("Failed to send stream update on '{target_id}'"))
    }
}
