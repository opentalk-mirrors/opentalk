// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::codec::Codec;
use sdp_types::attributes::fmtp::Fmtp;
use sdp_types::attributes::rtpmap::RtpMap;
use sdp_types::msg::MediaScope;

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
    pub payload: u32,

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

pub fn choose_video_codec(offer: &MediaScope) -> Option<SdpCodecInfo> {
    let rtpmap = offer
        .rtpmaps
        .iter()
        .find(|rtpmap| rtpmap.encoding == "H264" && rtpmap.clock_rate == 90000)?;

    Some(SdpCodecInfo {
        rtpmap: rtpmap.clone(),
        fmtp: None,
        gst_elements: GstCodecInfo {
            payload: rtpmap.payload,
            codec: Codec::H264,
        },
    })
}
