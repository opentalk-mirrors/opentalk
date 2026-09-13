// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::HashMap, future::Future, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use ezk::{
    nodes::Access, ConfigRange, Frame, NextEventIsCancelSafe, Source, SourceEvent, ValueRange,
};
use ezk_audio::{
    Channels, Format, RawAudio, RawAudioConfig, RawAudioConfigRange, RawAudioFrame, SampleRate,
    Samples,
};
use ezk_audio_nodes::AudioMixer;
use futures::StreamExt;
use livekit::webrtc::audio_stream::native::NativeAudioStream;
use tokio::{
    sync::Mutex,
    time::{interval, Interval},
};

use crate::Sink;

pub struct Silence {
    timestamp: u64,
    interval: Interval,
}

impl Default for Silence {
    fn default() -> Self {
        Self {
            timestamp: 0,
            interval: interval(Duration::from_millis(20)),
        }
    }
}

pub(crate) const SAMPLE_RATE: u32 = 48_000;
pub(crate) const CHANNELS: u32 = 2;
const FORMAT: Format = Format::I16;

impl Source for Silence {
    type MediaType = RawAudio;

    fn capabilities(&mut self) -> impl Future<Output = ezk::Result<Vec<RawAudioConfigRange>>> {
        std::future::ready(Ok(vec![RawAudioConfigRange {
            sample_rate: ValueRange::Value(SampleRate(SAMPLE_RATE)),
            channels: ValueRange::Value(Channels::NotPositioned(CHANNELS)),
            format: ValueRange::Value(FORMAT),
        }]))
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
        self.interval.tick().await;

        let samples_per_channel = SAMPLE_RATE / (1000 / self.interval.period().as_millis() as u32);

        let event = SourceEvent::Frame(Frame::new(
            RawAudioFrame {
                sample_rate: SampleRate(SAMPLE_RATE),
                channels: Channels::NotPositioned(CHANNELS),
                samples: Samples::equilibrium(FORMAT, (samples_per_channel * CHANNELS) as usize),
            },
            self.timestamp,
        ));

        self.timestamp += u64::from(samples_per_channel);

        Ok(event)
    }
}

impl NextEventIsCancelSafe for Silence {}

pub(crate) struct NativeAudioStreamSource {
    pub(crate) stream: NativeAudioStream,
    pub(crate) timestamp: u64,
}

impl Source for NativeAudioStreamSource {
    type MediaType = RawAudio;

    fn capabilities(&mut self) -> impl Future<Output = ezk::Result<Vec<RawAudioConfigRange>>> {
        std::future::ready(Ok(vec![RawAudioConfigRange {
            sample_rate: ValueRange::Value(SampleRate(SAMPLE_RATE)),
            channels: ValueRange::Value(Channels::NotPositioned(CHANNELS)),
            format: ValueRange::Value(FORMAT),
        }]))
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

pub(crate) async fn audio_mixer_task(
    audio_mixer: Access<AudioMixer>,
    sinks: Arc<Mutex<HashMap<String, Box<dyn Sink>>>>,
) {
    if let Err(e) = audio_mixer_task_inner(audio_mixer, sinks).await {
        log::error!("audio mixer task exited with error={e:?}");
    }
}

async fn audio_mixer_task_inner(
    mut audio_mixer: Access<AudioMixer>,
    sinks: Arc<Mutex<HashMap<String, Box<dyn Sink>>>>,
) -> Result<()> {
    'negotiate: loop {
        audio_mixer
            .negotiate_config(vec![RawAudioConfigRange {
                sample_rate: ValueRange::Value(SampleRate(SAMPLE_RATE)),
                channels: ValueRange::Value(Channels::NotPositioned(CHANNELS)),
                format: ValueRange::Value(FORMAT),
            }])
            .await
            .context("negotiate config for audio mixer failed")?;

        loop {
            let event = audio_mixer
                .next_event()
                .await
                .context("unable to poll next event")?;

            match event {
                SourceEvent::Frame(frame) => {
                    for sink in sinks.lock().await.values_mut() {
                        if let Err(err) = sink.on_audio_frame(frame.clone()).await {
                            log::error!("Unable to push audio: {err:?}");
                        }
                    }
                }
                SourceEvent::EndOfData => {
                    // No more audio sources, this task quits now.
                    return Ok(());
                }
                SourceEvent::RenegotiationNeeded => {
                    continue 'negotiate;
                }
            }
        }
    }
}
