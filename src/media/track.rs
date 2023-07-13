// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! GStreamer custom component based on `appsrc` to play audio tracks provided
//! by the application on demand without loading them off the disk each time.

use anyhow::bail;
use anyhow::Result;
use byte_slice_cast::*;
use gst::prelude::*;
use hound::WavReader;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::sync::broadcast;

const SAMPLES_PER_BUF: u64 = 4096;
const BYTES_PER_SAMPLE: u64 = 2;
const BYTES_PER_BUF: u64 = SAMPLES_PER_BUF * BYTES_PER_SAMPLE;
const SAMPLE_RATE: u64 = 48000;
const CHANNELS: u64 = 2;

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
wav_file!(DE_MODERATOR_MUTED => "../../audio/DE_moderator_muted.wav");
wav_file!(DE_ENTERED_WAITING_ROOM => "../../audio/DE_waiting_room_entered.wav");

// Sounds taken from
// https://gitlab.senfcall.de/senfcall-public/mute-and-unmute-sounds/-/tree/master/
wav_file!(DE_MUTED => "../../audio/DE_muted.wav");
wav_file!(DE_UNMUTED => "../../audio/DE_unmuted.wav");
wav_file!(DE_HAND_RAISED => "../../audio/DE_hand_raised.wav");
wav_file!(DE_HAND_LOWERED => "../../audio/DE_hand_lowered.wav");

/// Track to play
pub enum Track {
    Silence,
    WelcomeConferenceId,
    WelcomePasscode,
    WelcomeUsage,
    ConferenceClosed,
    InputInvalid,
    ModeratorMuted,
    EnteredWaitingRoom,
    Muted,
    Unmuted,
    HandRaised,
    HandLowered,
}

// substitute for gst_util_uint64_scale
fn scale(val: u64, num: u64, denom: u64) -> u64 {
    val * num / denom
}

/// Handle to the elements element data
///
/// Used to set the track the component should play
pub struct TrackController {
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
            Track::ModeratorMuted => data.track = Some(DE_MODERATOR_MUTED.clone()),
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
    num_samples: u64,
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

/// Creates an `appsrc` element which has one `src` pad which
/// emits a constant stream of [SAMPLE_RATE] [CHANNELS] channel interleaved (i16/S16LE) audio
///
/// Returns a `TrackController` and `Element`. The controller
/// can be used to control which tracks the element is playing
pub fn create_track_src() -> Result<(TrackController, gst::Element)> {
    let appsrc = gst_app::AppSrc::builder()
        .name("track-player")
        .caps(
            // Set the capabilities of the appsrc` to [SAMPLE_RATE] [CHANNEL] channel interleaved (i16/S16LE) audio
            &gst::Caps::builder("audio/x-raw")
                .field("format", "S16LE")
                .field("rate", SAMPLE_RATE as i32)
                .field("channels", CHANNELS as i32)
                .field("layout", "interleaved")
                .build(),
        )
        .format(gst::Format::Time)
        .max_bytes(1)
        .block(true)
        .build();

    let data = Arc::new(Mutex::new(ElementData {
        track: None,
        cursor: 0,
        playback_finished_responder: None,
        num_samples: 0,
    }));

    let data_clone = data.clone();
    appsrc.set_callbacks(
        gst_app::AppSrcCallbacks::builder()
            .need_data(move |appsrc, _size| {
                if let Err(err) = push_data(appsrc, &data_clone) {
                    log::error!("failed to push data, {:?}", err);
                }
            })
            .build(),
    );

    Ok((TrackController { data }, appsrc.upcast()))
}

fn push_data(appsrc: &gst_app::AppSrc, data: &Arc<Mutex<ElementData>>) -> Result<()> {
    let mut data = data.lock();

    // Get the samples to play
    let samples_to_play = match &data.track {
        Some(track) => &track[data.cursor / BYTES_PER_SAMPLE as usize..],
        None => &[][..],
    };

    let mut bytes_to_play = samples_to_play.as_byte_slice();

    let duration = gst::ClockTime::from_nseconds(scale(
        SAMPLES_PER_BUF / CHANNELS,
        gst::ClockTime::SECOND.nseconds(),
        SAMPLE_RATE,
    ));

    let timestamp = gst::ClockTime::from_nseconds(scale(
        data.num_samples / CHANNELS,
        gst::ClockTime::SECOND.nseconds(),
        SAMPLE_RATE,
    ));

    let mut buffer = gst::Buffer::with_size(BYTES_PER_BUF as usize)?;

    let buffer_ref = buffer.make_mut();
    buffer_ref.set_duration(duration);
    buffer_ref.set_pts(timestamp);

    // Create a constant size buffer from zero or more samples
    if bytes_to_play.is_empty() {
        buffer_ref
            .copy_from_slice(0, &[0; BYTES_PER_BUF as usize][..])
            .expect("copy must work, amount was allocated on buffer create");
    } else if bytes_to_play.len() < BYTES_PER_BUF as usize {
        buffer_ref
            .copy_from_slice(0, &[0; BYTES_PER_BUF as usize])
            .unwrap();
        buffer_ref.copy_from_slice(0, bytes_to_play).unwrap();
    } else {
        bytes_to_play = &bytes_to_play[..BYTES_PER_BUF as usize];
        buffer_ref.copy_from_slice(0, bytes_to_play).unwrap();
    };

    let bytes_to_play_len = bytes_to_play.len();
    if bytes_to_play_len == 0 {
        data.stop_playback();
    } else {
        data.cursor += bytes_to_play_len;
    }
    data.num_samples += SAMPLES_PER_BUF;

    // Drop mutex guard because pushing below can block
    drop(data);

    // Push audio bytes to the appsrc
    let ret = appsrc.push_buffer(buffer);

    // Check flow return
    if let Err(err) = ret {
        bail!("failed to push buffer, got {:?}", err);
    }

    Ok(())
}

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
        Lazy::force(&DE_MODERATOR_MUTED);
        Lazy::force(&DE_ENTERED_WAITING_ROOM);
        Lazy::force(&DE_MUTED);
        Lazy::force(&DE_UNMUTED);
    }
}
