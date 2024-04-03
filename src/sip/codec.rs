// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::codec::Codec;
use crate::settings::Settings;
use sdp_types::attributes::fmtp::Fmtp;
use sdp_types::attributes::rtpmap::RtpMap;
use sdp_types::msg::MediaScope;
use std::str::FromStr;

struct CodecEntry {
    static_payload: Option<u32>,
    name: &'static str,
    create: fn(rtpmap: Option<&RtpMap>, fmtp: Option<&Fmtp>) -> Option<SdpCodecInfo>,
}

/// List of all codecs and their constructor function sorted by priority
static AUDIO_CODEC_LIST: &[CodecEntry] = &[
    CodecEntry {
        static_payload: Some(9),
        name: "G722",
        create: g722_create,
    },
    CodecEntry {
        static_payload: Some(8),
        name: "PCMA",
        create: pcma_create,
    },
    CodecEntry {
        static_payload: Some(0),
        name: "PCMU",
        create: pcmu_create,
    },
];

fn g722_create(rtpmap: Option<&RtpMap>, _: Option<&Fmtp>) -> Option<SdpCodecInfo> {
    let rtpmap = rtpmap?.clone();

    Some(SdpCodecInfo {
        rtpmap,
        fmtp: None,
        gst_elements: GstCodecInfo {
            payload: 9,
            publish_raw_caps: gst::Caps::new_any(),
            codec_params: Vec::new(),
            codec: Codec::G722,
        },
    })
}

fn pcma_create(rtpmap: Option<&RtpMap>, _: Option<&Fmtp>) -> Option<SdpCodecInfo> {
    let rtpmap = rtpmap.cloned().unwrap_or_else(|| RtpMap {
        payload: 8,
        encoding: "PCMA".into(),
        clock_rate: 8000,
        params: None,
    });

    Some(SdpCodecInfo {
        rtpmap,
        fmtp: None,
        gst_elements: GstCodecInfo {
            payload: 8,
            publish_raw_caps: gst::Caps::new_any(),
            codec_params: Vec::new(),
            codec: Codec::G711PCMA,
        },
    })
}

fn pcmu_create(rtpmap: Option<&RtpMap>, _: Option<&Fmtp>) -> Option<SdpCodecInfo> {
    let rtpmap = rtpmap.cloned().unwrap_or_else(|| RtpMap {
        payload: 0,
        encoding: "PCMU".into(),
        clock_rate: 8000,
        params: None,
    });

    Some(SdpCodecInfo {
        rtpmap,
        fmtp: None,
        gst_elements: GstCodecInfo {
            payload: 0,
            publish_raw_caps: gst::Caps::new_any(),
            codec_params: Vec::new(),
            codec: Codec::G711PCMU,
        },
    })
}

pub struct SdpCodecInfo {
    pub rtpmap: RtpMap,
    pub fmtp: Option<Fmtp>,

    pub gst_elements: GstCodecInfo,
}

/// Information about the codec used in the gstreamer pipeline
///
/// Most of it are strings that name the elements which are used to encode & payload / unpack & decode.
#[derive(Clone)]
pub struct GstCodecInfo {
    /// RTP payload number to recognize the codec by
    pub payload: u32,

    /// Caps that limit the raw audio/video stream before converting, then encoding
    pub publish_raw_caps: gst::Caps,

    /// Parameters for gstreamer encode element
    pub codec_params: Vec<(String, String)>,

    pub codec: Codec,
}

/// Used by rtpbin's request-pt-map callback to provide caps
/// for incoming RTP packets using their payload number
#[derive(Clone)]
pub struct PayloadCaps {
    pub payload: u32,
    pub caps: gst::Caps,
}

