// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use sdp_types::MediaType as SdpMediaType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Audio,
    Video,
}

impl From<MediaType> for SdpMediaType {
    fn from(val: MediaType) -> Self {
        match val {
            MediaType::Audio => SdpMediaType::Audio,
            MediaType::Video => SdpMediaType::Video,
        }
    }
}

impl PartialEq<SdpMediaType> for MediaType {
    fn eq(&self, other: &SdpMediaType) -> bool {
        SdpMediaType::from(*self) == *other
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Codec {
    G711PCMA,
    G711PCMU,
    G722,

    H264,
}

impl Codec {
    pub fn media_type(self) -> MediaType {
        match self {
            Codec::G711PCMA | Codec::G711PCMU | Codec::G722 => MediaType::Audio,
            Codec::H264 => MediaType::Video,
        }
    }

    pub fn encoding_name(self) -> &'static str {
        match self {
            Codec::G711PCMA => "PCMA",
            Codec::G711PCMU => "PCMU",
            Codec::G722 => "G722",
            Codec::H264 => "H264",
        }
    }

    pub fn clock_rate(self) -> u32 {
        match self {
            Codec::G711PCMA | Codec::G711PCMU | Codec::G722 => 8000,
            Codec::H264 => 90_000,
        }
    }
}
