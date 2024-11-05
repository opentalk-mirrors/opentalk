// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::{RAW_AUDIO_CONFIG, RAW_AUDIO_CONFIG_RANGE};
use ezk::{ConfigRange, Frame, NextEventIsCancelSafe, Source, SourceEvent};
use ezk_audio::{
    Channels, RawAudio, RawAudioConfig, RawAudioConfigRange, RawAudioFrame, SampleRate,
};
use hound::WavReader;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::cmp;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::{interval, Interval};

const SAMPLE_RATE: u64 = 48000;
const CHANNELS: u64 = 2;
const BUFFERS_PER_SECOND: u64 = 50;
const SAMPLES_PER_CHANNEL: u64 = SAMPLE_RATE / BUFFERS_PER_SECOND;
const SAMPLES_PER_BUF: u64 = SAMPLES_PER_CHANNEL * CHANNELS;

macro_rules! wav_file {
    ($ident:ident=>$file:literal) => {
        static $ident: Lazy<Arc<[i16]>> = Lazy::new(|| {
            let bytes: &[u8] = include_bytes!($file);

            let mut reader = WavReader::new(bytes).expect(concat!("invalid wav file ", $file));

            reader
                .samples::<i16>()
                .map(|res| res.expect(concat!("invalid wav file ", $file)))
                .collect()
        });
    };
}

// Statically load all audio files as WAV and decode them lazily
// WAV files must be 48000Hz 2 channel interleaved (and optimally i16/S16LE) audio

wav_file!(DE_WELCOME_CONFERENCE_ID => "../../audio/DE_welcome_conference_id.wav");
wav_file!(DE_WELCOME_PASSCODE => "../../audio/DE_welcome_passcode.wav");
wav_file!(DE_WELCOME_USAGE => "../../audio/DE_welcome_usage.wav");
wav_file!(DE_CONFERENCE_CLOSED => "../../audio/DE_conference_closed.wav");
wav_file!(DE_INPUT_INVALID => "../../audio/DE_input_invalid.wav");
wav_file!(DE_ENTERED_WAITING_ROOM => "../../audio/DE_waiting_room_entered.wav");

// Sounds taken from
// https://gitlab.senfcall.de/senfcall-public/mute-and-unmute-sounds/-/tree/master/
wav_file!(DE_MUTED => "../../audio/DE_muted.wav");
wav_file!(DE_UNMUTED => "../../audio/DE_unmuted.wav");
wav_file!(DE_HAND_RAISED => "../../audio/DE_hand_raised.wav");
wav_file!(DE_HAND_LOWERED => "../../audio/DE_hand_lowered.wav");

/// Track to play
pub(crate) enum Track {
    Silence,
    WelcomeConferenceId,
    WelcomePasscode,
    WelcomeUsage,
    ConferenceClosed,
    InputInvalid,
    EnteredWaitingRoom,
    Muted,
    Unmuted,
    HandRaised,
    HandLowered,
}

/// Handle to the elements element data
///
/// Used to set the track the component should play
pub(crate) struct TrackController {
    data: Arc<Mutex<ElementData>>,
}

impl TrackController {
    pub fn play_track_and_respond(&self, track: Track, responder: broadcast::Sender<()>) {
        self.play_track_int(track, Some(responder));
    }

    pub fn play_track(&self, track: Track) {
        self.play_track_int(track, None);
    }

    /// Set the track to play, stopping any track currently playing
    fn play_track_int(&self, track: Track, responder: Option<broadcast::Sender<()>>) {
        let mut data = self.data.lock();

        data.stop_playback();
        data.playback_finished_responder = responder;

        match track {
            Track::Silence => data.track = None,
            Track::WelcomeConferenceId => data.track = Some(DE_WELCOME_CONFERENCE_ID.clone()),
            Track::WelcomePasscode => data.track = Some(DE_WELCOME_PASSCODE.clone()),
            Track::WelcomeUsage => data.track = Some(DE_WELCOME_USAGE.clone()),
            Track::ConferenceClosed => data.track = Some(DE_CONFERENCE_CLOSED.clone()),
            Track::InputInvalid => data.track = Some(DE_INPUT_INVALID.clone()),
            Track::EnteredWaitingRoom => data.track = Some(DE_ENTERED_WAITING_ROOM.clone()),
            Track::Muted => data.track = Some(DE_MUTED.clone()),
            Track::Unmuted => data.track = Some(DE_UNMUTED.clone()),
            Track::HandRaised => data.track = Some(DE_HAND_RAISED.clone()),
            Track::HandLowered => data.track = Some(DE_HAND_LOWERED.clone()),
        }
    }
}

struct ElementData {
    track: Option<Arc<[i16]>>,
    cursor: usize,
    playback_finished_responder: Option<broadcast::Sender<()>>,
}

impl ElementData {
    fn stop_playback(&mut self) {
        if self.track.is_some() {
            if let Some(responder) = self.playback_finished_responder.take() {
                let _ = responder.send(());
            }
        };

        self.track = None;
        self.cursor = 0;
    }
}

pub(crate) struct TrackSource {
    data: Arc<Mutex<ElementData>>,
    interval: Interval,
    timestamp: u64,
}

impl TrackSource {
    pub(crate) fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(ElementData {
                track: None,
                cursor: 0,
                playback_finished_responder: None,
            })),
            interval: interval(Duration::from_millis(20)),
            timestamp: 0,
        }
    }

    pub(crate) fn controller(&self) -> TrackController {
        TrackController {
            data: self.data.clone(),
        }
    }
}

impl Source for TrackSource {
    type MediaType = RawAudio;

    async fn capabilities(&mut self) -> ezk::Result<Vec<RawAudioConfigRange>> {
        Ok(vec![RAW_AUDIO_CONFIG_RANGE])
    }

    async fn negotiate_config(
        &mut self,
        available: Vec<RawAudioConfigRange>,
    ) -> ezk::Result<RawAudioConfig> {
        assert!(available.iter().any(|r| r.contains(&RAW_AUDIO_CONFIG)));
        Ok(RAW_AUDIO_CONFIG)
    }

    async fn next_event(&mut self) -> ezk::Result<SourceEvent<RawAudio>> {
        self.interval.tick().await;

        let mut data = self.data.lock();

        let mut samples_to_play = match &data.track {
            Some(track) => track[data.cursor..].to_vec(),
            None => vec![],
        };

        if samples_to_play.is_empty() {
            data.stop_playback();
        } else {
            data.cursor += cmp::min(samples_to_play.len(), SAMPLES_PER_BUF as usize);
        }

        samples_to_play.resize(SAMPLES_PER_BUF as usize, 0i16);

        let timestamp = self.timestamp;
        self.timestamp += SAMPLES_PER_BUF;

        Ok(SourceEvent::Frame(Frame::new(
            RawAudioFrame {
                sample_rate: SampleRate(SAMPLE_RATE as u32),
                channels: Channels::NotPositioned(CHANNELS as u32),
                samples: samples_to_play.into(),
            },
            timestamp,
        )))
    }
}

impl NextEventIsCancelSafe for TrackSource {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_wav_files() {
        Lazy::force(&DE_WELCOME_CONFERENCE_ID);
        Lazy::force(&DE_WELCOME_PASSCODE);
        Lazy::force(&DE_WELCOME_USAGE);
        Lazy::force(&DE_CONFERENCE_CLOSED);
        Lazy::force(&DE_INPUT_INVALID);
        Lazy::force(&DE_ENTERED_WAITING_ROOM);
        Lazy::force(&DE_MUTED);
        Lazy::force(&DE_UNMUTED);
    }
}
