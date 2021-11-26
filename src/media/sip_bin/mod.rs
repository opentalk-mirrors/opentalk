use anyhow::{Context, Result};
use bytes::Bytes;
use bytesstr::BytesStr;
use gio::prelude::*;
use gst::glib;
use gst::prelude::*;
use std::convert::TryFrom;
use std::net::{IpAddr, SocketAddr};

mod sdp;

pub fn create_sip_bin_from_sdp_offer(pub_addr: IpAddr, offer: Bytes) -> Result<(gst::Bin, String)> {
    // Parse SDP offer
    let offer = BytesStr::from_utf8_bytes(offer)?;
    let offer = sdp_types::msg::parse::<sdp_types::msg::Builder>(&offer)?;

    // Create RTP/RTCP UDP socket
    let (rtp_socket, rtp_port) = create_udp_socket()?;
    let (rtcp_socket, rtcp_port) = create_udp_socket()?;

    let local_info = sdp::LocalInfo {
        offer,
        rtp_addr: SocketAddr::new(pub_addr, rtp_port),
        rtcp_addr: SocketAddr::new(pub_addr, rtcp_port),
    };

    let answer = sdp::respond(local_info)?;

    // Create pipeline textual description
    let sip_bin = format!(
        r#"
            rtpbin name=session
    
            queue name=audio-sink max-size-time=20000000 ! audioconvert ! audioresample  ! alawenc ! rtppcmapay  ! session.send_rtp_sink_0
    
            udpsrc name=rtp-udp-src ! session.recv_rtp_sink_0
            udpsrc name=rtcp-udp-src caps="application/x-rtcp" ! session.recv_rtcp_sink_0
    
            session.send_rtp_src_0 ! udpsink host={ip} port={rtp_port} name=rtp-udp-sink
            session.send_rtcp_src_0 ! udpsink host={ip} port={rtcp_port} name=rtcp-udp-sink
        
            rtppcmadepay name=pcmadepay ! alawdec ! queue name=audio-src
            "#,
        ip = answer.rtp_addr.ip(),
        rtp_port = answer.rtp_addr.port(),
        rtcp_port = answer.rtcp_addr.port(),
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

    let audio_sink_ghost_pad = gst::GhostPad::new(Some("sip-audio-sink"), gst::PadDirection::Sink);
    sip_bin.add_pad(&audio_sink_ghost_pad)?;

    // TODO replace audiotestsrc against generic
    let audio_testsrc = gst::ElementFactory::make("audiotestsrc", None)?;
    sip_bin.add(&audio_testsrc)?;
    let audio_testsrc_src_pad = audio_testsrc.static_pad("src").unwrap();
    audio_testsrc_src_pad.link(&audio_sink_sink_pad)?;

    // Add 'linked' callback to GhostPad.
    // Once the ghostpad was linked against this callback will remove the audiotestsrc
    // and link whatever was linked against the ghostpad to the rtpstream to transmit that instead.
    let sip_bin_clone = sip_bin.clone();
    audio_sink_ghost_pad
        .clone()
        .connect("linked", true, move |_values| {
            if let Err(e) = || -> Result<()> {
                audio_testsrc_src_pad.unlink(&audio_sink_sink_pad)?;
                sip_bin_clone.remove(&audio_testsrc)?;
                audio_sink_ghost_pad.set_target(Some(&audio_sink_sink_pad))?;
                Ok(())
            }() {
                log::error!("failed to handle linked ghostpad {}", e);
            }

            None
        })?;

    // ==== CREATE AUDIO SRC GHOSTPAD (as in received audio from RTP)

    // Create outgoing audio ghost pad
    let audio_src = sip_bin
        .by_name("audio-src")
        .context("failed to get audio-src")?;

    let audio_src_src_pad = audio_src
        .static_pad("src")
        .context("failed to get src pad")?;

    let audio_src_ghost_pad = gst::GhostPad::new(Some("sip-audio-src"), gst::PadDirection::Src);
    sip_bin.add_pad(&audio_src_ghost_pad)?;

    // Connect outgoing audio to fakesink to not interrupt pipeline flow
    let audio_fakesink = gst::ElementFactory::make("fakesink", Some("sip-audio-fakesink"))?;
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
        })?;

    // Set sockets of udp_(src/sink)
    let rtp_udp_src = sip_bin.by_name("rtp-udp-src").unwrap();
    let rtp_udp_sink = sip_bin.by_name("rtp-udp-sink").unwrap();

    let rtcp_udp_src = sip_bin.by_name("rtcp-udp-src").unwrap();
    let rtcp_udp_sink = sip_bin.by_name("rtcp-udp-sink").unwrap();

    rtp_udp_src.set_property("socket", rtp_socket.clone())?;
    rtp_udp_sink.set_property("socket", rtp_socket)?;

    rtcp_udp_src.set_property("socket", rtcp_socket.clone())?;
    rtcp_udp_sink.set_property("socket", rtcp_socket)?;

    // Get the rtpbin element
    let rtp_bin = sip_bin.by_name("session").unwrap();

    rtp_bin.connect(
        "request-pt-map",
        false,
        on_request_pt_map(answer.telephone_event_pt),
    )?;

    rtp_bin.connect(
        "pad-added",
        true,
        on_pad_added(sip_bin.clone(), answer.telephone_event_pt),
    )?;

    Ok((sip_bin, answer.answer.to_string()))
}

