// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::port_pool::PortPool;
use anyhow::{Context, Result};
use bytes::Bytes;
use bytesstr::BytesStr;
use codec::{GstCodecInfo, PayloadCaps};
use gio::prelude::*;
use gst::glib;
use gst::prelude::ObjectExt;
use gst::prelude::*;
use sdp::SessionInfo;
use sdp_types::attributes::direction::Direction;
use std::net::{IpAddr, SocketAddr};
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

mod codec;
mod sdp;

pub struct SipBin {
    pub(crate) bin: gst::Bin,

    id: u64,
    version: u64,

    local_rtp_addr: SocketAddr,
    local_rtcp_addr: SocketAddr,

    current_session_info: SessionInfo,
}

impl SipBin {
    /// Create a new SipBin based on a public [IpAddr] and an SDP offer
    ///
    /// Usually called by the first SIP INVITE.
    pub fn new(pub_addr: IpAddr, offer: Bytes) -> Result<(SipBin, String)> {
        // Parse SDP offer
        let offer = BytesStr::from_utf8_bytes(offer)?;
        let offer = sdp_types::msg::parse::<sdp_types::msg::Builder>(&offer)?;

        // Create RTP/RTCP UDP socket
        let port_pool = PortPool::instance();

        let socket_pair = port_pool
            .create_rtp_socket_pair()
            .context("Failed to create RTP/RTCP sockets")?
            .context("No more RTP/RTCP ports are available")?;

        let local_rtp_addr = SocketAddr::new(pub_addr, socket_pair.rtp_port);
        let local_rtcp_addr = SocketAddr::new(pub_addr, socket_pair.rtcp_port);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = now;
        let version = now + 1;

        let local_info = sdp::LocalInfo {
            offer,
            id,
            version,
            rtp_addr: local_rtp_addr,
            rtcp_addr: local_rtcp_addr,
        };

        let current_session_info = sdp::respond(local_info)?;
        let bin = create_sip_bin(
            socket_pair.rtp_socket,
            socket_pair.rtcp_socket,
            current_session_info.telephone_event_pt,
            current_session_info.gst_elements.clone(),
        )
        .context("Failed to create sip_bin")?;

        let mut this = Self {
            bin,
            id,
            version,
            local_rtp_addr,
            local_rtcp_addr,
            current_session_info,
        };

        let send_data = matches!(
            this.current_session_info.local_sdp.media_scopes[0].direction,
            Direction::SendRecv | Direction::SendOnly
        );

        // When remote sends 0.0.0.0 as media address it does not want to receive ANY RTP traffic,
        // regardless of stream direction
        // Thats why we don't set the destination to the remote address and instead send them all to
        // the default 127.0.0.1
        if !this.current_session_info.rtp_addr.ip().is_unspecified() {
            this.set_destination(
                this.current_session_info.rtp_addr,
                this.current_session_info.rtcp_addr,
            )
            .context("Failed to set destination during creation")?;
        }

        // Close the audio data valve if no data is supposed to be sent
        if !send_data {
            this.set_hold()
                .context("Failed to set hold during creation")?;
        }

        let answer = this.current_session_info.local_sdp.to_string();

        Ok((this, answer))
    }

