//! GSttreamer custom component based on `appsrc` to play audio tracks provided
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

    /// Flag to check if the `push_data` callback is still registered/running
    push_data_running: bool,
}

/// Creates an `appsrc` element which has one `src` pad which
/// emits a constant stream of 48000 2 channel interleaved (i16/S16LE) audio
///
/// Returns a `TrackController` and `Element`. The controller
/// can be used to control which tracks the element is playing
pub fn create_track_src() -> Result<(TrackController, gst::Element)> {
    let appsrc = gst::ElementFactory::make("appsrc", None)?;

    let data = Arc::new(Mutex::new(ElementData {
        track: None,
        cursor: 0,
        push_data_running: false,
    }));

    // Set the capabilities of the appsrc` to 48000 2 channel interleaved (i16/S16LE) audio
    appsrc.set_property(
        "caps",
        gst::Caps::new_simple(
            "audio/x-raw",
            &[
                ("format", &"S16LE"),
                ("rate", &48000i32),
                ("channels", &2i32),
                ("layout", &"interleaved"),
            ],
        )
        .to_value(),
    )?;

    // Set the format property to time
    appsrc.set_property("format", gst::Format::Time)?;

    // Register need-data callback which will register the `push_data` callback to the glib runtime
    // push_data will be called until the `appsrc` buffer is completely full, then it un-registers itself
    // by returning glib::Continue(false)
    let data_clone = data.clone();
    appsrc.connect("need-data", true, move |values| {
        let appsrc = values[0].get::<gst::Element>().expect("appsrc is element");

        let mut data = data_clone.lock();

        // Avoid registering the callback twice
        if !data.push_data_running {
            data.push_data_running = true;

            // Register push-data to glib runtime which will calls it until the `appsrc` buffer is full
            let fun = push_data(appsrc, Arc::downgrade(&data_clone));
            glib::idle_add(fun);
        }

        None
    })?;

    Ok((TrackController { data }, appsrc))
}

fn push_data(
    appsrc: gst::Element,
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
            data.push_data_running = false;
        }

        continue_
    }
}

fn try_push_data(appsrc: &gst::Element, data: &mut ElementData) -> Result<glib::Continue> {
    // Check if next push would block because the buffer is full
    let current_bytes = appsrc.property("current-level-bytes")?.get::<u64>()?;
    let max_bytes = appsrc.property("max-bytes")?.get::<u64>()?;

    if current_bytes >= max_bytes {
        return Ok(glib::Continue(false));
    }

    // Get the samples to play
    let to_play = match &data.track {
        Some(track) => &track[data.cursor / 2..],
        None => &[][..],
    };

    let mut to_play = to_play.as_byte_slice();

    let duration =
        gst::ClockTime::from_nseconds(scale(4096, gst::ClockTime::SECOND.nseconds(), 48000));

    // Create a constant size buffer from zero or more samples
    let buffer = if to_play.is_empty() {
        let mut buffer = gst::Buffer::from_slice(&[0; 4096][..]);
        let buffer_ref = buffer.make_mut();
        buffer_ref.set_duration(duration);

        buffer
    } else if to_play.len() < 4096 {
        let mut buffer = gst::Buffer::with_size(4096).unwrap();
        let buffer_ref = buffer.make_mut();

        buffer_ref.copy_from_slice(0, &[0; 4069]).unwrap();
        buffer_ref.copy_from_slice(0, to_play).unwrap();
        buffer_ref.set_duration(duration);

        buffer
    } else {
        to_play = &to_play[..4096];

        let mut buffer = gst::Buffer::with_size(4096).unwrap();
        let buffer_ref = buffer.make_mut();
        buffer_ref.copy_from_slice(0, to_play).unwrap();
        buffer_ref.set_duration(duration);

        buffer
    };

    // Set the cursor forward
    data.cursor += to_play.len();

    // Push audio bytes to the appsrc
    let ret = appsrc.emit_by_name("push-buffer", &[&buffer])?;

    // Check flow return
    if let Some(ret) = ret.and_then(|ret| ret.get::<gst::FlowReturn>().ok()) {
        if ret != gst::FlowReturn::Ok {
            bail!("failed to push buffer, got {:?}", ret);
        }
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
