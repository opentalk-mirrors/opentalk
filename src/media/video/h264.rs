// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use bytes::Bytes;
use livekit::webrtc::video_frame::{I420Buffer, VideoFrame};
use openh264::{
    encoder::EncoderConfig,
    formats::{YUVSlices, YUVSource},
};
use opentalk_compositor::livekit::{self, webrtc::prelude::VideoRotation};
use rtp::{
    codecs::h264::{H264Packet, H264Payloader},
    packetizer::{Depacketizer, Payloader},
};
use std::time::Instant;

pub struct H264Encoder {
    encoder: openh264::encoder::Encoder,
    payloader: H264Payloader,
    init: Instant,
}

impl H264Encoder {
    pub fn create(bitrate: u32) -> Result<Self> {
        Ok(Self {
            encoder: openh264::encoder::Encoder::with_api_config(
                openh264::OpenH264API::from_source(),
                EncoderConfig::new().set_bitrate_bps(bitrate),
            )
            .context("Failed to create openh264 encoder")?,
            payloader: H264Payloader::default(),
            init: Instant::now(),
        })
    }

    pub fn encode_and_packetize(&mut self, yuv: &[u8], width: usize, height: usize) -> Vec<Bytes> {
        let (y, tmp) = yuv.split_at(width * height);
        let (u, v) = tmp.split_at((width * height) / 4);

        let slices = YUVSlices::new((y, u, v), (width, height), (width, width / 2, width / 2));

        let bitstream = match self.encoder.encode_at(
            &slices,
            openh264::Timestamp::from_millis(self.init.elapsed().as_millis() as u64),
        ) {
            Ok(bitstream) => bitstream.to_vec(),
            Err(e) => {
                log::error!("Failed to encode video frame, {e}");
                return vec![];
            }
        };

        match self.payloader.payload(1400, &Bytes::from(bitstream)) {
            Ok(packets) => packets,
            Err(e) => {
                log::error!("Failed to payload openh264 bitstream, {e}");
                vec![]
            }
        }
    }
}

pub struct H264Decoder {
    encoder: openh264::decoder::Decoder,
    init: Instant,
}

impl H264Decoder {
    pub fn create() -> Result<Self> {
        Ok(Self {
            encoder: openh264::decoder::Decoder::new()
                .context("Failed to create openh264 decoder")?,
            init: Instant::now(),
        })
    }

    pub fn decode_packet(&mut self, packet: &[u8]) -> Option<VideoFrame<I420Buffer>> {
        let packet = Bytes::copy_from_slice(packet);
        let packet = H264Packet::default().depacketize(&packet).ok()?;
        let decoded_yuv = self.encoder.decode(&packet).ok()??;

        let (width, height) = decoded_yuv.dimensions();

        let (stride_y, stride_u, stride_v) = decoded_yuv.strides();

        let mut buffer = I420Buffer::with_strides(
            width as u32,
            height as u32,
            stride_y as u32,
            stride_u as u32,
            stride_v as u32,
        );

        buffer.data_mut().0.copy_from_slice(decoded_yuv.y());
        buffer.data_mut().1.copy_from_slice(decoded_yuv.u());
        buffer.data_mut().2.copy_from_slice(decoded_yuv.v());

        Some(VideoFrame {
            rotation: VideoRotation::VideoRotation0,
            timestamp_us: self.init.elapsed().as_micros() as i64,
            buffer,
        })
    }
}
