// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use opentalk_roomserver_client::api::command::SignalingModuleCommand;
use opentalk_roomserver_types_transcription::{
    command::TranscriptionCommand, segment::TranscriptionSegment,
    service::event::TranscriptionServiceEvent,
};

use crate::Room;

pub struct TranscriptionServiceApi<'a> {
    pub(super) room: &'a mut Room,
}

impl TranscriptionServiceApi<'_> {
    pub async fn send_transcription_started(&mut self) -> Result<()> {
        self.room
            .signaling
            .send(
                &SignalingModuleCommand::Transcription(
                    TranscriptionCommand::TranscriptionServiceEvent {
                        event: TranscriptionServiceEvent::Started,
                    },
                )
                .into(),
            )
            .await
            .context("Failed to send TranscriptionServiceEvent::Started")
    }

    pub async fn send_transcription_segment(
        &mut self,
        segment: TranscriptionSegment,
    ) -> Result<()> {
        self.room
            .signaling
            .send(
                &SignalingModuleCommand::Transcription(
                    TranscriptionCommand::TranscriptionServiceEvent {
                        event: TranscriptionServiceEvent::Segment(segment),
                    },
                )
                .into(),
            )
            .await
            .context("Failed to send TranscriptionServiceEvent::Segment")
    }

    pub async fn send_transcription_stopped(&mut self) -> Result<()> {
        self.room
            .signaling
            .send(
                &SignalingModuleCommand::Transcription(
                    TranscriptionCommand::TranscriptionServiceEvent {
                        event: TranscriptionServiceEvent::Stopped,
                    },
                )
                .into(),
            )
            .await
            .context("Failed to send TranscriptionServiceEvent::Stopped")
    }
}
