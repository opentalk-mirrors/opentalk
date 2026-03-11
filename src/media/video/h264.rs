// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::sip::H264PacketizationMode;
use anyhow::{Context, Result};
use bytes::Bytes;
use livekit::webrtc::video_frame::{I420Buffer, VideoFrame};
use openh264::{
    OpenH264API,
    encoder::{
        BitRate, Encoder, EncoderConfig, FrameRate, IntraFramePeriod, Level, Profile,
        RateControlMode,
    },
    formats::{YUVSlices, YUVSource},
    nal_units,
};
use opentalk_compositor::livekit::{self, webrtc::prelude::VideoRotation};
use rtp::{
    codecs::h264::{H264Packet, H264Payloader},
    packetizer::{Depacketizer, Payloader},
};
use std::time::Instant;

const MTU: u32 = 1200;

pub struct H264Encoder {
    encoder: openh264::encoder::Encoder,
    payloader: H264Payloader,
    init: Instant,
    packetization_mode: H264PacketizationMode,
}

impl H264Encoder {
    pub fn create(
        packetization_mode: H264PacketizationMode,
        bitrate: u32,
        profile: Option<Profile>,
        level: Option<Level>,
        fps: u32,
    ) -> Result<Self> {
        #[expect(clippy::cast_precision_loss)]
        let mut encoder_config = EncoderConfig::new()
            .bitrate(BitRate::from_bps(bitrate))
            .intra_frame_period(IntraFramePeriod::from_num_frames(90))
            .rate_control_mode(RateControlMode::Bitrate)
            .max_frame_rate(FrameRate::from_hz(fps as f32));

        match packetization_mode {
            H264PacketizationMode::SingleNal => encoder_config = encoder_config.max_slice_len(MTU),
            H264PacketizationMode::NonInterleavedMode => {}
        }

        if let Some(profile) = profile {
            encoder_config = encoder_config.profile(profile);
        }

        if let Some(level) = level {
            encoder_config = encoder_config.level(level);
        }

        let encoder = Encoder::with_api_config(OpenH264API::from_source(), encoder_config)
            .context("Failed to create openh264 encoder")?;

        Ok(Self {
            encoder,
            payloader: H264Payloader::default(),
            init: Instant::now(),
            packetization_mode,
        })
    }

    pub fn encode_and_packetize(
        &mut self,
        yuv: &[u8],
        width: usize,
        height: usize,
        force_intra_frame: bool,
    ) -> Vec<Bytes> {
        let (y, tmp) = yuv.split_at(width * height);
        let (u, v) = tmp.split_at((width * height) / 4);

        let slices = YUVSlices::new((y, u, v), (width, height), (width, width / 2, width / 2));

        if force_intra_frame {
            self.encoder.force_intra_frame();
        }

        let bitstream = match self.encoder.encode_at(
            &slices,
            openh264::Timestamp::from_millis(
                u64::try_from(self.init.elapsed().as_millis())
                    .expect("conversion to u64 not possible"),
            ),
        ) {
            Ok(bitstream) => bitstream.to_vec(),
            Err(e) => {
                log::error!("Failed to encode video frame, {e}");
                return vec![];
            }
        };

        match self.packetization_mode {
            H264PacketizationMode::SingleNal => nal_units(&bitstream)
                .map(|nalu| Bytes::copy_from_slice(&nalu[3..]))
                .collect(),
            H264PacketizationMode::NonInterleavedMode => {
                match self
                    .payloader
                    .payload(MTU as usize, &Bytes::from(bitstream))
                {
                    Ok(packets) => packets,
                    Err(e) => {
                        log::error!("Failed to payload openh264 bitstream, {e}");
                        vec![]
                    }
                }
            }
        }
    }
}

pub struct H264Decoder {
    depayloader: H264Packet,
    decoder: openh264::decoder::Decoder,
    init: Instant,
}

impl H264Decoder {
    pub fn create() -> Result<Self> {
        Ok(Self {
            depayloader: H264Packet::default(),
            decoder: openh264::decoder::Decoder::new()
                .context("Failed to create openh264 decoder")?,
            init: Instant::now(),
        })
    }

    pub fn decode_packet(&mut self, packet: &[u8]) -> Option<Result<VideoFrame<I420Buffer>, ()>> {
        let packet = Bytes::copy_from_slice(packet);

        let packet = match self.depayloader.depacketize(&packet) {
            Ok(packet) => packet,
            Err(e) => {
                log::warn!("Failed to depacketize H.264, {e}");
                // Reset depayloader on error, may be caused by packet loss
                self.depayloader = H264Packet::default();
                return None;
            }
        };

        // Depayloader returns empty bytes when decoding a FU-A fragment, so skip for now
        if packet.is_empty() {
            return None;
        }

        let decoded_yuv = match self.decoder.decode(&packet) {
            Ok(decoded_yuv) => decoded_yuv?,
            Err(e) => {
                log::warn!("Failed to decode packet: {e:?}");
                return Some(Err(()));
            }
        };

        let (width, height) = decoded_yuv.dimensions();

        let (stride_y, stride_u, stride_v) = decoded_yuv.strides();

        let mut buffer = I420Buffer::with_strides(
            u32::try_from(width).expect("conversion to u32 not possible"),
            u32::try_from(height).expect("conversion to u32 not possible"),
            u32::try_from(stride_y).expect("conversion to u32 not possible"),
            u32::try_from(stride_u).expect("conversion to u32 not possible"),
            u32::try_from(stride_v).expect("conversion to u32 not possible"),
        );

        buffer.data_mut().0.copy_from_slice(decoded_yuv.y());
        buffer.data_mut().1.copy_from_slice(decoded_yuv.u());
        buffer.data_mut().2.copy_from_slice(decoded_yuv.v());

        Some(Ok(VideoFrame {
            rotation: VideoRotation::VideoRotation0,
            timestamp_us: i64::try_from(self.init.elapsed().as_micros())
                .expect("conversion to i64 not possible"),
            buffer,
        }))
    }
}
