// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use anyhow::Result;

/// Sample rate which is passed into the VAD
const VAD_SAMPLE_RATE: u32 = 16000;

/// Number of channels used by the VAD
const VAD_CHANNELS: usize = 1;

/// Chunk size which is passed into the VAD
const VAD_CHUNK_SIZE: usize = 256;

/// How many chunks to add before and after speech segments
///
/// one chunk in seconds is VAD_CHUNK_SIZE / VAD_SAMPLE_RATE
///
/// 512 / 16000 = 32ms chunk
///
/// 10 chunks then are 320ms padding
const VAD_PADDING_CHUNKS: usize = 10;

/// Confidence threshold of the voice detector
const VAD_DETECTION_THRESHOLD: f32 = 0.5;

/// How many chunks must actually contain speech before a segment is worth transcribing
const VAD_MIN_SPEECH_CHUNKS: usize = 18;

#[derive(Debug, Clone, Copy)]
pub(crate) struct VadConfig {
    pub(crate) sample_rate: u32,
    pub(crate) channels: usize,
    pub(crate) chunk_size: usize,
    pub(crate) padding_chunks: usize,
    pub(crate) detection_threshold: f32,
    pub(crate) min_speech_chunks: usize,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            sample_rate: VAD_SAMPLE_RATE,
            channels: VAD_CHANNELS,
            chunk_size: VAD_CHUNK_SIZE,
            padding_chunks: VAD_PADDING_CHUNKS,
            detection_threshold: VAD_DETECTION_THRESHOLD,
            min_speech_chunks: VAD_MIN_SPEECH_CHUNKS,
        }
    }
}

pub(crate) struct VadStage {
    current: Option<PendingSegment>,
    ready: VecDeque<SpeechSegment>,
    detector: VoiceDetector,
    min_speech_chunks: usize,
}

pub(crate) struct SpeechSegment {
    pub(crate) samples: Vec<f32>,
    pub(crate) started_at: Instant,
}

struct PendingSegment {
    started_at: Instant,
    samples: Vec<f32>,
    speech_chunks: usize,
}

impl PendingSegment {
    fn finish(self, min_speech_chunks: usize) -> Option<SpeechSegment> {
        if self.speech_chunks < min_speech_chunks {
            log::trace!(
                "Discarding segment with only {} speech chunks",
                self.speech_chunks
            );
            return None;
        }

        Some(SpeechSegment {
            samples: self.samples,
            started_at: self.started_at,
        })
    }
}

impl VadStage {
    pub(crate) fn new(config: VadConfig) -> Result<Self> {
        Ok(Self {
            current: None,
            ready: VecDeque::new(),
            detector: VoiceDetector::new(config),
            min_speech_chunks: config.min_speech_chunks,
        })
    }

    pub(crate) fn process(&mut self, samples: Vec<f32>) -> Option<SpeechSegment> {
        let min_speech_chunks = self.min_speech_chunks;

        self.detector
            .handle_samples(Instant::now(), &samples, |chunk| {
                if matches!(chunk.kind, ChunkKind::NotSpeech) {
                    if let Some(segment) = self.current.take()
                        && let Some(segment) = segment.finish(min_speech_chunks)
                    {
                        self.ready.push_back(segment);
                    }

                    return;
                }

                let segment = self.current.get_or_insert_with(|| PendingSegment {
                    started_at: chunk.timestamp,
                    samples: Vec::new(),
                    speech_chunks: 0,
                });

                segment.samples.extend(chunk.samples);
                if matches!(chunk.kind, ChunkKind::Speech) {
                    segment.speech_chunks += 1;
                }
            });

        self.ready.pop_front()
    }
}

/// Wraps VoiceActivityDetector and adds audio padding around detected speech segments
///
/// Extends detected speech segments with padding (audio before/after each segment), preventing abrupt cut-offs
struct VoiceDetector {
    detector: Box<earshot::Detector>,
    threshold: f32,
    max_padding_before_speech: usize,
    max_padding_after_speech: usize,

    /// Queue of samples to be processed
    queue: ReFramer,

    /// Queue of chunks kept in case speech is detected in a later chunk
    /// Always a maximum of `max_padding_chunks`
    before_padding_chunks: VecDeque<(Vec<f32>, Instant)>,

    /// Number of remaining chunks to emit after the speech threshold drops below `max_padding_chunks`
    ///
    /// Is set back to `max_padding_chunks` when speech is detected
    remaining_after_speech_padding: usize,
}

struct ChunkInfo {
    samples: Vec<f32>,
    timestamp: Instant,

    kind: ChunkKind,
}

