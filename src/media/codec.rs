// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use gst::element_factory::ElementBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Audio,
    Video,
}

#[derive(Debug, Clone, Copy)]
pub enum Codec {
    G711PCMA,
    G711PCMU,
    G722,

    H264,
}

impl Codec {
    pub fn media_type(&self) -> MediaType {
        match self {
            Codec::G711PCMA | Codec::G711PCMU | Codec::G722 => MediaType::Audio,
            Codec::H264 => MediaType::Video,
        }
    }

    pub fn from_encoding_name(name: &str) -> Option<Self> {
        match name {
            "PCMA" => Some(Self::G711PCMA),
            "PCMU" => Some(Self::G711PCMU),
            "G722" => Some(Self::G722),
            "H264" => Some(Self::H264),
            _ => None,
        }
    }

    pub fn encode_element(&self) -> ElementBuilder<'static> {
        match self {
            Codec::G711PCMA => gst::ElementFactory::make("alawenc"),
            Codec::G711PCMU => gst::ElementFactory::make("mulawenc"),
            Codec::G722 => gst::ElementFactory::make("avenc_g722"),
            Codec::H264 => gst::ElementFactory::make("openh264enc")
                .property_from_str("usage-type", "camera")
                .property_from_str("rate-control", "bitrate")
                // TODO: Make this configurable by SDP
                .property("bitrate", 128_000u32),
        }
    }

    pub fn encoded_caps(&self) -> gst::Caps {
        match self {
            Codec::G711PCMA => gst::Caps::builder("audio/x-alaw").build(),
            Codec::G711PCMU => gst::Caps::builder("audio/x-mulaw").build(),
            Codec::G722 => gst::Caps::builder("audio/G722").build(),
            Codec::H264 => gst::Caps::builder("video/x-h264")
                .field("profile", "baseline")
                .field("stream-format", "byte-stream")
                .build(),
        }
    }

    pub fn decode_element(&self) -> &str {
        match self {
            Codec::G711PCMA => "alawdec",
            Codec::G711PCMU => "mulawdec",
            Codec::G722 => "avdec_g722",
            Codec::H264 => "avdec_h264",
        }
    }

    pub fn payload_element(&self) -> ElementBuilder<'static> {
        match self {
            Codec::G711PCMA => gst::ElementFactory::make("rtppcmapay"),
            Codec::G711PCMU => gst::ElementFactory::make("rtppcmupay"),
            Codec::G722 => gst::ElementFactory::make("rtpg722pay"),
            Codec::H264 => gst::ElementFactory::make("rtph264pay")
                .property_from_str("aggregate-mode", "zero-latency"),
        }
    }

    pub fn depayload_element(&self) -> &str {
        match self {
            Codec::G711PCMA => "rtppcmadepay",
            Codec::G711PCMU => "rtppcmudepay",
            Codec::G722 => "rtpg722depay",
            Codec::H264 => "rtph264depay",
        }
    }

    pub fn caps(&self) -> gst::Caps {
        match self {
            Codec::G711PCMA => gst::Caps::builder("application/x-rtp")
                .field("media", "audio")
                .field("encoding-name", "PCMA")
                .field("clock-rate", 8000i32)
                .build(),
            Codec::G711PCMU => gst::Caps::builder("application/x-rtp")
                .field("media", "audio")
                .field("encoding-name", "PCMU")
                .field("clock-rate", 8000i32)
                .build(),
            Codec::G722 => gst::Caps::builder("application/x-rtp")
                .field("media", "audio")
                .field("encoding-name", "G722")
                .field("clock-rate", 8000i32)
                .build(),
            Codec::H264 => gst::Caps::builder("application/x-rtp")
                .field("media", "video")
                .field("encoding-name", "H264")
                .field("clock-rate", 90_000i32)
                .build(),
        }
    }
}
