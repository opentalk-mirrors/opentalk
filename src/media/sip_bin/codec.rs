// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use sdp_types::attributes::fmtp::Fmtp;
use sdp_types::attributes::rtpmap::RtpMap;
use sdp_types::msg::MediaScope;

struct CodecEntry {
    static_payload: Option<u32>,
    name: &'static str,
    create: fn(rtpmap: Option<&RtpMap>, fmtp: Option<&Fmtp>) -> Option<SdpCodecInfo>,
}

/// List of all codecs and their constructor function sorted by priority
static CODEC_LIST: &[CodecEntry] = &[
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
            payload_caps: PayloadCaps {
                payload: 9,
                caps: gst::Caps::builder("application/x-rtp")
                    .field("media", "audio")
                    .field("encoding-name", "G722")
                    .field("clock-rate", 8000i32)
                    .build(),
            },
            rtppay: "rtpg722pay",
            rtpdepay: "rtpg722depay",
            encoder: "avenc_g722",
            decoder: "avdec_g722",
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
            payload_caps: PayloadCaps {
                payload: 8,
                caps: gst::Caps::builder("application/x-rtp")
                    .field("media", "audio")
                    .field("encoding-name", "PCMA")
                    .field("clock-rate", 8000i32)
                    .build(),
            },
            rtppay: "rtppcmapay",
            rtpdepay: "rtppcmadepay",
            encoder: "alawenc",
            decoder: "alawdec",
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
            payload_caps: PayloadCaps {
                payload: 0,
                caps: gst::Caps::builder("application/x-rtp")
                    .field("media", "audio")
                    .field("encoding-name", "PCMU")
                    .field("clock-rate", 8000i32)
                    .build(),
            },
            rtppay: "rtppcmupay",
            rtpdepay: "rtppcmudepay",
            encoder: "mulawenc",
            decoder: "mulawdec",
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
    pub payload_caps: PayloadCaps,

    pub rtppay: &'static str,
    pub rtpdepay: &'static str,

    pub encoder: &'static str,
    pub decoder: &'static str,
}

/// Used by rtpbin's request-pt-map callback to provide caps
/// for incoming RTP packets using their payload number
#[derive(Clone)]
pub struct PayloadCaps {
    pub payload: u32,
    pub caps: gst::Caps,
}

pub fn choose_codec(offer: &MediaScope) -> Option<SdpCodecInfo> {
    for entry in CODEC_LIST {
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
