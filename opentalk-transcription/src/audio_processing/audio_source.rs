// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use audio_nodes::AudioConvert;
use ezk::{Source, SourceEvent};
use ezk_audio::{Format, RawAudioConfigRange, SampleRate};
use livekit::{track::RemoteAudioTrack, webrtc::audio_stream::native::NativeAudioStream};

use crate::audio::{CHANNELS, NativeAudioStreamSource, SAMPLE_RATE};

pub(crate) struct AudioSource {
    pub(crate) audio_convert: AudioConvert<NativeAudioStreamSource>,
}

impl AudioSource {
    pub(crate) async fn new(audio_track: RemoteAudioTrack) -> Result<Self> {
        let mut audio_convert = AudioConvert::new(NativeAudioStreamSource {
            stream: NativeAudioStream::new(
                audio_track.rtc_track(),
                SAMPLE_RATE as i32,
                CHANNELS as i32,
            ),
            timestamp: 0,
        });

        audio_convert
            .negotiate_config(vec![RawAudioConfigRange {
                sample_rate: ezk::ValueRange::Value(SampleRate(SAMPLE_RATE)),
                channels: ezk::ValueRange::Value(ezk_audio::Channels::NotPositioned(CHANNELS)),
                format: ezk::ValueRange::Value(Format::F32),
            }])
            .await?;

        Ok(Self { audio_convert })
    }

    pub(crate) async fn next_samples(&mut self) -> Option<Vec<f32>> {
        loop {
            match self.audio_convert.next_event().await {
                Ok(SourceEvent::Frame(frame)) => {
                    let samples = frame.into_data().samples.into_samples::<f32>();
                    log::trace!(
                        "new raw audio frame with length: {} received.",
                        samples.len()
                    );
                    return Some(samples);
                }
                Ok(SourceEvent::EndOfData) => {
                    return None;
                }
                Ok(source_event) => {
                    log::warn!("Received unexpected audio source event: {source_event:?}");
                }
                Err(err) => {
                    log::error!("Received error on audio source event retrieval: {err:?}");
                    return None;
                }
            }
        }
    }
}
