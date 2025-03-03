// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::{RAW_AUDIO_CONFIG, RAW_AUDIO_CONFIG_RANGE};
use crate::settings::Language;
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
    ($ident:ident => $file:expr) => {
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
macro_rules! audio_files {
    ($($name:literal, $ident:ident;)*) => {
        #[allow(unused)]
        pub(crate) enum Track {
            $($ident,)*
        }

        impl Track {
            fn data(&self, lang: Language) -> Arc<[i16]> {
                match lang {
                    Language::EN => {
                        match self {
                            $(
                            Self::$ident => {
                                wav_file!(FILE => concat!("../../audio/en/", $name, ".wav"));
                                FILE.clone()
                            }
                            )*
                        }
                    }
                    Language::DE => {
                        match self {
                            $(
                            Self::$ident => {
                                wav_file!(FILE => concat!("../../audio/de/", $name, ".wav"));
                                FILE.clone()
                            }
                            )*
                        }
                    }
                }
            }
        }

        #[test]
        fn all_wav_files_are_valid() {
            $(
                let _ = Track::$ident.data(Language::EN);
                let _ = Track::$ident.data(Language::DE);
            )*
        }
    };
}

audio_files! {
    "consent_stopped_accept", ConsentStoppedAccept;
    "consent_stopped_reject", ConsentStoppedReject;
    "conference_closed", ConferenceClosed;
    "conference_timelimit", ConferenceTimelimit;
    "conference_userlimit", ConferenceUserlimit;
    "consent_accept", ConsentAccept;
    "consent_info", ConsentInfo;
    "consent_invalid", ConsentInvalid;
    "consent_reject", ConsentReject;
    "input_invalid", InputInvalid;
    "input_timeout", InputTimeout;
    "moderator_kicked", ModeratorKicked;
    "moderator_lowered_hand", ModeratorLoweredHand;
    "moderator_muted", ModeratorMuted;
    "moderator_paused", ModeratorPaused;
    "moderator_resumed", ModeratorResumed;
    "moderator_send_waiting_room", ModeratorSendWaitingRoom;
    "waiting_room_entered", WaitingRoomEntered;
    "welcome_conference_id", WelcomeConferenceId;
    "welcome_passcode", WelcomePasscode;
    "welcome_refused", WelcomeRefused;
    "welcome_time", WelcomeTime;
    "welcome_unavailable", WelcomeUnavailable;
    "welcome_usage", WelcomeUsage;

    // Not generated! Used for mute/unmute, raise/lower hand
    "on", On;
    "off", Off;
}

/// Handle to the elements element data
///
/// Used to set the track the component should play
pub(crate) struct TrackController {
    data: Arc<Mutex<ElementData>>,
}

impl TrackController {
    pub fn play_track_and_respond(
        &self,
        language: Language,
        track: Option<Track>,
        responder: broadcast::Sender<()>,
    ) {
        self.play_track_int(language, track, Some(responder));
    }

    pub fn play_track(&self, language: Language, track: Option<Track>) {
        self.play_track_int(language, track, None);
    }

    /// Set the track to play, stopping any track currently playing
    fn play_track_int(
        &self,
        language: Language,
        track: Option<Track>,
        responder: Option<broadcast::Sender<()>>,
    ) {
        let mut data = self.data.lock();

        data.stop_playback();
        data.playback_finished_responder = responder;

        data.track = track.map(|t| t.data(language));
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
