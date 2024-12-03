// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::{codec::Codec, DEFAULT_FPS};
use crate::settings::Settings;
use h264_profile_level_id::{Level, Profile};
use openh264::encoder::{Level as OpenH264Level, Profile as OpenH264Profile};
use sdp_types::{MediaDescription, RtpMap};
use std::str::FromStr;

/// Describes the codec used to receive audio/video
#[derive(Debug, Clone)]
pub struct SdpRecvCodec {
    pub payload: u8,
    pub codec: Codec,
    pub fmtp: Option<String>,
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
        fps: u32,
        level: OpenH264Level,
        profile: OpenH264Profile,
        packetization_mode: H264PacketizationMode,
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
            fmtp: None,
        },
        SdpRecvCodec {
            payload: 8,
            codec: Codec::G711PCMA,
            fmtp: None,
        },
        SdpRecvCodec {
            payload: 0,
            codec: Codec::G711PCMU,
            fmtp: None,
        },
    ]
}

pub fn offer_all_video_codecs() -> Vec<SdpRecvCodec> {
    vec![SdpRecvCodec {
        payload: 96,
        codec: Codec::H264,
        fmtp: Some("profile-level-id=420028;max-mbps=243000;max-fs=8100;level-asymmetry-allowed=1;packetization-mode=0".into()),
    }]
}

pub fn available_video_codec() -> &'static str {
    "H264"
}

#[derive(Debug, Clone, Copy)]
pub enum H264PacketizationMode {
    // https://www.rfc-editor.org/rfc/rfc6184#section-6.2
    SingleNal,
    // https://www.rfc-editor.org/rfc/rfc6184#section-6.3
    NonInterleavedMode,
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
        let mut packetization_mode = H264PacketizationMode::SingleNal;

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
                "packetization-mode" => match value {
                    0 => packetization_mode = H264PacketizationMode::SingleNal,
                    1 => packetization_mode = H264PacketizationMode::NonInterleavedMode,
                    _ => log::warn!("Unhandled packetization mode {value}"),
                },
                _ => continue,
            }
        }

        if let Some(level) = level {
            // Calculate pre encode caps
            let ((width, height), fps) =
                calculate_best_resolution_and_fps(level.level(), max_mbps, max_fs);

            // Calculate post encode caps
            let profile = match level.profile() {
                Profile::Baseline => OpenH264Profile::Baseline,
                Profile::Main => OpenH264Profile::Main,
                Profile::High => OpenH264Profile::High,
                Profile::ConstrainedBaseline
                | Profile::ConstrainedHigh
                | Profile::PredictiveHigh444 => {
                    log::warn!(
                        "Client requested unsupported H.264 profile {:?}, falling back to baseline",
                        level.profile()
                    );
                    OpenH264Profile::Baseline
                }
            };

            let level = match level.level() {
                Level::Level1b => OpenH264Level::Level_1_B,
                Level::Level1 => OpenH264Level::Level_1_0,
                Level::Level11 => OpenH264Level::Level_1_1,
                Level::Level12 => OpenH264Level::Level_1_2,
                Level::Level13 => OpenH264Level::Level_1_3,
                Level::Level2 => OpenH264Level::Level_2_0,
                Level::Level21 => OpenH264Level::Level_2_1,
                Level::Level22 => OpenH264Level::Level_2_2,
                Level::Level3 => OpenH264Level::Level_3_0,
                Level::Level31 => OpenH264Level::Level_3_1,
                Level::Level32 => OpenH264Level::Level_3_2,
                Level::Level4 => OpenH264Level::Level_4_0,
                Level::Level41 => OpenH264Level::Level_4_1,
                Level::Level42 => OpenH264Level::Level_4_2,
                Level::Level5 => OpenH264Level::Level_5_0,
                Level::Level51 => OpenH264Level::Level_5_1,
                Level::Level52 => OpenH264Level::Level_5_2,
            };

            codec_params = SdpSendCodecParams::H264 {
                width,
                height,
                fps,
                level,
                profile,
                packetization_mode,
            }
        }
    }

    for bandwidth in &offer.bandwidth {
        match bandwidth.type_.as_str() {
            // The TIAS Bandwidth Modifier
            // https://www.rfc-editor.org/rfc/rfc3890#section-6.2
            "TIAS" => bitrate = Some(bandwidth.bandwidth),
            _ => continue,
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
    let mut fps = DEFAULT_FPS;

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
            let Some(previous_i) = i.checked_sub(1) else {
                // Check failed in the first iteration, fall back to default
                break;
            };

            let previous_width = previous_i * 16 * 4;
            let previous_height = previous_i * 9 * 4;

            return ((previous_width, previous_height), fps);
        }
    }

    // All resolutions are too small, fall back to 1080p
    ((1920, 1080), fps)
}