pub fn choose_audio_codec(offer: &MediaScope) -> Option<SdpCodecInfo> {
    for entry in AUDIO_CODEC_LIST {
        // first check all payload numbers in the m= line
        for payload in &offer.desc.fmts {
            let static_pt_matches = if let Some(static_payload) = entry.static_payload {
                static_payload == *payload
            } else {
                false
            };

            if static_pt_matches {
                let rtpmap = offer.rtpmaps.iter().find(|x| x.payload == *payload);
                let fmtp = offer.fmtps.iter().find(|x| x.format == *payload);

                if let Some(info) = (entry.create)(rtpmap, fmtp) {
                    return Some(info);
                }
            }
        }

        // if no static payloads matches try matching by name
        for rtpmap in &offer.rtpmaps {
            if rtpmap.encoding.eq_ignore_ascii_case(entry.name) {
                let fmtp = offer.fmtps.iter().find(|x| x.format == rtpmap.payload);

                if let Some(info) = (entry.create)(Some(rtpmap), fmtp) {
                    return Some(info);
                }
            }
        }
    }

    None
}

pub fn available_video_codec() -> &'static str {
    "H264"
}

pub fn choose_video_codec(settings: &Settings, offer: &MediaScope) -> Option<SdpCodecInfo> {
    let rtpmap = offer
        .rtpmaps
        .iter()
        .find(|rtpmap| rtpmap.encoding == "H264" && rtpmap.clock_rate == 90000)?;

    let fmtp = offer
        .fmtps
        .iter()
        .find(|fmtp| fmtp.format == rtpmap.payload);

    let mut capsfilter = gst::Caps::new_any();
    let mut codec_params = vec![];

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

                    // openh264enc takes bitrate in bits/s, so multiply by 1000
                    codec_params.push(("bitrate".to_owned(), (value * 1000).to_string()));
                }
                _ => continue,
            }
        }

        if let Some(level) = level {
            let ((width, height), fps) =
                calculate_best_resolution_and_fps(level.level(), max_mbps, max_fs);

            let caps = format!("video/x-raw,width={width},height={height},framerate={fps}/1");

            match gst::Caps::from_str(&caps) {
                Ok(caps) => capsfilter = caps,
                Err(e) => log::warn!("Failed to parse caps '{caps}', {e}"),
            }
        }
    }

    Some(SdpCodecInfo {
        rtpmap: rtpmap.clone(),
        fmtp: None,
        gst_elements: GstCodecInfo {
            payload: rtpmap.payload,
            publish_raw_caps: capsfilter,
            codec_params,
            codec: Codec::H264,
        },
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

    // Common resolutions
    // from ITU-T H.264 (08/2021) Annex A.3.4 page 301 Table A-6 'Maximum frame rates'
    const ALLOWED_RESOLUTIONS: [(u32, u32); 34] = [
        (128, 96),
        (176, 144),
        (320, 240),
        (352, 240),
        (352, 288),
        (352, 480),
        (352, 576),
        (640, 480),
        (704, 480),
        (720, 480),
        (704, 576),
        (720, 576),
        (800, 600),
        (1024, 768),
        (1280, 720),
        (1280, 960),
        (1280, 1024),
        (1408, 960),
        (1408, 1152),
        (1600, 1200),
        (1920, 1088),
        (2048, 1024),
        (2048, 1088),
        (2048, 1536),
        (2560, 1920),
        (3616, 1536),
        (3680, 1536),
        (3840, 2160),
        (4096, 2048),
        (4096, 2160),
        (4096, 2304),
        (7680, 4320),
        (8192, 4096),
        (8192, 4320),
    ];

    // 1 macroblock is 16x16 pixels
    let max_pixels = max_macro_blocks_per_second * 256 / fps;

    for (width, height) in ALLOWED_RESOLUTIONS.into_iter().rev() {
        if width < height {
            continue;
        }

        if width * height <= max_pixels {
            return ((width, height), fps);
        }
    }

    // All resolutions are too large, fall back to the lowest and pray
    (ALLOWED_RESOLUTIONS[0], fps)
}
