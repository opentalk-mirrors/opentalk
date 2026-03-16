// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::{DEFAULT_FPS, codec::Codec};
use crate::settings::Settings;
use h264_profile_level_id::{Level, Profile};
use openh264::OpenH264API;
use openh264::encoder::{Level as OpenH264Level, Profile as OpenH264Profile};
use openh264_sys2::API;
use sdp_types::{ImageAttrSets, ImageAttrXyRange, MediaDescription, RtpMap};
use std::cmp;
use std::mem::MaybeUninit;
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
    vec![
        SdpRecvCodec {
            payload: 96,
            codec: Codec::H264,
            fmtp: Some(openh264_decoder_fmtp(H264PacketizationMode::SingleNal)),
        },
        SdpRecvCodec {
            payload: 97,
            codec: Codec::H264,
            fmtp: Some(openh264_decoder_fmtp(
                H264PacketizationMode::NonInterleavedMode,
            )),
        },
    ]
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

pub fn choose_video_codec(
    settings: &Settings,
    offer: &MediaDescription,
) -> Option<(SdpSendCodec, SdpRecvCodec)> {
    'next_pt: for rtpmap in offer
        .rtpmap
        .iter()
        .filter(|rtpmap| rtpmap.encoding == "H264" && rtpmap.clock_rate == 90000)
    {
        let fmtp = offer.fmtp.iter().find(|fmtp| fmtp.format == rtpmap.payload);

        let imageattr = offer
            .imageattr
            .iter()
            .find(|imageattr| imageattr.pt.is_none_or(|pt| pt == rtpmap.payload));

        let mut codec_params = SdpSendCodecParams::None;
        let mut bitrate = None;
        let mut packetization_mode = H264PacketizationMode::SingleNal;

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
                        Err(e) => {
                            log::warn!("Failed to parse profile-level-id in H.264 params, {e}");
                            continue 'next_pt;
                        }
                    }

                    continue;
                }

                let Ok(value) = value.parse::<u32>() else {
                    log::warn!("Failed to parse value for '{value}' in H.264 params");
                    continue 'next_pt;
                };

                match key {
                    "max-mbps" => max_mbps = Some(value),
                    "max-fs" => max_fs = Some(value),
                    "max-br" => {
                        // max-br is in kbit/s, convert to to bits/s
                        let value = (value * 1000).min(settings.sip.max_video_bitrate);

                        bitrate = Some(value);
                    }
                    "packetization-mode" => match value {
                        0 => packetization_mode = H264PacketizationMode::SingleNal,
                        1 => packetization_mode = H264PacketizationMode::NonInterleavedMode,
                        _ => {
                            log::warn!("Unhandled packetization mode {value}");
                            continue 'next_pt;
                        }
                    },
                    _ => {}
                }
            }

            if let Some(level) = level {
                // Calculate pre encode caps
                let ((mut width, mut height), fps) =
                    calculate_best_resolution_and_fps(level.level(), max_mbps, max_fs);

                // Clamp resolution if imageattr is set
                if let Some(imageattr) = imageattr {
                    clamp_resolution_by_imageattr(imageattr, &mut width, &mut height);
                }

                // Calculate post encode caps
                let profile = match level.profile() {
                    Profile::ConstrainedBaseline | Profile::Baseline => OpenH264Profile::Baseline,
                    Profile::Main => OpenH264Profile::Main,
                    Profile::High | Profile::ConstrainedHigh => OpenH264Profile::High,
                    Profile::PredictiveHigh444 => {
                        continue 'next_pt;
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
            if bandwidth.type_.as_str() == "TIAS" {
                bitrate = Some(bandwidth.bandwidth);
            }
        }

        return Some((
            SdpSendCodec {
                payload: rtpmap.payload,
                codec: Codec::H264,
                codec_params,
                bitrate,
            },
            SdpRecvCodec {
                payload: rtpmap.payload,
                codec: Codec::H264,
                fmtp: Some(openh264_decoder_fmtp(packetization_mode)),
            },
        ));
    }

    None
}

fn clamp_resolution_by_imageattr(
    imageattr: &sdp_types::ImageAttr,
    width: &mut u32,
    height: &mut u32,
) {
    let Some(ImageAttrSets::Sets(sets)) = &imageattr.recv else {
        return;
    };

    // Find highest weighted set (by `q` parameter)
    let Some(set) = sets.iter().max_by(|set_a, set_b| {
        let a = set_a.q.unwrap_or(0.5);
        let b = set_b.q.unwrap_or(0.5);
        a.total_cmp(&b)
    }) else {
        return;
    };

    clamp_by_range(width, &set.x);
    clamp_by_range(height, &set.y);
}

fn clamp_by_range(value: &mut u32, range: &ImageAttrXyRange) {
    match range {
        ImageAttrXyRange::Range {
            lower,
            upper,
            step: _,
        } => *value = (*value).clamp(*lower, *upper),
        ImageAttrXyRange::List(items) => {
            if let Some(item) = items.first() {
                *value = cmp::min(*value, *item);
            }
        }
        ImageAttrXyRange::Value(v) => *value = cmp::min(*value, *v),
    }
}

fn calculate_best_resolution_and_fps(
    level: h264_profile_level_id::Level,
    max_mbps: Option<u32>,
    max_fs: Option<u32>,
) -> ((u32, u32), u32) {
    use h264_profile_level_id::Level::{
        Level1, Level1b, Level2, Level3, Level4, Level5, Level11, Level12, Level13, Level21,
        Level22, Level31, Level32, Level41, Level42, Level51, Level52,
    };

    // Default 30 fps, which is what the compositor produces
    let mut fps = DEFAULT_FPS;

    // Max macroblock processing rate MaxMBPS (MB/s)
    // from ITU-T H.264 (08/2021) Annex A.3.1 page 294 Table A-1 'Level limits'
    let mut max_macro_blocks_per_second = match level {
        Level1 | Level1b => 1485,
        Level11 => 3000,
        Level12 => 6000,
        Level13 | Level2 => 11_880,
        Level21 => 19_800,
        Level22 => 20_250,
        Level3 => 40_500,
        Level31 => 108_000,
        Level32 => 216_000,
        Level4 | Level41 => 245_760,
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

pub fn openh264_decoder_fmtp(packetization_mode: H264PacketizationMode) -> String {
    let openh264_sys2::TagDecoderCapability {
        iProfileIdc,
        iProfileIop,
        iLevelIdc,
        iMaxMbps,
        iMaxFs,
        iMaxCpb,
        iMaxDpb,
        iMaxBr,
        bRedPicCap: _,
    } = unsafe {
        let mut capability = MaybeUninit::uninit();

        assert_eq!(
            OpenH264API::from_source().WelsGetDecoderCapability(capability.as_mut_ptr()),
            0,
            "openh264 WelsGetDecoderCapability failed"
        );

        capability.assume_init()
    };

    let packetization_mode = match packetization_mode {
        H264PacketizationMode::SingleNal => 0,
        H264PacketizationMode::NonInterleavedMode => 1,
    };

    format!(
        "profile-level-id={iProfileIdc:02X}{iProfileIop:02X}{iLevelIdc:02X};\
        level-asymmetry-allowed=1;\
        packetization-mode={packetization_mode};\
        max-mbps={iMaxMbps};\
        max-fs={iMaxFs};\
        max-cpb={iMaxCpb};\
        max-dpb={iMaxDpb};\
        max-br={iMaxBr}",
    )
}
