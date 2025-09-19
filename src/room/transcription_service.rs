// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use types::signaling::transcription::{
    segment::Segment,
    service::{TranscriptionServiceCommand, TranscriptionServiceEvent},
};

use super::Room;
use crate::OpenTalkEvent;

#[derive(Debug)]
pub enum OpenTalkTranscriptionServiceEvent {
    /// The transcription should be stopped
    Stop,
}

pub struct TranscriptionServiceApi<'a> {
    pub(super) room: &'a mut Room,
}

impl<'a> TranscriptionServiceApi<'a> {
    /// Send a `Started` command indicating that transcription by the client has started
    ///
    /// This must be sent before sending any `Segment` commands.
    pub async fn send_started(&mut self) -> Result<()> {
        self.room
            .signaling
            .send(TranscriptionServiceCommand::Started)
            .await
            .context("Failed to send Started command")
    }

    /// Send a `Stopped` command indicating that transcription by the client has stopped
    pub async fn send_stopped(&mut self) -> Result<()> {
        self.room
            .signaling
            .send(TranscriptionServiceCommand::Stopped)
            .await
            .context("Failed to send Started command")
    }

    /// Send a `Segment` command containing a transcription segment
    pub async fn send_segment(&mut self, segment: Segment) -> Result<()> {
        self.room
            .signaling
            .send(TranscriptionServiceCommand::Segment(segment))
            .await
            .context("Failed to send Segment command")
    }
}

pub(super) fn map_websocket_event(event: TranscriptionServiceEvent) -> OpenTalkEvent {
    match event {
        TranscriptionServiceEvent::Stop => {
            OpenTalkEvent::TranscriptionService(OpenTalkTranscriptionServiceEvent::Stop)
        }
    }
}
