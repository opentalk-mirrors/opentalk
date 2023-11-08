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
use gst::prelude::*;
use parking_lot::Mutex;
use sdp::SessionInfo;
use sdp_types::attributes::direction::Direction;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Weak};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Notify;

mod codec;
mod custom_rtpdtmfdepay;
mod sdp;

pub use custom_rtpdtmfdepay::init_custom_rtpdtmfdepay;

pub struct SipBin {
    pub(crate) bin: gst::Bin,

    id: u64,
    version: u64,

    local_rtp_addr: SocketAddr,
    local_rtcp_addr: SocketAddr,

    current_session_info: SessionInfo,

    /// Keep the only strong reference of the element_chains map to drop all elements when this sip-bin is dropped
    _element_chains: Arc<Mutex<HashMap<String, ElementChain>>>,
}

impl SipBin {
    /// Create a new SipBin based on a public [IpAddr] and an SDP offer
    ///
    /// Usually called by the first SIP INVITE.
    pub fn new(
        pub_addr: IpAddr,
        offer: Bytes,
        ready_to_send: Arc<Notify>,
    ) -> Result<(SipBin, String)> {
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
        let element_chains = Arc::new(Mutex::new(HashMap::new()));

        let bin = create_sip_bin(
            Arc::downgrade(&element_chains),
            socket_pair.rtp_socket,
            socket_pair.rtcp_socket,
            current_session_info.telephone_event_pt,
            current_session_info.gst_elements.clone(),
            ready_to_send,
        )
        .context("Failed to create sip_bin")?;

        let mut this = Self {
            bin,
            id,
            version,
            local_rtp_addr,
            local_rtcp_addr,
            current_session_info,
            _element_chains: element_chains,
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
    element_chains: Weak<Mutex<HashMap<String, ElementChain>>>,
    rtp_socket: gio::Socket,
    rtcp_socket: gio::Socket,
    telephone_event_pt: u32,
    gst_elements: GstCodecInfo,
    ready_to_send: Arc<Notify>,
) -> Result<gst::Bin> {
    // Create pipeline textual description
    let sip_bin = format!(
        r#"
        rtpbin name=session autoremove=true

        queue name=audio-sink min-threshold-time=20000000 ! audio/x-raw ! clocksync sync=true ! valve name=audio-input-valve ! audioconvert ! audioresample  ! {encoder} ! {rtppay} ! session.send_rtp_sink_0

        udpsrc name=rtp-udp-src ! queue ! watchdog name=rtp-watchdog timeout=10000 ! session.recv_rtp_sink_0
        udpsrc name=rtcp-udp-src caps="application/x-rtcp" ! queue ! session.recv_rtcp_sink_0

        session.send_rtp_src_0 ! watchdog name=send-rtp-watchdog timeout=10000 ! udpsink host=127.0.0.1 port=9 name=rtp-udpsink
        session.send_rtcp_src_0 ! udpsink host=127.0.0.1 port=9 name=rtcp-udpsink
        "#,
        encoder = gst_elements.encoder,
        rtppay = gst_elements.rtppay,
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

    rtp_bin.connect_pad_added(on_pad_added(
        sip_bin.downgrade(),
        element_chains.clone(),
        gst_elements.payload_caps.payload,
        telephone_event_pt,
        gst_elements.clone(),
    ));

    rtp_bin.connect_pad_removed(on_pad_removed(sip_bin.downgrade(), element_chains));

    let notify_on_ready_to_send = ready_to_send.clone();
    rtp_bin.connect("on-sender-ssrc-active", true, move |_values| {
        notify_on_ready_to_send.notify_one();

        None
    });

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

enum ElementChain {
    Audio {
        depay: gst::Element,
        decode: gst::Element,
        queue: gst::Element,
        ghost_pad: gst::GhostPad,
    },
    Dtmf {
        depay: gst::Element,
        fakesink: gst::Element,
    },
    Unkonwn {
        fakesink: gst::Element,
    },
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
            log::error!(
                "requested caps for unknown pt {}, expected media_pt={} or dtmf_pt={}",
                pt,
                payload_caps.payload,
                telephone_event_pt
            );

            // rtpbin's jitterbuffer has detected a new payload type which is unknown to us.
            // If we return nothing here, then this callback will be called every time a new RTP packet
            // with that unknown pt is received, spamming the log output with warnings and errors all around.
            //
            // But we're not really able to provide the correct clock-rate since the payload type is ... unknown
            //
            // So this provides caps with a super high clock-rate. The jitterbuffer now calculates that every RTP packet
            // with this payload type is lagging years behind and doesn't event think about queuing it up. This way
            // every packets can be discarded immediately into the linked fakesink.
            gst::Caps::builder("application/x-rtp")
                .field("clock-rate", i32::MAX)
                .build()
        };

        Some(caps.to_value())
    }
}

fn on_pad_added(
    sip_bin_weak: glib::WeakRef<gst::Bin>,
    element_chains: Weak<Mutex<HashMap<String, ElementChain>>>,
    expected_media_pt: u32,
    telephone_event_pt: u32,
    gst_elements: GstCodecInfo,
) -> impl Fn(&gst::Element, &gst::Pad) {
    move |_, pad| {
        if let Some((sip_bin, element_chains)) =
            sip_bin_weak.upgrade().zip(element_chains.upgrade())
        {
            if let Err(e) = try_on_pad_added(
                &sip_bin,
                &element_chains,
                expected_media_pt,
                telephone_event_pt,
                gst_elements.clone(),
                pad,
            ) {
                log::error!("failed to handle added pad, {e:?}");
            }
        }
    }
}

fn try_on_pad_added(
    sip_bin: &gst::Bin,
    element_chains: &Mutex<HashMap<String, ElementChain>>,
    expected_media_pt: u32,
    telephone_event_pt: u32,
    gst_elements: GstCodecInfo,
    pad: &gst::Pad,
) -> Result<()> {
    // Get the pads capabilities, in it search for the payload of the incoming stream
    let caps = pad.caps().context("no caps on pad, cannot link")?;
    let caps = caps
        .structure(0)
        .context("caps does not contain any capabilities")?;

    log::debug!("New pad with caps {caps:#?}");

    // Extract payload number from capabilities
    let payload = caps.value("payload")?.get::<i32>()?;

    //  media payload
    if expected_media_pt as i32 == payload {
        // Create a chain of rtp-depay ! decode ! queue
        let depay = gst::ElementFactory::make(gst_elements.rtpdepay)
            .build()
            .context("Failed to create rtp de-payloader element")?;
        let decode = gst::ElementFactory::make(gst_elements.decoder)
            .build()
            .context("Failed to create audio decoder element")?;
        let queue = gst::ElementFactory::make("queue")
            .build()
            .context("Failed to create queue")?;

        sip_bin
            .add_many(&[&depay, &decode, &queue])
            .context("Failed to add audio deocder elements")?;
        gst::Element::link_many(&[&depay, &decode, &queue])
            .context("Failed to link audio deocder elements")?;

        // Link the incoming newly added pad to the audiodepay
        let depay_sink = depay
            .static_pad("sink")
            .context("rtpdepay element has no static-pad `sink`")?;
        pad.link(&depay_sink)?;

        // Create a ghost pad to route the audio outside the bin
        let queue_src = queue
            .static_pad("src")
            .context("Failed to get queue's static-pad `src`")?;
        let ghost_pad = gst::GhostPad::with_target(None, &queue_src)?;
        sip_bin.add_pad(&ghost_pad)?;

        depay.sync_state_with_parent()?;
        decode.sync_state_with_parent()?;
        queue.sync_state_with_parent()?;

        element_chains.lock().insert(
            pad.name().to_string(),
            ElementChain::Audio {
                depay,
                decode,
                queue,
                ghost_pad,
            },
        );
    } else if telephone_event_pt as i32 == payload {
        // A DTMF event was detected and a pad was added for it.
        //
        // Create a dtmfdepay and link its audio output to a fakesink as we don't want the audio
        // from it, only the signals it emits on the pipeline's bus.
        let dtmfdepay = gst::ElementFactory::make("opentalk-rtpdtmfdepay").build()?;
        let fakesink = gst::ElementFactory::make("fakesink").build()?;

        // add the dtmfdepay and fakesink to the pipeline
        sip_bin.add_many(&[&dtmfdepay, &fakesink])?;
        gst::Element::link_many(&[&dtmfdepay, &fakesink])?;

        // Link the new pad to the dtmfdepay
        let dtmfdepay_sink = dtmfdepay
            .static_pad("sink")
            .context("dtmfdepay static pad 'sink' not available")?;
        pad.link(&dtmfdepay_sink)?;

        dtmfdepay.sync_state_with_parent()?;
        fakesink.sync_state_with_parent()?;

        element_chains.lock().insert(
            pad.name().to_string(),
            ElementChain::Dtmf {
                depay: dtmfdepay,
                fakesink,
            },
        );
    } else {
        log::error!("pad with unexpected pt={payload} added, expected media with pt={expected_media_pt} or dtmf with pt={telephone_event_pt}. \
                     Future RTP packets with this payload will be discarded.");

        let fakesink = gst::ElementFactory::make("fakesink").build()?;
        sip_bin.add(&fakesink)?;

        pad.link(
            &fakesink
                .static_pad("sink")
                .context("missing static-pad sink on fakesink")?,
        )?;

        fakesink.sync_state_with_parent()?;

        element_chains
            .lock()
            .insert(pad.name().to_string(), ElementChain::Unkonwn { fakesink });
    }

    Ok(())
}

fn on_pad_removed(
    sip_bin_weak: glib::WeakRef<gst::Bin>,
    element_chains: Weak<Mutex<HashMap<String, ElementChain>>>,
) -> impl Fn(&gst::Element, &gst::Pad) {
    move |_, pad| {
        if let Some((sip_bin, element_chains)) =
            sip_bin_weak.upgrade().zip(element_chains.upgrade())
        {
            if let Err(e) = try_on_pad_removed(&sip_bin, &element_chains, pad) {
                log::error!("failed to handle removed pad, {e:?}");
            }
        }
    }
}

fn try_on_pad_removed(
    sip_bin: &gst::Bin,
    element_chains: &Mutex<HashMap<String, ElementChain>>,
    pad: &gst::Pad,
) -> Result<()> {
    let name = pad.name().to_string();

    log::debug!("Pad removed name={name:?}");

    let element_chain = element_chains
        .lock()
        .remove(&name)
        .context("Unkown pad {name:?}")?;

    match element_chain {
        ElementChain::Audio {
            depay,
            decode,
            queue,
            ghost_pad,
        } => {
            gst::Element::unlink_many(&[&depay, &decode, &queue]);
            sip_bin.remove_pad(&ghost_pad)?;
            sip_bin.remove_many(&[&depay, &decode, &queue])?;

            depay.set_state(gst::State::Null)?;
            decode.set_state(gst::State::Null)?;
            queue.set_state(gst::State::Null)?;
        }
        ElementChain::Dtmf { depay, fakesink } => {
            gst::Element::unlink_many(&[&depay, &fakesink]);
            sip_bin.remove_many(&[&depay, &fakesink])?;

            depay.set_state(gst::State::Null)?;
            fakesink.set_state(gst::State::Null)?;
        }
        ElementChain::Unkonwn { fakesink } => {
            sip_bin.remove(&fakesink)?;
            fakesink.set_state(gst::State::Null)?;
        }
    }

    Ok(())
}