    /// Update the SipBin based on a new SDP offer
    ///
    /// Usually called by a SIP ReINVITE.
    pub fn update(&mut self, offer: Bytes) -> Result<String> {
        // Parse SDP offer
        let offer = BytesStr::from_utf8_bytes(offer)?;
        let offer = sdp_types::msg::parse::<sdp_types::msg::Builder>(&offer)?;

        self.version += 1;

        let local_info = sdp::LocalInfo {
            offer,
            id: self.id,
            version: self.version,
            rtp_addr: self.local_rtp_addr,
            rtcp_addr: self.local_rtcp_addr,
        };

        let session_info = sdp::respond(local_info)?;

        let send_data = matches!(
            session_info.local_sdp.media_scopes[0].direction,
            Direction::SendRecv | Direction::SendOnly
        );

        // check if version of sdp::respond is larger than current version.
        if session_info.remote_version > self.current_session_info.remote_version {
            // on unspecified ip (0.0.0.0) stop sending data anywhere, else set destination to sdp addresses
            // See https://datatracker.ietf.org/doc/html/rfc3264#section-8.4
            // and https://datatracker.ietf.org/doc/html/rfc2543#appendix-B.5
            if session_info.rtp_addr.ip().is_unspecified() {
                self.set_destination(([127, 0, 0, 1], 9).into(), ([127, 0, 0, 1], 9).into())?;
            } else if session_info.rtp_addr != self.current_session_info.rtp_addr
                || session_info.rtcp_addr != self.current_session_info.rtcp_addr
            {
                // Update destination in udp-sinks
                self.set_destination(session_info.rtp_addr, session_info.rtcp_addr)?;
            }

            if send_data {
                self.set_unhold()
                    .context("Failed to set unhold during update")?;
            } else {
                self.set_hold()
                    .context("Failed to set hold during update")?;
            }
        }

        self.current_session_info = session_info;

        Ok(self.current_session_info.local_sdp.to_string())
    }