fn create_udp_socket() -> Result<(gio::Socket, u16)> {
    let socket = gio::Socket::new(
        gio::SocketFamily::Ipv4,
        gio::SocketType::Datagram,
        gio::SocketProtocol::Udp,
    )?;

    let addr = gio::InetAddress::new_any(gio::SocketFamily::Ipv4);
    let addr = gio::InetSocketAddress::new(&addr, 0);
    socket.bind(&addr, false)?;

    let local_address = socket
        .property("local-address")?
        .get::<gio::SocketAddress>()?;
    let local_port = local_address.property("port")?.get::<u32>()?;

    Ok((socket, u16::try_from(local_port)?))
}

/// Callback function whenever the rtpbin encounters a new payload inside the RTP stream.
///
/// Provides gst::Caps for the given payload number, so gstreamer can properly handle
/// the payload. Currently maps 8 to PCMA and telephone_event_pt to telephone-event/dtmf caps.
fn on_request_pt_map(telephone_event_pt: u32) -> impl Fn(&[glib::Value]) -> Option<glib::Value> {
    move |values| {
        let pt = values[2].get::<u32>().unwrap();

        let caps = if pt == 8 {
            gst::Caps::builder_full()
                .structure(gst::Structure::new(
                    "application/x-rtp",
                    &[
                        ("media", &"audio"),
                        ("encoding-name", &"PCMA"),
                        ("clock-rate", &8000i32),
                    ],
                ))
                .build()
        } else if pt == telephone_event_pt {
            gst::Caps::builder_full()
                .structure(gst::Structure::new(
                    "application/x-rtp",
                    &[
                        ("media", &"audio"),
                        ("encoding-name", &"TELEPHONE-EVENT"),
                        ("clock-rate", &8000i32),
                    ],
                ))
                .build()
        } else {
            return None;
        };

        Some(caps.to_value())
    }
}

fn on_pad_added(
    sip_bin: gst::Bin,
    telephone_event_pt: u32,
) -> impl Fn(&[glib::Value]) -> Option<glib::Value> {
    move |values| {
        if let Err(e) = try_on_pad_added(&sip_bin, telephone_event_pt, values) {
            log::error!("failed to handle added pad, {}", e);
        }

        None
    }
}

fn try_on_pad_added(
    sip_bin: &gst::Bin,
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

    //  PCMA audio
    if payload == 8 {
        // Get the pcmadepay from withing the pipeline
        // Its defined in the textual description so this should not panic
        let pcmadepay = sip_bin.by_name("pcmadepay").unwrap();

        // Try to get the `sink` static pad to link it with the new pad
        let sink_pad = pcmadepay
            .static_pad("sink")
            .context("pcmadepay static pad 'sink' not available")?;

        // Link the incoming newly added pad to the pcmadepay
        pad.link(&sink_pad)?;
    } else if telephone_event_pt as i32 == payload {
        // A DTMF event was detected and a pad was added for it.
        //
        // Create a dtmfdepay and link its audio output to a fakesink as we don't want the audio
        // from it, only the signals it emits on the pipeline's bus.
        let dtmfdepay = gst::ElementFactory::make("rtpdtmfdepay", None)?;
        let fakesink = gst::ElementFactory::make("fakesink", None)?;

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
    }

    Ok(())
}
