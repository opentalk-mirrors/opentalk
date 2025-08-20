// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use types::signaling::transcription::{segment::Segment, service::TranscriptionServiceCommand};

use super::Room;

pub struct TranscriptionServiceApi<'a> {
    pub(super) room: &'a mut Room,
}

impl<'a> TranscriptionServiceApi<'a> {
    pub async fn send_segment(&mut self, segment: Segment) -> Result<()> {
        self.room
            .signaling
            .send(TranscriptionServiceCommand::Segment(segment))
            .await
            .context("sending transcription service command failed")
    }
}
