// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::codec::Codec;
use crate::settings::Settings;
use h264_profile_level_id::Profile;
use sdp_types::{MediaDescription, RtpMap};
use std::str::FromStr;

/// Describes the codec used to receive audio/video
#[derive(Debug, Clone)]
pub struct SdpRecvCodec {
    pub payload: u8,
    pub codec: Codec,
}

impl SdpRecvCodec {
    pub fn make_rtpmap(&self) -> RtpMap {
        RtpMap {
            payload: self.payload,
            encoding: self.codec.encoding_name().into(),
            clock_rate: self.codec.clock_rate(),
            params: None,
        }
    }
}

/// Describes the codec used to send audio/video and how to packetize it
#[derive(Debug, Clone)]
pub struct SdpSendCodec {
    /// RTP payload number to recognize the codec by
    pub payload: u8,
    pub codec: Codec,
    pub codec_params: SdpSendCodecParams,
    // bits/s
    pub bitrate: Option<u32>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SdpSendCodecParams {
    None,
    H264 {
        width: u32,
        height: u32,
        #[allow(dead_code)] // TODO: fps limiting
        fps: u32,
        #[allow(dead_code)] // TODO: h264 profiles
        profile: String,
    },
}

pub fn choose_audio_codec(offer: &MediaDescription) -> Option<SdpSendCodec> {
    let codecs = [(9, Codec::G722), (8, Codec::G711PCMA), (0, Codec::G711PCMU)];

    for (pt, codec) in codecs {
        // first check all payload numbers in the m= line
        for &payload in &offer.media.fmts {
            if pt == payload {
                return Some(SdpSendCodec {
                    payload: pt,
                    codec,
                    codec_params: SdpSendCodecParams::None,
                    bitrate: None,
                });
            }
        }
    }

    None
}

pub fn offer_all_audio_codecs() -> Vec<SdpRecvCodec> {
    vec![
        SdpRecvCodec {
            payload: 9,
            codec: Codec::G722,
        },
        SdpRecvCodec {
            payload: 8,
            codec: Codec::G711PCMA,
        },
        SdpRecvCodec {
            payload: 0,
            codec: Codec::G711PCMU,
        },
    ]
}

pub fn offer_all_video_codecs() -> Vec<SdpRecvCodec> {
    vec![SdpRecvCodec {
        payload: 96,
        codec: Codec::H264,
    }]
}

pub fn available_video_codec() -> &'static str {
    "H264"
}

pub fn choose_video_codec(settings: &Settings, offer: &MediaDescription) -> Option<SdpSendCodec> {
    let rtpmap = offer
        .rtpmaps
        .iter()
        .find(|rtpmap| rtpmap.encoding == "H264" && rtpmap.clock_rate == 90000)?;

    let fmtp = offer
        .fmtps
        .iter()
        .find(|fmtp| fmtp.format == rtpmap.payload);

    let mut codec_params = SdpSendCodecParams::None;
    let mut bitrate = None;

    if let Some(fmtp) = fmtp {
        let params = fmtp.params.split(';');

        let mut level = None;
        let mut max_mbps: Option<u32> = None;
        let mut max_fs: Option<u32> = None;

        for p in params {
            let Some((key, value)) = p.split_once('=') else {
                continue;
            };

            if key == "profile-level-id" {
                match h264_profile_level_id::ProfileLevelId::from_str(value) {
                    Ok(v) => level = Some(v),
                    Err(e) => log::warn!("Failed to parse profile-level-id in H.264 params, {e}"),
                }

                continue;
            }

            let Ok(value) = value.parse::<u32>() else {
                log::warn!("Failed to parse value for '{value}' in H.264 params");
                continue;
            };

            match key {
                "max-mbps" => max_mbps = Some(value),
                "max-fs" => max_fs = Some(value),
                "max-br" => {
                    let value = value.min(settings.sip.max_video_bitrate);

                    // convert from kbit/s to bits/s
                    bitrate = Some(value * 1000);
                }
                _ => continue,
            }
        }

        if let Some(level) = level {
            // Calculate pre encode caps
            let ((width, height), fps) =
                calculate_best_resolution_and_fps(level.level(), max_mbps, max_fs);

            // Calculate post encode caps
            let h264_profile = match level.profile() {
                Profile::ConstrainedBaseline => "constrained-baseline",
                Profile::Baseline => "baseline",
                Profile::Main => "main",
                Profile::ConstrainedHigh => "constrained-high",
                Profile::High => "high",
                Profile::PredictiveHigh444 => {
                    log::warn!("Caller requested unsupported H.264 profile `PredictiveHigh444`");
                    return None;
                }
            };

            codec_params = SdpSendCodecParams::H264 {
                width,
                height,
                fps,
                profile: h264_profile.into(),
            }
        }
    }

    Some(SdpSendCodec {
        payload: rtpmap.payload,
        codec: Codec::H264,
        codec_params,
        bitrate,
    })
}

fn calculate_best_resolution_and_fps(
    level: h264_profile_level_id::Level,
    max_mbps: Option<u32>,
    max_fs: Option<u32>,
) -> ((u32, u32), u32) {
    use h264_profile_level_id::Level::*;

    // Default 30 fps, which is what the compositor produces
    let mut fps = 30;

    // Max macroblock processing rate MaxMBPS (MB/s)
    // from ITU-T H.264 (08/2021) Annex A.3.1 page 294 Table A-1 'Level limits'
    let mut max_macro_blocks_per_second = match level {
        Level1 => 1485,
        Level1b => 1485,
        Level11 => 3000,
        Level12 => 6000,
        Level13 => 11_880,
        Level2 => 11_880,
        Level21 => 19_800,
        Level22 => 20_250,
        Level3 => 40_500,
        Level31 => 108_000,
        Level32 => 216_000,
        Level4 => 245_760,
        Level41 => 245_760,
        Level42 => 522_240,
        Level5 => 589_824,
        Level51 => 983_040,
        Level52 => 2_073_600,
    };

    if let Some(max_mbps) = max_mbps {
        max_macro_blocks_per_second = max_mbps;
    }

    if let Some(max_fs @ 1..) = max_fs {
        fps = (max_macro_blocks_per_second / max_fs).max(1);
    }

    // 1 macroblock is 16x16 pixels
    let max_pixels = max_macro_blocks_per_second * 256 / fps;

    // Try to find the largest up to 1920x1080 (which is the magic number 30)
    for i in 1..30 {
        let width = i * 16 * 4;
        let height = i * 9 * 4;

        if width * height > max_pixels {
            return ((width - 4, height - 4), fps);
        }
    }

    // All resolutions are too small, fall back to 1080p
    ((1920, 1080), fps)
}
