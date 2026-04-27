// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

use crate::{
    audio::{CHANNELS, SAMPLE_RATE},
    transcription::TranscriptionContext,
};

pub(crate) const SAMPLE_SIZE: usize = 512;
/// The cutoff threshold for the power level
const POWER_LEVEL_THRESHOLD: f32 = 0.02;
/// The probability threshold for the voice activity detection (VAD)
const VAD_PROBABILITY_THRESHOLD: f32 = 0.03;
pub(crate) const POST_BUFFER_LENGTH: Duration = Duration::from_millis(500);
pub(crate) const PRE_BUFFER_LENGTH: Duration = Duration::from_millis(200);
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
    HadActivity(Vec<f32>),
    Silence,
}

pub(crate) async fn detect_speech(transcription: &mut TranscriptionContext) -> bool {
    if transcription.audio_queue.len() < SAMPLE_SIZE {
        return false;
    }

    let mut samples = Vec::with_capacity(512);

    let mut i: usize = 0;
    while let Some(sample) = transcription.audio_queue.pop_front() {
        samples.push(sample);
        i += 1;
        if i == SAMPLE_SIZE {
            break;
        }
    }

    let samples_sum: f32 = samples.iter().map(|s| s * s).sum();
    let power_level = (samples_sum / samples.len() as f32).sqrt();

    log::trace!(
        "Threshold hit: {}, found: {}",
        power_level > POWER_LEVEL_THRESHOLD,
        power_level
    );
    if power_level <= POWER_LEVEL_THRESHOLD {
        return false;
    }

    let vad_probability = transcription
        .voice_activity_detector
        .predict(samples.clone());
    log::trace!("speech probability: {:.02}", vad_probability);
    if vad_probability > VAD_PROBABILITY_THRESHOLD {
        transcription.samples.extend(samples);
        return true;
    } else {
        let start_buffer = samples.len().saturating_sub(MINIMUM_SAMPLES);
        transcription.samples = samples[start_buffer..].to_vec();
    }

    false
}
