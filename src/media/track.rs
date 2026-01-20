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
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::{Interval, interval};

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
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    "microphones_restriction_enabled", MicrophonesRestrictionEnabled;
    "microphones_restriction_lifted", MicrophonesRestrictionLifted;
    "recording_stopped_accept", RecordingStoppedAccept;
    "recording_stopped_reject", RecordingStoppedReject;

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
        track: Track,
        responder: broadcast::Sender<()>,
    ) {
        self.enqueue_track(language, track, Some(responder));
    }

    pub fn play_track(&self, language: Language, track: Track) {
        self.enqueue_track(language, track, None);
    }

    /// Stop the current track from playing
    pub fn stop_current_track(&self) {
        let mut data = self.data.lock();

        if let Some(front) = data.queue.pop_front()
            && let Some(responder) = front.playback_finished_responder
        {
            let _ = responder.send(());
        }
    }

    fn enqueue_track(
        &self,
        language: Language,
        track: Track,
        responder: Option<broadcast::Sender<()>>,
    ) {
        let mut data = self.data.lock();

        // Avoid queuing the same track more than once
        if data.queue.iter().any(|t| t.track == track) {
            return;
        }

        data.queue.push_back(PlayingTrack {
            track,
            track_data: track.data(language),
            cursor: 0,
            playback_finished_responder: responder,
        });
    }
}

struct ElementData {
    queue: VecDeque<PlayingTrack>,
}

struct PlayingTrack {
    track: Track,
    track_data: Arc<[i16]>,
    cursor: usize,
    playback_finished_responder: Option<broadcast::Sender<()>>,
}

impl ElementData {
    fn write_samples(&mut self, mut dst: &mut [i16]) {
        while let Some(front) = self.queue.front_mut() {
            let amt = dst.len().min(front.track_data.len() - front.cursor);
            if amt == 0 {
                return;
            }

            dst[..amt].copy_from_slice(&front.track_data[front.cursor..front.cursor + amt]);
            dst = &mut dst[amt..];
            front.cursor += amt;

            if front.cursor == front.track_data.len() {
                if let Some(sender) = &front.playback_finished_responder {
                    let _ = sender.send(());
                }
                self.queue.pop_front();
            }
        }
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
                queue: VecDeque::new(),
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

        let mut samples =
            vec![0i16; usize::try_from(SAMPLES_PER_BUF).expect("conversion to usize not possible")];
        data.write_samples(&mut samples);

        let timestamp = self.timestamp;
        self.timestamp += SAMPLES_PER_BUF;

        Ok(SourceEvent::Frame(Frame::new(
            RawAudioFrame {
                sample_rate: SampleRate(
                    u32::try_from(SAMPLE_RATE).expect("conversion to u32 not possible"),
                ),
                channels: Channels::NotPositioned(
                    u32::try_from(CHANNELS).expect("conversion to u32 not possible"),
                ),
                samples: samples.into(),
            },
            timestamp,
        )))
    }
}

impl NextEventIsCancelSafe for TrackSource {}
