// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use types::{
    common::streaming::StreamingTargetId,
    signaling::{
        recording::{StreamStatus, StreamUpdated},
        recording_service::{
            command::RecordingServiceCommand,
            event::RecordingServiceEvent,
            state::{RecorderStreamInfo, RecordingServiceState},
        },
    },
};

use super::{OpenTalkEvent, Room};
use crate::signaling::outgoing;

#[derive(Debug)]
pub enum OpenTalkRecordingServiceEvent {
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
    pub(super) streams: BTreeMap<StreamingTargetId, RecorderStreamInfo>,
}

impl RoomRecordingServiceState {
    pub(super) fn new(state: RecordingServiceState) -> Self {
        Self {
            streams: state.streams,
        }
    }

    pub(super) fn handle_websocket_event(event: RecordingServiceCommand) -> OpenTalkEvent {
        OpenTalkEvent::RecordingService(match event {
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
    pub fn recording(&self) -> Option<(&StreamingTargetId, &RecorderStreamInfo)> {
        self.all_streams()
            .iter()
            .find(|(_, stream_info)| match stream_info {
                RecorderStreamInfo::Recording(_) => true,
                RecorderStreamInfo::Streaming(_) => false,
            })
    }

    pub fn live_streams(&self) -> BTreeMap<&StreamingTargetId, &RecorderStreamInfo> {
        self.all_streams()
            .iter()
            .filter(|(_, stream_info)| match stream_info {
                RecorderStreamInfo::Recording(_) => false,
                RecorderStreamInfo::Streaming(_) => true,
            })
            .collect()
    }

    pub fn all_streams(&self) -> &BTreeMap<StreamingTargetId, RecorderStreamInfo> {
        &self.room.recording_service_state.as_ref().unwrap().streams
    }

    /// Send an update for a stream
    pub async fn send_stream_update(
        &mut self,
        target_id: StreamingTargetId,
        status: StreamStatus,
    ) -> Result<()> {
        self.room
            .signaling
            .send(outgoing::Command::RecordingService(
                RecordingServiceEvent::StreamUpdated(StreamUpdated { target_id, status }),
            ))
            .await
            .context("Failed to send stream update '{status}' on '{target_id}'")
    }
}
