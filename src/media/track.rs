//! GStreamer custom component based on `appsrc` to play audio tracks provided
//! by the application on demand without loading them off the disk each time.

use anyhow::bail;
use anyhow::Result;
use byte_slice_cast::*;
use gst::glib;
use gst::prelude::*;
use hound::WavReader;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::Weak;

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

wav_file!(DE_ENTER_ID => "../../audio/DE_enter_id.wav");
wav_file!(DE_ENTER_PIN => "../../audio/DE_enter_pin.wav");
wav_file!(DE_SUCCESS => "../../audio/DE_success.wav");
wav_file!(DE_FAILURE => "../../audio/DE_failure.wav");

/// Track to play
pub enum Track {
    // Silence,
    EnterId,
    EnterPin,
    Success,
    Failure,
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
    /// Set the track to play, will stop any track playing currently
    pub fn play_track(&self, track: Track) {
        let mut data = self.data.lock();

        data.cursor = 0;

        match track {
            // Track::Silence => data.track = None,
            Track::EnterId => data.track = Some(DE_ENTER_ID.clone()),
            Track::EnterPin => data.track = Some(DE_ENTER_PIN.clone()),
            Track::Success => data.track = Some(DE_SUCCESS.clone()),
            Track::Failure => data.track = Some(DE_FAILURE.clone()),
        }
    }
}

struct ElementData {
    track: Option<Arc<[i16]>>,
    cursor: usize,

    num_samples: u64,

    /// Flag to check if the `push_data` callback is still registered/running
    push_data_running: Option<glib::SourceId>,
}

/// Creates an `appsrc` element which has one `src` pad which
/// emits a constant stream of 48000 2 channel interleaved (i16/S16LE) audio
///
/// Returns a `TrackController` and `Element`. The controller
/// can be used to control which tracks the element is playing
pub fn create_track_src() -> Result<(TrackController, gst::Element)> {
    let appsrc = gst::ElementFactory::make("appsrc", Some("track-player"))?
        .downcast::<gst_app::AppSrc>()
        .unwrap();

    let data = Arc::new(Mutex::new(ElementData {
        track: None,
        cursor: 0,
        num_samples: 0,
        push_data_running: None,
    }));

    // Set the capabilities of the appsrc` to 48000 2 channel interleaved (i16/S16LE) audio
    appsrc.set_caps(Some(
        &gst::Caps::builder("audio/x-raw")
            .field("format", "S16LE")
            .field("rate", 48000i32)
            .field("channels", 2i32)
            .field("layout", "interleaved")
            .build(),
    ));

    // Set the format property to time
    appsrc.set_format(gst::Format::Time);
    appsrc.set_is_live(true);
    appsrc.set_do_timestamp(true);

    // Register need-data callback which will register the `push_data` callback to the glib runtime
    // push_data will be called until the `appsrc` buffer is completely full, then it un-registers itself
    // by returning glib::Continue(false)
    let data_clone = data.clone();
    let data_clone2 = data.clone();
    appsrc.set_callbacks(
        gst_app::AppSrcCallbacks::builder()
            .need_data(move |appsrc, _size| {
                let mut data = data_clone.lock();

                // Avoid registering the callback twice
                if data.push_data_running.is_none() {
                    // Register push-data to glib runtime which will calls it until the `appsrc` buffer is full
                    data.push_data_running = Some(glib::idle_add(push_data(
                        appsrc.clone(),
                        Arc::downgrade(&data_clone),
                    )));
                }
            })
            .enough_data(move |_appsrc| {
                let mut data = data_clone2.lock();

                if let Some(source_id) = data.push_data_running.take() {
                    glib::source_remove(source_id);
                }
            })
            .build(),
    );

    Ok((TrackController { data }, appsrc.upcast()))
}

fn push_data(
    appsrc: gst_app::AppSrc,
    data: Weak<Mutex<ElementData>>,
) -> impl FnMut() -> glib::Continue {
    move || {
        let data = if let Some(data) = data.upgrade() {
            data
        } else {
            return glib::Continue(false);
        };

        let mut data = data.lock();

        let continue_ = match try_push_data(&appsrc, &mut data) {
            Ok(continue_) => continue_,
            Err(e) => {
                log::error!("failed to push data, {:?}", e);
                glib::Continue(false)
            }
        };

        if !continue_.0 {
            data.push_data_running = None;
        }

        continue_
    }
}

fn try_push_data(appsrc: &gst_app::AppSrc, data: &mut ElementData) -> Result<glib::Continue> {
    const SAMPLES_PER_BUF: u64 = 4096;
    const BYTES_PER_SAMPLE: u64 = 2;
    const BYTES_PER_BUF: u64 = SAMPLES_PER_BUF * BYTES_PER_SAMPLE;
    const SAMPLE_RATE: u64 = 48000;
    const CHANNELS: u64 = 2;

    // Check if next push would block because the buffer is full
    let current_bytes = appsrc.current_level_bytes();
    let max_bytes = appsrc.max_bytes();

    if current_bytes >= max_bytes {
        return Ok(glib::Continue(false));
    }

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

    data.num_samples += SAMPLES_PER_BUF;

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

    // Set the cursor forward
    data.cursor += bytes_to_play.len();

    // Push audio bytes to the appsrc
    let ret = appsrc.push_buffer(buffer);

    // Check flow return
    if let Err(err) = ret {
        bail!("failed to push buffer, got {:?}", err);
    }

    Ok(glib::Continue(true))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn verify_wav_files() {
        Lazy::force(&DE_ENTER_ID);
        Lazy::force(&DE_ENTER_PIN);
        Lazy::force(&DE_SUCCESS);
        Lazy::force(&DE_FAILURE);
    }
}