enum ChunkKind {
    NotSpeech,
    SpeechPadding,
    Speech,
}

impl VoiceDetector {
    fn new(config: VadConfig) -> VoiceDetector {
        VoiceDetector {
            detector: earshot::Detector::default_boxed(),
            threshold: config.detection_threshold,
            max_padding_before_speech: config.padding_chunks,
            max_padding_after_speech: config.padding_chunks,
            queue: ReFramer::new(
                config.channels,
                config.sample_rate as usize,
                Duration::from_secs_f64(config.chunk_size as f64 / config.sample_rate as f64),
            ),
            before_padding_chunks: VecDeque::new(),
            remaining_after_speech_padding: config.padding_chunks,
        }
    }

    /// Takes some samples and tries to detect speech in the audio stream
    ///
    /// `on_chunk` is eventually called with all audio sample passed in the VoiceDetector
    fn handle_samples(
        &mut self,
        timestamp: Instant,
        samples: &[f32],
        mut on_chunk: impl FnMut(ChunkInfo),
    ) {
        self.queue.push(samples, timestamp);

        while let Some((samples, timestamp)) = self.queue.pop() {
            let confidence = self.detector.predict_f32(&samples);

            if confidence >= self.threshold {
                // Speech detected

                self.emit_speech_detected_chunk(&mut on_chunk, samples, timestamp);
            } else {
                // Not speech

                if self.remaining_after_speech_padding > 0 {
                    self.emit_trailing_speech_chunk(&mut on_chunk, samples, timestamp);
                } else {
                    self.queue_non_speech_chunk(&mut on_chunk, samples, timestamp);
                }
            }
        }
    }

    fn emit_speech_detected_chunk(
        &mut self,
        on_chunk: &mut impl FnMut(ChunkInfo),
        samples: Vec<f32>,
        timestamp: Instant,
    ) {
        let before_padding_chunks_len = self.before_padding_chunks.len();

        // If there are before_padding_chunks emit them all with a fade in effect
        for (i, (mut chunk, timestamp)) in self.before_padding_chunks.drain(..).enumerate() {
            effects::fade_in(i, before_padding_chunks_len, &mut chunk);

            on_chunk(ChunkInfo {
                samples: chunk,
                timestamp,
                kind: ChunkKind::SpeechPadding,
            });
        }

        on_chunk(ChunkInfo {
            samples,
            timestamp,
            kind: ChunkKind::Speech,
        });

        self.remaining_after_speech_padding = self.max_padding_after_speech;
    }

    fn emit_trailing_speech_chunk(
        &mut self,
        on_chunk: &mut impl FnMut(ChunkInfo),
        mut samples: Vec<f32>,
        timestamp: Instant,
    ) {
        effects::fade_out(
            self.max_padding_after_speech - self.remaining_after_speech_padding,
            self.max_padding_after_speech,
            &mut samples,
        );

        on_chunk(ChunkInfo {
            samples,
            timestamp,
            kind: ChunkKind::SpeechPadding,
        });

        self.remaining_after_speech_padding -= 1;
    }

    fn queue_non_speech_chunk(
        &mut self,
        on_chunk: &mut impl FnMut(ChunkInfo),
        samples: Vec<f32>,
        timestamp: Instant,
    ) {
        self.before_padding_chunks.push_back((samples, timestamp));

        if self.before_padding_chunks.len() > self.max_padding_before_speech
            && let Some((samples, timestamp)) = self.before_padding_chunks.pop_front()
        {
            // emit dequeued old non-speech chunk
            on_chunk(ChunkInfo {
                samples,
                timestamp,
                kind: ChunkKind::NotSpeech,
            });
        }
    }
}

/// Reframes dynamically sized frames into fixed duration frames while preserving timestamps
struct ReFramer {
    current: Option<(Vec<f32>, Instant)>,
    ready: VecDeque<(Vec<f32>, Instant)>,

    chunk_duration: Duration,

    channels: usize,
    sample_rate: usize,
}

impl ReFramer {
    fn new(channels: usize, sample_rate: usize, chunk_duration: Duration) -> ReFramer {
        ReFramer {
            current: None,
            ready: VecDeque::new(),
            chunk_duration,
            channels,
            sample_rate,
        }
    }

    fn samples_duration(&self, num_samples: usize) -> Duration {
        Duration::from_secs_f64(
            num_samples as f64 / (self.sample_rate as f64 * self.channels as f64),
        )
    }

    fn samples_per_frame(&self) -> usize {
        self.channels
            * (self.sample_rate as f64 * self.chunk_duration.as_secs_f64()).round() as usize
    }

