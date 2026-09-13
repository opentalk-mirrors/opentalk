// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use gstreamer::{Bin, GhostPad};
use serde::Deserialize;

use crate::{
    gstreamer::{add_ghost_pad, GStreamerSink},
    parse_bin_from_description_with_context, EncoderType,
};

const DEFAULT_AUDIO_RATE: usize = 48000;
const DEFAULT_AUDIO_BITRATE: usize = 96000;
const DEFAULT_VIDEO_BITRATE: usize = 6000;

/// RTMP compositor output to stream over RTMP.
#[derive(Debug)]
pub struct RTMPSink {
    bin: Bin,
    video_sink_pad: GhostPad,
    audio_sink_pad: GhostPad,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RTMPParameters {
    pub location: String,
    pub audio_bitrate: Option<usize>,
    pub audio_rate: Option<usize>,
    pub video_bitrate: Option<usize>,
    pub video_speed_preset: Option<SpeedPreset>,
    pub encoder_type: EncoderType,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub enum SpeedPreset {
    Ultrafast = 1,
    Superfast = 2,
    Veryfast = 3,
    Faster = 4,
    Fast = 5,
    #[default]
    Medium = 6,
    Slow = 7,
    Slower = 8,
    Veryslow = 9,
    Placebo = 10,
    None = 0,
}

impl RTMPSink {
    /// Create and add new rtmp sink into existing pipeline.
    ///
    /// # Errors
    ///
    /// This can fail for the following reasons:
    /// - Unable to create `videoconvert` for `GStreamer`.
    /// - Unable to create `x264enc` for `GStreamer`.
    /// - Unable to create `h264parse` for `GStreamer`.
    /// - Unable to create `mux` for `GStreamer`.
    /// - Unable to create `audioconvert` for `GStreamer`.
    /// - Unable to create `audioresample` for `GStreamer`.
    /// - Unable to create `fdkaacenc` for `GStreamer`.
    /// - Unable to create `aacparse` for `GStreamer`.
    /// - Unable to create `flvmux` for `GStreamer`.
    /// - Unable to create `rtmpsink` for `GStreamer`.
    /// - `GhostPad` cannot be created for the `video_sink_pad` or `audio_sink_pad`.
    pub fn create(parameters: RTMPParameters) -> Result<RTMPSink> {
        let bin = parse_bin_from_description_with_context(
            format!(
                r#"
            name="RTMP-Sink"

            videoconvert
                name=video
            ! {encoder_type}
            ! video/x-h264,profile=high
            ! h264parse
            ! mux.

            audioconvert
                name=audio
            ! audioresample
            ! audio/x-raw,rate={audio_rate}
            ! fdkaacenc bitrate={audio_bitrate}
            ! audio/mpeg
            ! aacparse
            ! audio/mpeg, mpegversion=4
            ! mux.

            flvmux
                name=mux
                streamable=true
            ! rtmpsink
                location="{location}"
            "#,
                encoder_type = match parameters.encoder_type {
                    EncoderType::CPU => format!(
                        "x264enc speed-preset={video_speed_preset} tune=zerolatency bitrate={video_bitrate}",
                        video_bitrate = parameters.video_bitrate.unwrap_or(DEFAULT_VIDEO_BITRATE),
                        video_speed_preset = parameters.video_speed_preset.unwrap_or_default() as usize,
                    ),
                    EncoderType::VAAPI => format!(
                        "vaapih264enc tune=low-power bitrate={video_bitrate}",
                        video_bitrate = parameters.video_bitrate.unwrap_or(DEFAULT_VIDEO_BITRATE),
                    ),
                },
                location = parameters.location,
                audio_bitrate = parameters.audio_bitrate.unwrap_or(DEFAULT_AUDIO_BITRATE),
                audio_rate = parameters.audio_rate.unwrap_or(DEFAULT_AUDIO_RATE),
            )
            .as_str(),
            false,
        )?;

        let video_sink_pad = add_ghost_pad(&bin, "video", "sink")
            .context("unable to add GhostPad for video sink")?;
        let audio_sink_pad = add_ghost_pad(&bin, "audio", "sink")
            .context("unable to add GhostPad for audio sink")?;

        Ok(Self {
            bin,
            video_sink_pad,
            audio_sink_pad,
        })
    }
}

impl GStreamerSink for RTMPSink {
    fn video(&self) -> GhostPad {
        self.video_sink_pad.clone()
    }

    fn audio(&self) -> GhostPad {
        self.audio_sink_pad.clone()
    }

    fn bin(&self) -> Bin {
        self.bin.clone()
    }
}
