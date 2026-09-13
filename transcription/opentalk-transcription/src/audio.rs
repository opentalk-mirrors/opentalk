// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Audio plumbing that adapts a LiveKit native audio stream into an [`ezk`] [`Source`].

use ezk::{ConfigRange, Frame, NextEventIsCancelSafe, Source, SourceEvent, ValueRange};
use ezk_audio::{
    Channels, Format, RawAudio, RawAudioConfig, RawAudioConfigRange, RawAudioFrame, SampleRate,
};
use futures::StreamExt;
use livekit::webrtc::audio_stream::native::NativeAudioStream;

/// Sample rate expected by the transcription pipeline, in Hz.
pub(crate) const SAMPLE_RATE: u32 = 16_000;
/// Number of audio channels expected by the transcription pipeline.
pub(crate) const CHANNELS: u32 = 1;
/// Sample format produced by the LiveKit native audio stream.
pub(crate) const FORMAT: Format = Format::I16;

/// An [`ezk`] [`Source`] that yields frames from a LiveKit [`NativeAudioStream`].
pub(crate) struct NativeAudioStreamSource {
    /// The underlying LiveKit audio stream.
    pub(crate) stream: NativeAudioStream,
    /// Running sample count used as the presentation timestamp of the emitted frames.
    pub(crate) timestamp: u64,
}

impl Source for NativeAudioStreamSource {
    type MediaType = RawAudio;

    async fn capabilities(&mut self) -> ezk::Result<Vec<RawAudioConfigRange>> {
        Ok(vec![RawAudioConfigRange {
            sample_rate: ValueRange::Value(SampleRate(SAMPLE_RATE)),
            channels: ValueRange::Value(Channels::NotPositioned(CHANNELS)),
            format: ValueRange::Value(FORMAT),
        }])
    }

    async fn negotiate_config(
        &mut self,
        available: Vec<RawAudioConfigRange>,
    ) -> ezk::Result<RawAudioConfig> {
        let config = RawAudioConfig {
            sample_rate: SampleRate(SAMPLE_RATE),
            channels: Channels::NotPositioned(CHANNELS),
            format: FORMAT,
        };

        if !available.iter().any(|r| r.contains(&config)) {
            return Err(ezk::Error::negotiation_failed(
                available,
                self.capabilities().await?,
            ));
        }

        Ok(config)
    }

    async fn next_event(&mut self) -> ezk::Result<SourceEvent<Self::MediaType>> {
        let Some(frame) = self.stream.next().await else {
            return Ok(SourceEvent::EndOfData);
        };

        let timestamp = self.timestamp;
        self.timestamp += u64::from(frame.samples_per_channel);

        Ok(SourceEvent::Frame(Frame::new(
            RawAudioFrame {
                sample_rate: SampleRate(frame.sample_rate),
                channels: Channels::NotPositioned(frame.num_channels),
                samples: frame.data.into_owned().into(),
            },
            timestamp,
        )))
    }
}

impl NextEventIsCancelSafe for NativeAudioStreamSource {}
