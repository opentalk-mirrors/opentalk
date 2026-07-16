// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::VecDeque, time::Duration};

use anyhow::Result;
use opentalk_client_signaling::opentalk_types::common::time::Timestamp;
use voice_activity_detector::VoiceActivityDetector;

use crate::audio::{CHANNELS, SAMPLE_RATE};

pub(crate) const SAMPLE_SIZE: usize = 512;
/// The cutoff threshold for the power level
const POWER_LEVEL_THRESHOLD: f32 = 0.02;
/// The probability threshold for the voice activity detection (VAD)
const VAD_PROBABILITY_THRESHOLD: f32 = 0.03;
pub(crate) const POST_BUFFER_LENGTH: Duration = Duration::from_millis(500);
const PRE_BUFFER_LENGTH: Duration = Duration::from_millis(200);
pub(crate) const MINIMUM_SAMPLES: usize =
    sample_block_per_duration(SAMPLE_RATE, CHANNELS, PRE_BUFFER_LENGTH);

pub(crate) const fn sample_block_per_duration(
    sample_rate: u32,
    channels: u32,
    duration: Duration,
) -> usize {
    let seconds = duration.as_secs_f64();
    // length = sample_rate * channels * time
    let total_samples = seconds * sample_rate as f64 * channels as f64;
    total_samples as usize
}

pub(crate) enum SilenceBuffer {
    Silence,
    HadActivity(Vec<f32>),
}

/// The audio samples of a detected speech utterance together with the time it started.
pub(crate) struct SpeechSegment {
    pub(crate) samples: Vec<f32>,
    pub(crate) started_at: Timestamp,
}

pub(crate) struct VadStage {
    audio_queue: VecDeque<f32>,
    voice_activity_detector: VoiceActivityDetector,
    samples: Vec<f32>,
    start_point: Timestamp,
    silence_buffer: SilenceBuffer,
}

impl VadStage {
    pub(crate) fn new() -> Result<Self> {
        let voice_activity_detector = VoiceActivityDetector::builder()
            .sample_rate(SAMPLE_RATE)
            .chunk_size(SAMPLE_SIZE)
            .build()?;

        Ok(Self {
            audio_queue: VecDeque::with_capacity(SAMPLE_SIZE * 2),
            voice_activity_detector,
            samples: (0..MINIMUM_SAMPLES).map(|_| 0.0_f32).collect(),
            start_point: Timestamp::now(),
            silence_buffer: SilenceBuffer::Silence,
        })
    }

    pub(crate) fn process(&mut self, samples: Vec<f32>) -> Option<SpeechSegment> {
        self.audio_queue.extend(&samples);

        let detected = self.detect_speech_chunk();

        if detected {
            self.silence_buffer = SilenceBuffer::HadActivity(Vec::with_capacity(
                sample_block_per_duration(SAMPLE_RATE, CHANNELS, POST_BUFFER_LENGTH),
            ));
        } else if let SilenceBuffer::HadActivity(activity) = &mut self.silence_buffer {
            activity.extend(&samples);
        }

        let SilenceBuffer::HadActivity(silence) = &mut self.silence_buffer else {
            return None;
        };

        if silence.len() < sample_block_per_duration(SAMPLE_RATE, CHANNELS, POST_BUFFER_LENGTH) {
            return None;
        }

        log::debug!(
            "silence buffer exceeded minimum size of: {}",
            sample_block_per_duration(SAMPLE_RATE, CHANNELS, POST_BUFFER_LENGTH)
        );

        let trailing_silence = std::mem::take(silence);
        self.samples.extend(trailing_silence.iter());
        self.silence_buffer = SilenceBuffer::Silence;

        let speech_samples = std::mem::replace(
            &mut self.samples,
            (0..MINIMUM_SAMPLES).map(|_| 0.0).collect(),
        );

        Some(SpeechSegment {
            samples: speech_samples,
            started_at: self.start_point,
        })
    }

    fn detect_speech_chunk(&mut self) -> bool {
        if self.audio_queue.len() < SAMPLE_SIZE {
            return false;
        }

        let mut chunk = Vec::with_capacity(SAMPLE_SIZE);
        for _ in 0..SAMPLE_SIZE {
            if let Some(s) = self.audio_queue.pop_front() {
                chunk.push(s);
            }
        }

        let rms: f32 = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        log::trace!(
            "Threshold hit: {}, found: {}",
            rms > POWER_LEVEL_THRESHOLD,
            rms
        );

        if rms <= POWER_LEVEL_THRESHOLD {
            return false;
        }

        let probability = self.voice_activity_detector.predict(chunk.clone());
        log::trace!("speech probability: {:.02}", probability);

        let is_threshold_met = probability > VAD_PROBABILITY_THRESHOLD;
        if is_threshold_met {
            self.samples.extend(chunk);
        } else {
            let start_buffer = chunk.len().saturating_sub(MINIMUM_SAMPLES);
            self.samples = chunk[start_buffer..].to_vec();
        }

        is_threshold_met
    }
}
