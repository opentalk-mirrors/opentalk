// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use anyhow::{Context, Result};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperState};

use super::voice_activity_detection::SpeechSegment;
use crate::audio::{CHANNELS, SAMPLE_RATE};

#[derive(Debug, Clone)]
pub(crate) struct TranscriptionConfig {
    /// Number of beams explored by the beam search decoder
    ///
    /// Maximum 8
    pub(crate) beam_size: i32,

    /// Temperature the decoder starts at
    pub(crate) temperature: f32,
    /// Temperature increase for fallback passes
    pub(crate) temperature_inc: f32,

    /// Maximum token entropy, before retrying with increased temperature
    ///
    /// Lowering favours quality
    pub(crate) entropy_threshold: f32,

    /// Minimum token log probability, before retrying with increased temperature
    ///
    /// Raising favours quality
    pub(crate) logprob_threshold: f32,

    /// Probability above which whisper's own decoder treats a window as silence
    pub(crate) no_speech_threshold: f32,

    /// Whether to decode every segment with the context of previous segments
    pub(crate) no_context: bool,

    /// Whether to suppress non-speech tokens such as `[Musik]` or `*lacht*`
    pub(crate) suppress_nst: bool,
}

impl Default for TranscriptionConfig {
    fn default() -> Self {
        Self {
            beam_size: 8,
            temperature: 0.0,
            temperature_inc: 0.2,
            entropy_threshold: 2.4,
            logprob_threshold: -1.0,
            no_speech_threshold: 0.6,
            no_context: true,
            suppress_nst: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Transcript {
    pub text: String,
}

pub struct TranscriptionStage {
    config: TranscriptionConfig,
    state: WhisperState,
    language: Option<String>,
}

impl TranscriptionStage {
    pub fn new(context: Arc<WhisperContext>, language: Option<String>) -> Result<Self> {
        let config = TranscriptionConfig::default();
        let state = context.create_state()?;

        Ok(TranscriptionStage {
            config,
            state,
            language,
        })
    }

    pub fn transcribe(&mut self, segment: &mut SpeechSegment) -> Result<Transcript> {
        let sample_duration = segment.samples.len() as f32 / SAMPLE_RATE as f32 / CHANNELS as f32;

        if sample_duration < 1.0 {
            segment
                .samples
                .resize((SAMPLE_RATE * CHANNELS * 2) as usize, 0.0);
        }

        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: self.config.beam_size,
            patience: -1.0,
        });
        params.set_language(self.language.as_deref());
        params.set_temperature(self.config.temperature);
        params.set_temperature_inc(self.config.temperature_inc);
        params.set_entropy_thold(self.config.entropy_threshold);
        params.set_logprob_thold(self.config.logprob_threshold);
        params.set_no_speech_thold(self.config.no_speech_threshold);
        params.set_no_context(self.config.no_context);
        params.set_suppress_blank(true);
        params.set_suppress_nst(self.config.suppress_nst);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        self.state.full(params, &segment.samples[..])?;

        let num_segments = self.state.full_n_segments();
        let mut parts = Vec::new();
        let mut no_speech_prob: f32 = 0.0;

        for i in 0..num_segments {
            let seg = self
                .state
                .get_segment(i)
                .with_context(|| format!("Couldn't get segment #{i}"))?;

            no_speech_prob = no_speech_prob.max(seg.no_speech_probability());

            let text = seg
                .to_str_lossy()
                .with_context(|| format!("Failed to get text for segment #{i}"))?;

            if !text.is_empty() {
                parts.push(text.into_owned());
            }
        }

        Ok(Transcript {
            text: parts.join("").trim().to_owned(),
        })
    }
}
