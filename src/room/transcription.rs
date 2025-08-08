// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::Room;
use anyhow::{Context, Result};
use types::signaling::transcription::{segment::Segment, service::TranscriptionServiceCommand};

pub struct TranscriptionApi<'a> {
    pub(super) room: &'a mut Room,
}

impl<'a> TranscriptionApi<'a> {
    pub async fn send_segment(&mut self, segment: Segment) -> Result<()> {
        self.room
            .signaling
            .send(TranscriptionServiceCommand::Segment(segment))
            .await
            .context("sending transcription service command failed")
    }
}