    fn push(&mut self, mut samples: &[f32], mut timestamp: Instant) {
        let num_samples_per_frame = self.samples_per_frame();

        while !samples.is_empty() {
            let Some((mut current_vec, current_ts)) = self.current.take() else {
                self.bite_off_frame(&mut samples, &mut timestamp, num_samples_per_frame);
                continue;
            };

            let expected_next = current_ts + self.samples_duration(current_vec.len());
            let gap = timestamp.saturating_duration_since(expected_next);

            if gap > self.chunk_duration {
                // new samples chunk is further in the future than chunk_duration,
                // emit current chunk with trailing zeroes and start a new one

                current_vec.resize(num_samples_per_frame, 0.0);
                self.ready.push_back((current_vec, current_ts));
            } else {
                let n = num_samples_per_frame - current_vec.len();
                let n = samples.len().min(n);

                current_vec.extend_from_slice(&samples[..n]);

                samples = &samples[n..];

                timestamp += self.samples_duration(n);

                if current_vec.len() == num_samples_per_frame {
                    self.ready.push_back((current_vec, current_ts));
                } else {
                    self.current = Some((current_vec, current_ts));
                }
            }
        }
    }

    // Removes at least one frame's worth of samples from the given `samples` and stores them in `current` or emits them directly
    fn bite_off_frame(
        &mut self,
        samples: &mut &[f32],
        timestamp: &mut Instant,
        num_samples_per_frame: usize,
    ) {
        assert!(self.current.is_none());

        let n = samples.len().min(num_samples_per_frame);

        if n == num_samples_per_frame {
            self.ready.push_back((samples[..n].to_vec(), *timestamp));
        } else {
            self.current = Some((samples[..n].to_vec(), *timestamp));
        }

        *samples = &samples[n..];
        *timestamp += self.samples_duration(n);
    }

    fn pop(&mut self) -> Option<(Vec<f32>, Instant)> {
        self.ready.pop_front()
    }
}

mod effects {
    #![cfg_attr(test, expect(clippy::chunks_exact_to_as_chunks))]

    use std::f32::consts::PI;

    /// Apply a partial fade-in for the n'th chunk in a series of chunks
    pub(super) fn fade_in(nth_chunk: usize, max_chunks: usize, chunk: &mut [f32]) {
        let total_len = chunk.len() * max_chunks;
        let base_index = chunk.len() * nth_chunk;

        let len = total_len as f32;

        for (index, sample) in chunk.iter_mut().enumerate() {
            let index = (base_index + index) as f32;

            let multiplier = (1.0 - (PI * index / len).cos()) / 2.0;

            *sample *= multiplier.clamp(0.0, 1.0);
        }
    }

    /// Apply a partial fade-out for the n'th chunk in a series of chunks
    pub(super) fn fade_out(nth_chunk: usize, max_chunks: usize, chunk: &mut [f32]) {
        let total_len = chunk.len() * max_chunks;
        let base_index = chunk.len() * nth_chunk;

        let len = total_len as f32;

        for (index, sample) in chunk.iter_mut().enumerate() {
            let index = (base_index + index) as f32;

            let multiplier = (1.0 + (PI * index / len).cos()) / 2.0;

            *sample *= multiplier.clamp(0.0, 1.0);
        }
    }

    #[cfg(test)]
    const CHUNK_SIZE: usize = 512;
    #[cfg(test)]
    const NUM_CHUNKS: usize = 12;

    #[cfg(test)]
    #[test]
    fn plot_fade_in() {
        let mut data = vec![1.0f32; CHUNK_SIZE * NUM_CHUNKS];

        for (i, chunk) in data.chunks_exact_mut(CHUNK_SIZE).enumerate() {
            fade_in(i, NUM_CHUNKS, chunk);
        }

        plot_data(data);
    }

    #[cfg(test)]
    #[test]
    fn plot_fade_out() {
        let mut data = vec![1.0f32; CHUNK_SIZE * NUM_CHUNKS];

        for (i, chunk) in data.chunks_exact_mut(CHUNK_SIZE).enumerate() {
            fade_out(i, NUM_CHUNKS, chunk);
        }

        plot_data(data);
    }

    #[cfg(test)]
    fn plot_data(data: Vec<f32>) {
        use textplots::Plot;

        let data: Vec<_> = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as f32 / (CHUNK_SIZE * NUM_CHUNKS) as f32, v))
            .collect();

        textplots::Chart::new(128, 64, 0.0, 1.0)
            .lineplot(&textplots::Shape::Lines(&data))
            .display();
    }
}
