// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::Result;
use bitstream_io::{BigEndian, BitRead, BitReader};
use gst::glib;
use gst::prelude::*;
use gst::subclass::{prelude::*, ElementMetadata};
use gst_rtp::subclass::prelude::*;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::io::Cursor;

glib::wrapper! {
    /// Largely mimics the implementation from gstreamer's built in rtpdtmfdepay element, without emitting a tone from
    /// it's src and slighty changing the detection of new DTMF events.
    pub(super) struct RTPDtmfDepay(ObjectSubclass<RTPDtmfDepayClass>)
        @extends gst_rtp::RTPBaseDepayload, gst::Element, gst::Object;
}

#[derive(Debug, Default)]
pub(super) struct RTPDtmfDepayClass {
    state: Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    last_timestamp: Option<u32>,
}

#[glib::object_subclass]
impl ObjectSubclass for RTPDtmfDepayClass {
    const NAME: &'static str = "GstRtpDtmfDepay";
    type Type = RTPDtmfDepay;
    type ParentType = gst_rtp::RTPBaseDepayload;
}

impl ObjectImpl for RTPDtmfDepayClass {}
impl GstObjectImpl for RTPDtmfDepayClass {}

impl ElementImpl for RTPDtmfDepayClass {
    fn metadata() -> Option<&'static ElementMetadata> {
        static ELEMENT_METADATA: Lazy<ElementMetadata> = Lazy::new(|| {
            ElementMetadata::new(
                "RTP DTMF Depayloader",
                "Codec/Depayloader/Network/RTP",
                "Depayload DTMF from RTP packets",
                "OpenTalk",
            )
        });

        Some(&ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<[gst::PadTemplate; 2]> = Lazy::new(|| {
            [
                gst::PadTemplate::new(
                    "sink",
                    gst::PadDirection::Sink,
                    gst::PadPresence::Always,
                    &gst::Caps::builder("application/x-rtp")
                        .field("media", "audio")
                        .field("payload", gst::IntRange::new(96, 127))
                        .field("clock-rate", gst::IntRange::new(0, i32::MAX))
                        .field("encoding-name", "TELEPHONE-EVENT")
                        .build(),
                )
                .expect("PadTemplate is valid"),
                // Offering a src which will never emit data to satisfy the RTPBaseDepayload base class
                gst::PadTemplate::new(
                    "src",
                    gst::PadDirection::Src,
                    gst::PadPresence::Always,
                    &gst::Caps::new_any(),
                )
                .expect("PadTemplate is valid"),
            ]
        });

        PAD_TEMPLATES.as_ref()
    }
}

impl RTPBaseDepayloadImpl for RTPDtmfDepayClass {
    fn process_rtp_packet(
        &self,
        rtp_buffer: &gst_rtp::RTPBuffer<gst_rtp::rtp_buffer::Readable>,
    ) -> Option<gst::Buffer> {
        if let Err(e) = self.try_process_rtp_packet(rtp_buffer) {
            log::warn!("Failed to process DTMF RTP packet, {:?}", e);
        }

        // Never push any data to `src`
        None
    }
}

impl RTPDtmfDepayClass {
    fn try_process_rtp_packet(
        &self,
        rtp_buffer: &gst_rtp::RTPBuffer<'_, gst_rtp::rtp_buffer::Readable>,
    ) -> Result<()> {
        let timestamp = rtp_buffer.timestamp();

        let mut reader = BitReader::endian(Cursor::new(rtp_buffer.payload()?), BigEndian);

        let event: u8 = reader.read(8)?;
        let _is_end: bool = reader.read_bit()?;
        reader.skip(1)?;
        let volume: u8 = reader.read(6)?;
        let _duration: u16 = reader.read(16)?;

        let mut state = self.state.lock();

        // Don't check the RTP Marker bit here, which is something the original rtpdtmfdepay does. Some devices send
        // multiple RTP packets with the bit set, which triggers multiple events from the same DTMF event.
        if state.last_timestamp < Some(timestamp) {
            state.last_timestamp = Some(timestamp);

            let message = gst::message::Element::new(
                gst::Structure::builder("dtmf-event")
                    .field("number", event as i32)
                    .field("volume", volume as i32)
                    .field("type", 1)
                    .field("method", 1)
                    .build(),
            );

            self.obj().post_message(message)?;
        }

        Ok(())
    }
}

pub fn init_custom_rtpdtmfdepay() -> Result<(), glib::BoolError> {
    gst::Element::register(
        None,
        "opentalk-rtpdtmfdepay",
        gst::Rank::Marginal,
        RTPDtmfDepay::static_type(),
    )
}
