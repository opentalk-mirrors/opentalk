// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use opentalk_roomserver_types_transcription::{
    event::TranscriptionEvent,
    state::{TranscriptionState, TranscriptionStatus},
};

use crate::{OpenTalkEvent, Room};

#[derive(Debug)]
pub(super) struct RoomTranscriptionState {
    pub(super) transcription_status: TranscriptionStatus,
}

impl RoomTranscriptionState {
    pub(super) fn new(transcription_state: TranscriptionState) -> Self {
        Self {
            transcription_status: transcription_state.status,
        }
    }

    pub(super) fn handle_websocket_event(
        &mut self,
        event: TranscriptionEvent,
    ) -> Result<Option<OpenTalkEvent>> {
        match &event {
            TranscriptionEvent::Segment(..) => {}
            TranscriptionEvent::StateUpdated { status } => {
                self.transcription_status = status.clone();
            }
            TranscriptionEvent::ServiceCommand { .. } => { /* unreachable */ }
            TranscriptionEvent::Error(transcription_error) => {
                log::warn!("Got transcription error event: {transcription_error:?}");
            }
        }

        Ok(Some(OpenTalkEvent::Transcription(event)))
    }
}

pub struct TranscriptionApi<'a> {
    pub(super) room: &'a mut Room,
}

impl TranscriptionApi<'_> {
    /// Returns the active Transcription status
    #[must_use]
    pub fn transcription_status(&mut self) -> TranscriptionStatus {
        self.room
            .transcription_state
            .as_ref()
            .expect("transcription state must be set")
            .transcription_status
            .clone()
    }
}
