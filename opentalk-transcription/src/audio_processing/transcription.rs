// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use anyhow::{Context, Result};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use super::voice_activity_detection::SpeechSegment;
use crate::audio::{CHANNELS, SAMPLE_RATE};

pub(crate) struct TranscriptionStage {
    state: WhisperState,
    language: Option<String>,
}

impl TranscriptionStage {
    pub(crate) fn new(context: Arc<WhisperContext>, language: Option<String>) -> Result<Self> {
        let state = context.create_state()?;
        Ok(Self { state, language })
    }

    pub(crate) fn transcribe(&mut self, mut segment: SpeechSegment) -> Result<String> {
        let sample_duration = segment.samples.len() as f32 / SAMPLE_RATE as f32 / CHANNELS as f32;

        if sample_duration < 1.0 {
            segment
                .samples
                .resize((SAMPLE_RATE * CHANNELS * 2) as usize, 0.0);
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(self.language.as_deref());
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        self.state.full(params, &segment.samples[..])?;

        let num_segments = self.state.full_n_segments();
        let mut parts = Vec::new();
        for i in 0..num_segments {
            let seg = self
                .state
                .get_segment(i)
                .with_context(|| format!("Couldn't get segment #{i}"))?;

            let text = seg
                .to_str_lossy()
                .with_context(|| format!("Failed to get text for segment #{i}"))?;

            if !text.is_empty() {
                parts.push(text);
            }
        }

        Ok(parts.join("").trim().to_owned())
    }
}
