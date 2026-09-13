// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fmt::Debug,
    future::{ready, Future},
    pin::Pin,
    time::Instant,
};

use anyhow::{Context, Result};
use ezk::Frame;
use ezk_audio::RawAudio;
use futures::{future::BoxFuture, FutureExt};
use glib::object::Cast;
use gstreamer::{
    prelude::{ElementExt, GstBinExt, PipelineExt},
    Bin, Buffer, ClockTime, Element, ElementFactory, Fraction, GhostPad, Sample, State,
    SystemClock,
};
use gstreamer_app::AppSrc;

use super::pipeline_watched::PipelineWatched;
use crate::{
    audio::{CHANNELS, SAMPLE_RATE},
    debug, GstBinErrorExt, GstElementBuilderErrorExt, GstElementErrorExt, GstGhostPadErrorExt,
    GstPadErrorExt, Sink, FRAMES_PER_SECOND, HEIGHT, WIDTH,
};

/// Trait of an output sink.
pub trait GStreamerSink: Send + Debug + 'static {
    fn bin(&self) -> Bin;

    /// Get sink pad of the audio sink.
    fn audio(&self) -> GhostPad;

    /// Get sink pad of the video sink.
    fn video(&self) -> GhostPad;

    /// Decides if the bus should not be watched, because the bus watcher is required outside of this sink
    fn init_bus_watch(&self) -> bool {
        true
    }

    /// Does the sink pipeline require an eos signal before nulling
    fn requires_eos(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub(crate) struct GStreamerActiveSink {
    pub(crate) pipeline: PipelineWatched,
    pub(crate) start: Instant,
    // The sink needs to be hold until it's dropped at the end
    pub(crate) inner: Box<dyn GStreamerSink>,
    pub(crate) audio_src: AppSrc,
    pub(crate) video_src: AppSrc,
}

impl GStreamerActiveSink {
    pub(crate) fn new(start: Instant, name: &str, sink: impl GStreamerSink) -> Result<Self> {
        let pipeline = PipelineWatched::new(name, sink.init_bus_watch(), sink.requires_eos())
            .context("unable to create PipelineWatched")?;

        pipeline.use_clock(Some(&SystemClock::obtain()));
        pipeline.set_base_time(ClockTime::ZERO);
        pipeline.set_start_time(None);

        let bin = sink.bin();
        pipeline.add_with_context(&bin)?;

        let audio_src = AppSrc::builder()
            .name("audiosrc")
            .caps(
                &gstreamer::Caps::builder("audio/x-raw")
                    .field("format", "S16LE")
                    .field("layout", "interleaved")
                    .field("rate", SAMPLE_RATE as i32)
                    .field("channels", CHANNELS as i32)
                    .build(),
            )
            .min_latency(200_000_000i64)
            .format(gstreamer::Format::Time)
            .max_bytes(1)
            .block(true)
            .is_live(true)
            .build();

        let video_src = AppSrc::builder()
            .name("videosrc")
            .caps(
                &gstreamer::Caps::builder("video/x-raw")
                    .field("format", "I420")
                    .field("width", WIDTH as i32)
                    .field("height", HEIGHT as i32)
                    .field("framerate", Fraction::new(25, 1))
                    .build(),
            )
            .min_latency(200_000_000i64)
            .format(gstreamer::Format::Time)
            .max_bytes(1)
            .block(true)
            .is_live(true)
            .build();

        let active_sink = GStreamerActiveSink {
            pipeline,
            start,
            inner: Box::new(sink),
            audio_src,
            video_src,
        };

        active_sink
            .link_audio_mixer()
            .context("unable to link AudioMixer to sink")?;

        active_sink
            .link_video_mixer()
            .context("unable to link VideoMixer to sink")?;

        active_sink
            .pipeline
            .set_state_with_context(State::Playing)?;
        active_sink
            .inner
            .bin()
            .set_state_with_context(State::Playing)?;
        active_sink
            .pipeline
            .sync_children_states()
            .context("unable to sync children states for pipeline")?;

        debug::dot(
            &*active_sink.pipeline,
            format!("link-sink_sink-pipeline_{name}").as_str(),
        );

        Ok(active_sink)
    }

    /// Link the given sink to the `audio_mixer`.
    ///
    /// # Errors
    ///
    /// This can fail if the audio sink could not be linked to the `audio_mixer`.
    fn link_audio_mixer(&self) -> Result<()> {
        let audio_src = self.audio_src.clone().upcast::<Element>();

        let queue = ElementFactory::make("queue")
            .property_from_str("leaky", "downstream")
            .property("max-size-time", 10_000_000_000u64)
            .property("max-size-bytes", 10_000_000u32)
            .property("max-size-buffers", 0u32)
            .build_with_context()?;
        let audioconvert = ElementFactory::make("audioconvert").build_with_context()?;

        self.pipeline
            .add_many_with_context(&[&audio_src, &queue, &audioconvert])?;

        Element::link_many_with_context(&[&audio_src, &queue, &audioconvert])?;

        audioconvert
            .static_pad_with_context("src")?
            .link_with_context(&self.inner.audio())?;

        Ok(())
    }

    /// Link the given sink to the `video_mixer`.
    ///
    /// # Errors
    ///
    /// This can fail if the video sink could not be linked to the `video_mixer`.
    fn link_video_mixer(&self) -> Result<()> {
        let video_sink = self.inner.video();
        let video_src = self.video_src.clone().upcast::<Element>();

        let queue = ElementFactory::make("queue")
            .property_from_str("leaky", "downstream")
            .property("max-size-time", 10_000_000_000u64)
            .property("max-size-bytes", 200_000_000u32)
            .property("max-size-buffers", 0u32)
            .build_with_context()?;
        let videoconvert = ElementFactory::make("videoconvert").build_with_context()?;

        self.pipeline
            .add_many_with_context(&[&video_src, &queue, &videoconvert])?;

        Element::link_many_with_context(&[&video_src, &queue, &videoconvert])?;

        videoconvert
            .static_pad_with_context("src")?
            .link_with_context(&video_sink)?;

        Ok(())
    }
}

impl Sink for GStreamerActiveSink {
    fn on_audio_frame(&mut self, frame: Frame<RawAudio>) -> BoxFuture<'_, Result<()>> {
        let samples = frame.data().samples.as_bytes();
        let mut buffer = gstreamer::Buffer::with_size(samples.len())
            .expect("unable to initialize gstreamer buffer with size");
        let mut_buffer = buffer.make_mut();
        mut_buffer
            .copy_from_slice(0, samples)
            .expect("unable to copy mut_buffer from slice samples: {samples:?}");

        mut_buffer.set_pts(gstreamer::ClockTime::from_mseconds(
            Instant::now().duration_since(self.start).as_millis() as u64,
        ));

        let sample = Sample::builder()
            .buffer(&buffer)
            .caps(
                &gstreamer::Caps::builder("audio/x-raw")
                    .field("format", "S16LE")
                    .field("layout", "interleaved")
                    .field("rate", SAMPLE_RATE as i32)
                    .field("channels", CHANNELS as i32)
                    .build(),
            )
            .build();

        if let Err(err) = self.audio_src.push_sample(&sample) {
            log::error!("Unable to push audio sample {sample:?}, received: {err:?}");
        }

        ready(Ok(())).boxed()
    }

    fn on_video_frame(&mut self, buffer: &[u8]) -> Result<()> {
        let mut gstreamer_buffer = Buffer::with_size(buffer.len())?;
        let mut_gstreamer_buffer = gstreamer_buffer.make_mut();
        mut_gstreamer_buffer
            .copy_from_slice(0, buffer)
            .ok()
            .context("unable to copy from slice")?;
        mut_gstreamer_buffer.set_pts(gstreamer::ClockTime::from_mseconds(
            Instant::now().duration_since(self.start).as_millis() as u64,
        ));

        let sample = Sample::builder()
            .buffer(&gstreamer_buffer)
            .caps(
                &gstreamer::Caps::builder("video/x-raw")
                    .field("format", "I420")
                    .field("width", WIDTH as i32)
                    .field("height", HEIGHT as i32)
                    .field("framerate", Fraction::new(FRAMES_PER_SECOND as i32, 1))
                    .build(),
            )
            .build();

        log::trace!("Push video sample: {sample:?}");
        if let Err(err) = self.video_src.push_sample(&sample) {
            log::error!("Unable to push video sample {sample:?}, received: {err:?}");
        }

        Ok(())
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async {
            self.pipeline.close().await;
            Ok(())
        })
    }
}

/// Adds a `GhostPad` to the given `Bin`.
///
/// # Errors
///
/// There are three reasons why this could fail:
/// - The element name cannot be found in the bin.
/// - The pad cannot be found in the element.
/// - The `GhostPad` cannot be added to the bin.
#[allow(clippy::must_use_candidate)]
pub fn add_ghost_pad(bin: &Bin, name: &str, pad: &str) -> Result<GhostPad> {
    trace!(
        "add_ghost_pad({bin}, {name}, {pad}) ",
        bin = debug::name(bin)
    );
    let pad = bin
        .by_name_with_context(name)?
        .static_pad_with_context(pad)?;
    let ghost_pad = GhostPad::with_target_with_context(Some(name), &pad)?;
    bin.add_pad_with_context(&ghost_pad)?;

    Ok(ghost_pad)
}