    fn set_destination(
        &mut self,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<()> {
        log::trace!(
            "Setting destination to {} and {} for {}",
            remote_rtp_addr,
            remote_rtcp_addr,
            self.id,
        );

        let rtp_udp_sink = self
            .bin
            .by_name("rtp-udpsink")
            .context("failed to get rtp-udpsink from sip_bin")?;

        let rtcp_udp_sink = self
            .bin
            .by_name("rtcp-udpsink")
            .context("failed to get rtcp-udpsink from sip_bin")?;

        rtp_udp_sink.set_property("host", remote_rtp_addr.ip().to_string());
        rtp_udp_sink.set_property("port", remote_rtp_addr.port() as i32);

        rtcp_udp_sink.set_property("host", remote_rtcp_addr.ip().to_string());
        rtcp_udp_sink.set_property("port", remote_rtcp_addr.port() as i32);

        Ok(())
    }

    /// Unlinks the udp sockets from the rtp and rtcp srcs and links them to a fakesink
    fn set_hold(&mut self) -> Result<()> {
        log::trace!("Holding call for {}", self.id);

        let rtp_valve = self
            .bin
            .by_name("audio-input-valve")
            .context("failed to get audio-input-valve")?;

        rtp_valve.set_property("drop", true);

        Ok(())
    }

    /// Unlinks the fakesink, updates the udp socket destinations and links udp sockets back into the pipeline
    fn set_unhold(&mut self) -> Result<()> {
        log::trace!("Unholding call for {}", self.id);

        let rtp_valve = self
            .bin
            .by_name("audio-input-valve")
            .context("failed to get audio-input-valve")?;

        rtp_valve.set_property("drop", false);

        Ok(())
    }
}

/// Creates a gst sip-bin
///
/// The default state has:
/// * No udp destination port set.
/// *
fn create_sip_bin(
    rtp_socket: gio::Socket,
    rtcp_socket: gio::Socket,
    telephone_event_pt: u32,
    gst_elements: GstCodecInfo,
) -> Result<gst::Bin> {
    // Create pipeline textual description
    let sip_bin = format!(
        r#"
        rtpbin name=session

        queue name=audio-sink min-threshold-time=20000000 ! clocksync sync=true ! valve name=audio-input-valve ! audioconvert ! audioresample  ! {encoder} ! {rtppay} ! session.send_rtp_sink_0

        udpsrc name=rtp-udp-src ! queue ! watchdog name=rtp-watchdog timeout=10000 ! session.recv_rtp_sink_0
        udpsrc name=rtcp-udp-src caps="application/x-rtcp" ! queue ! session.recv_rtcp_sink_0

        session.send_rtp_src_0 ! udpsink host=127.0.0.1 port=9 name=rtp-udpsink
        session.send_rtcp_src_0 ! udpsink host=127.0.0.1 port=9 name=rtcp-udpsink
    
        {rtpdepay} name=audiodepay ! {decoder} ! queue name=audio-src

        "#,
        encoder = gst_elements.encoder,
        decoder = gst_elements.decoder,
        rtppay = gst_elements.rtppay,
        rtpdepay = gst_elements.rtpdepay,
    );

    // Parse pipeline
    let sip_bin = gst::parse_bin_from_description_with_name(&sip_bin, false, "sip-bin")?;

    // ==== CREATE AUDIO SINK GHOSTPAD (as in audio to be sent via RTP)

    let audio_sink = sip_bin
        .by_name("audio-sink")
        .context("failed to get audio-sink")?;
    let audio_sink_sink_pad = audio_sink
        .static_pad("sink")
        .context("failed to get sink pad")?;

    let audio_sink_ghost_pad =
        gst::GhostPad::with_target(Some("sip-audio-sink"), &audio_sink_sink_pad)?;
    sip_bin.add_pad(&audio_sink_ghost_pad)?;

    // ==== CREATE AUDIO SRC GHOSTPAD (as in received audio from RTP)

    // Create outgoing audio ghost pad
    let audio_src = sip_bin
        .by_name("audio-src")
        .context("failed to get audio-src")?;

    let audio_src_src_pad = audio_src
        .static_pad("src")
        .context("failed to get src pad")?;

    // TODO(r.floren) Rename this to src, to follow the general gstreamer eco system.
    let audio_src_ghost_pad = gst::GhostPad::new(Some("sip-audio-src"), gst::PadDirection::Src);
    sip_bin.add_pad(&audio_src_ghost_pad)?;

    // Connect outgoing audio to fakesink to not interrupt pipeline flow
    let audio_fakesink = gst::ElementFactory::make("fakesink")
        .name("sip-audio-fakesink")
        .build()?;

    sip_bin.add(&audio_fakesink)?;
    let audio_fakesink_sink_pad = audio_fakesink.static_pad("sink").unwrap();
    audio_src_src_pad.link(&audio_fakesink_sink_pad)?;

    // Create 'linked' callback to ghostpad.
    // Once the ghostpad was linked, remove the fakesink
    // and connect audio to the ghostpad instead
    let sip_bin_clone = sip_bin.clone();
    audio_src_ghost_pad
        .clone()
        .connect("linked", true, move |_values| {
            if let Err(e) = || -> Result<()> {
                audio_src_src_pad.unlink(&audio_fakesink_sink_pad)?;
                sip_bin_clone.remove(&audio_fakesink)?;
                audio_src_ghost_pad.set_target(Some(&audio_src_src_pad))?;
                Ok(())
            }() {
                log::error!("failed to handle linked ghostpad {}", e);
            }

            None
        });

    // Set sockets of udp_(src/sink)
    let rtp_udp_src = sip_bin.by_name("rtp-udp-src").unwrap();
    let rtp_udp_sink = sip_bin.by_name("rtp-udpsink").unwrap();

    let rtcp_udp_src = sip_bin.by_name("rtcp-udp-src").unwrap();
    let rtcp_udp_sink = sip_bin.by_name("rtcp-udpsink").unwrap();

    rtp_udp_src.set_property("socket", rtp_socket.clone());
    rtp_udp_sink.set_property("socket", rtp_socket);

    rtcp_udp_src.set_property("socket", rtcp_socket.clone());
    rtcp_udp_sink.set_property("socket", rtcp_socket);

    // Get the rtpbin element
    let rtp_bin = sip_bin.by_name("session").unwrap();

    rtp_bin.connect(
        "pad-added",
        true,
        on_pad_added(
            sip_bin.clone(),
            gst_elements.payload_caps.payload,
            telephone_event_pt,
        ),
    );

    rtp_bin.connect(
        "request-pt-map",
        false,
        on_request_pt_map(gst_elements.payload_caps, telephone_event_pt),
    );

    Ok(sip_bin)
}

impl Drop for SipBin {
    fn drop(&mut self) {
        let port_pool = PortPool::instance();

        port_pool.clear_port(self.local_rtp_addr.port())
    }
}

/// Callback function whenever the rtpbin encounters a new payload inside the RTP stream.
///
/// Provides gst::Caps for the given payload number, so gstreamer can properly handle
/// the payload.
///
/// Provides the caps of `payload_caps.caps` if payload is equal to payload_caps.payload`
/// and telephone event caps if `payload == telephone_event_pt`
fn on_request_pt_map(
    payload_caps: PayloadCaps,
    telephone_event_pt: u32,
) -> impl Fn(&[glib::Value]) -> Option<glib::Value> {
    move |values| {
        let pt = values[2].get::<u32>().unwrap();

        let caps = if pt == payload_caps.payload {
            payload_caps.caps.clone()
        } else if pt == telephone_event_pt {
            gst::Caps::builder("application/x-rtp")
                .field("media", "audio")
                .field("encoding-name", "TELEPHONE-EVENT")
                .field("clock-rate", 8000)
                .build()
        } else {
            log::warn!("requested caps for unknown pt {}", pt);

            gst::Caps::new_empty()
        };

        Some(caps.to_value())
    }
}

fn on_pad_added(
    sip_bin: gst::Bin,
    expected_media_pt: u32,
    telephone_event_pt: u32,
) -> impl Fn(&[glib::Value]) -> Option<glib::Value> {
    move |values| {
        if let Err(e) = try_on_pad_added(&sip_bin, expected_media_pt, telephone_event_pt, values) {
            log::error!("failed to handle added pad, {}", e);
        }

        None
    }
}

fn try_on_pad_added(
    sip_bin: &gst::Bin,
    expected_media_pt: u32,
    telephone_event_pt: u32,
    values: &[glib::Value],
) -> Result<()> {
    // First argument of the 'pad-added' must always be the added pad
    let pad = values[1].get::<gst::Pad>()?;

    // Get the pads capabilities, in it search for the payload of the incoming stream
    let caps = pad.caps().context("no caps on pad, cannot link")?;
    let caps = caps
        .structure(0)
        .context("caps does not contain any capabilities")?;

    // Extract payload number from capabilities
    let payload = caps.value("payload")?.get::<i32>()?;

    //  media payload
    if expected_media_pt as i32 == payload {
        // Get the audiodepay from withing the pipeline
        // Its defined in the textual description so this should not panic
        let audiodepay = sip_bin.by_name("audiodepay").unwrap();

        // Try to get the `sink` static pad to link it with the new pad
        let sink_pad = audiodepay
            .static_pad("sink")
            .context("audiodepay static pad 'sink' not available")?;

        // Link the incoming newly added pad to the audiodepay
        pad.link(&sink_pad)?;
    } else if telephone_event_pt as i32 == payload {
        // A DTMF event was detected and a pad was added for it.
        //
        // Create a dtmfdepay and link its audio output to a fakesink as we don't want the audio
        // from it, only the signals it emits on the pipeline's bus.
        let dtmfdepay = gst::ElementFactory::make("rtpdtmfdepay")
            .name("dtmf-depay")
            .build()?;
        let fakesink = gst::ElementFactory::make("fakesink")
            .name("dtmf-fakesink")
            .build()?;

        // add the dtmfdepay and fakesink to the pipeline
        sip_bin.add_many(&[&fakesink, &dtmfdepay])?;

        // Link the new pad (from this callback) to the dtmfdepay
        let dtmfdepay_sink = dtmfdepay
            .static_pad("sink")
            .context("dtmfdepay static pad 'sink' not available")?;
        pad.link(&dtmfdepay_sink)?;

        // Link the dtmfdepay to the fakesink
        let dtmfdepay_src = dtmfdepay
            .static_pad("src")
            .context("dtmfdepay static pad 'src' not available")?;
        let fakesink_sink = fakesink
            .static_pad("sink")
            .context("fakesink static pad 'sink' not available")?;
        dtmfdepay_src.link(&fakesink_sink)?;

        dtmfdepay.set_state(gst::State::Playing)?;
        fakesink.set_state(gst::State::Playing)?;
    } else {
        log::warn!("pad with unknown pt {} added", payload);
    }

    Ok(())
}
