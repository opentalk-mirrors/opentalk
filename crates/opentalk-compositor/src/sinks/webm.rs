// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use glib::object::ObjectExt;
use gstreamer::{Bin, GhostPad};
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::{
    elements::matroskas3sink::DEFAULT_CHUNK_SIZE,
    gstreamer::{add_ghost_pad, GStreamerSink},
    parse_bin_from_description_with_context, EncoderType, GstBinErrorExt,
};

#[derive(Debug)]
pub struct WebMSink {
    bin: Bin,
    video_sink: GhostPad,
    audio_sink: GhostPad,
    buffer: broadcast::Sender<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebMParameters {
    pub encoder_type: EncoderType,
    pub chunk_size: Option<u64>,
}

impl WebMSink {
    /// Create and add new `WebM` sink into existing pipeline.
    ///
    /// # Errors
    ///
    /// This can fail for the following reasons:
    /// - Cannot create `videoconvert` in `GStreamer`.
    /// - Cannot create `videorate` in `GStreamer`.
    /// - Cannot create `videoscale` in `GStreamer`.
    /// - Cannot create `mux` in `GStreamer`.
    /// - Cannot create `audioconvert` in `GStreamer`.
    /// - Cannot create `webmmux` in `GStreamer`.
    /// - Cannot create `queue` in `GStreamer`.
    /// - Cannot create `filedsink` in `GStreamer`.
    /// - The local address in `params.address` cannot be listened.
    /// - `GhostPad` cannot be created for `video_sink` or `audio_sink`.
    pub fn create(params: &WebMParameters) -> Result<Self> {
        //
        trace!("{params:?})");

        // The video encoder is setup for a buffer of 6s (vp9enc buffer-size=6000 [ms]) max.
        // Therefore the audio queue is set to 8s (queue max-size-time=8000000000 [ns]) and
        // the video queue to 2s (queue max-size-time=8000000000 [ns])
        let bin = parse_bin_from_description_with_context(
            &format!(
                r#"
                name="WebM-Sink"

                videoconvert
                    name=video
                ! videorate
                    drop-only=true
                ! videoscale
                ! {encoder}
                ! queue
                    max-size-time=2000000000 max-size-bytes=0 max-size-buffers=0
                ! mux.

                audioconvert
                    name=audio
                ! audio/x-raw,format=S16LE,layout=interleaved,rate=48000
                ! opusenc bitrate=96000 complexity=7 audio-type=voice
                ! queue
                    max-size-time=8000000000 max-size-bytes=0 max-size-buffers=0
                ! mux.

                webmmux
                    name=mux
                    writing-app=OpenTalk
                    offset-to-zero=true
                ! opentalk-matroska-s3-sink
                    name=matroska-s3
                    chunk-size={chunk_size}
                "#,
                encoder = match params.encoder_type {
                    EncoderType::CPU =>
                        "
                        video/x-raw,format=I420,pixel-aspect-ratio=1/1,colorimetry=bt709
                        ! vp8enc
                          deadline=1 cpu-used=4 threads=4 token-partitions=1
                          end-usage=cbr target-bitrate=2600000 undershoot=90
                          buffer-size=6000 buffer-initial-size=4000 buffer-optimal-size=5000
                          dropframe-threshold=25 resize-allowed=true
                    ",
                    EncoderType::VAAPI =>
                        "
                        video/x-raw,format=NV12,pixel-aspect-ratio=1/1,colorimetry=bt709
                        ! vaapivp9enc
                    ",
                },
                chunk_size = params.chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE)
            ),
            false,
        )?;

        let video_sink = add_ghost_pad(&bin, "video", "sink")
            .context("unable to add GhostPad for video sink")?;
        let audio_sink = add_ghost_pad(&bin, "audio", "sink")
            .context("unable to add GhostPad for audio sink")?;

        let matroska_s3 = bin.by_name_with_context("matroska-s3")?;

        let buffer = broadcast::Sender::new(10);
        let sender = buffer.clone();

        // This can never panic, because the data schema is enforced and contains the part number
        matroska_s3.connect("part", false, move |values| {
            let Ok(Ok(part)) = values[1].get::<u64>().map(u32::try_from) else {
                return None;
            };
            let Ok(bytes) = values[2].get::<glib::Bytes>() else {
                return None;
            };
            log::trace!(
                "Received part with part number {part} and {} bytes",
                bytes.len(),
            );

            let mut data = Vec::with_capacity(size_of_val(&part) + bytes.len());
            data.extend_from_slice(&part.to_be_bytes());
            data.extend_from_slice(&bytes);

            let _ = sender.send(data);

            None
        });

        Ok(Self {
            bin,
            video_sink,
            audio_sink,
            buffer,
        })
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.buffer.subscribe()
    }
}

impl GStreamerSink for WebMSink {
    fn video(&self) -> GhostPad {
        self.video_sink.clone()
    }

    fn audio(&self) -> GhostPad {
        self.audio_sink.clone()
    }

    fn bin(&self) -> Bin {
        self.bin.clone()
    }
}
