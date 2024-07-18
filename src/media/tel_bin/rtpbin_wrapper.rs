// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::port_pool::PortPool;
use crate::media::util::ElementConvenienceExt;
use crate::sip::SrtpInfo;
use anyhow::{ensure, Context, Result};
use gio::InetSocketAddress;
use gst::glib;
use gst::prelude::*;
use parking_lot::Mutex;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// Convenience wrapper around gstreamer's rtpbin element which manages sending an receiving rtp/rtcp over the network
pub struct RtpBin {
    bin: gst::Bin,
    rtpbin: gst::Element,
    sessions: HashMap<u32, Session>,
    ptmap: Arc<Mutex<HashMap<u32, gst::Caps>>>,
}

struct Session {
    rtcp_udpsink: gst::Element,
    rtcp_udpsrc: gst::Element,
    rtpbin_recv_rtcp_sink: gst::Pad,
    rtpbin_send_rtcp_src: gst::Pad,

    rtcp_socket_addr: InetSocketAddress,

    srtp: Option<SrtpElements>,

    sender: Option<Sender>,
    receiver: Option<Receiver>,
}

struct SrtpElements {
    srtpenc: gst::Element,
    srtpdec: gst::Element,
}

impl Drop for Session {
    fn drop(&mut self) {
        let port_pool = PortPool::instance();
        let port = self.rtcp_socket_addr.property::<u32>("port");
        port_pool.clear_port((port as u16) - 1);
    }
}

struct Sender {
    ghost_pad: gst::GhostPad,
    watchdog: gst::Element,
    rtp_udpsink: gst::Element,
}

struct Receiver {
    rtp_udpsrc: gst::Element,
    watchdog: gst::Element,
    rtpbin_recv_rtp_sink: gst::Pad,
}

impl RtpBin {
    pub fn new(ready_to_send: Arc<Notify>) -> Result<Self> {
        let bin = gst::parse::bin_from_description("rtpbin name=rtpbin", false)?;
        let rtpbin = bin
            .by_name("rtpbin")
            .context("Failed to get rtpbin from bin")?;

        let rtpbin_src_pads = Arc::new(Mutex::new(HashMap::new()));

        rtpbin.connect_pad_added(rtpbin_pad_added(rtpbin_src_pads.clone()));
        rtpbin.connect_pad_removed(rtpbin_pad_removed(rtpbin_src_pads));

        let ptmap: Arc<Mutex<HashMap<u32, gst::Caps>>> = Arc::default();

        rtpbin.connect("request-pt-map", true, rtpbin_request_pt_map(ptmap.clone()));

        let notify_on_ready_to_send = ready_to_send.clone();
        rtpbin.connect("on-sender-ssrc-active", true, move |_values| {
            notify_on_ready_to_send.notify_one();

            None
        });

        Ok(Self {
            bin,
            rtpbin,
            sessions: HashMap::new(),
            ptmap,
        })
    }

    pub fn bin(&self) -> &gst::Bin {
        &self.bin
    }

    async fn get_or_create_session(
        &mut self,
        session_id: u32,
        srtp_info: Option<&SrtpInfo>,
        rtcp_udpsrc: gst::Element,
        rtcp_udpsink: gst::Element,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<&mut Session> {
        match self.sessions.entry(session_id) {
            Entry::Occupied(session) => Ok(session.into_mut()),
            Entry::Vacant(vacant) => {
                rtcp_udpsink.set_property("host", remote_rtcp_addr.ip().to_string());
                rtcp_udpsink.set_property("port", remote_rtcp_addr.port() as i32);

                let recv_rtcp_sink = self
                    .rtpbin
                    .request_pad_simple(&format!("recv_rtcp_sink_{session_id}"))
                    .context("Failed to request pad 'recv_rtcp_sink_%u' on rtpbin")?;
                let send_rtcp_src = self
                    .rtpbin
                    .request_pad_simple(&format!("send_rtcp_src_{session_id}"))
                    .context("Failed to request pad 'send_rtcp_src_%u' on rtpbin")?;

                self.bin
                    .add(&rtcp_udpsrc)
                    .context("Failed to add rtcp-udpsrc to bin")?;
                self.bin
                    .add(&rtcp_udpsink)
                    .context("Failed to add rtcp-udpsink to bin")?;

                let srtp = if let Some(srtp) = srtp_info {
                    let mut send_key = gst::Buffer::with_size(srtp.send_key.len())
                        .context("Failed to create send_key buffer")?;
                    send_key
                        .make_mut()
                        .copy_from_slice(0, &srtp.send_key)
                        .expect("allocated enough space for this to not fail");

                    let srtpenc = gst::ElementFactory::make("srtpenc")
                        .property("key", &send_key)
                        .property_from_str("rtp-cipher", srtp.gst_cipher)
                        .property_from_str("rtcp-cipher", srtp.gst_cipher)
                        .property_from_str("rtp-auth", srtp.gst_auth)
                        .property_from_str("rtcp-auth", srtp.gst_auth)
                        .build()?;

                    let mut recv_key = gst::Buffer::with_size(srtp.recv_key.len())
                        .context("Failed to create recv_key buffer")?;
                    recv_key
                        .make_mut()
                        .copy_from_slice(0, &srtp.recv_key)
                        .expect("allocated enough space for this to not fail");

                    let srtpdec = gst::ElementFactory::make("srtpdec").build()?;

                    let gst_cipher = srtp.gst_cipher;
                    let gst_auth = srtp.gst_auth;
                    srtpdec.connect("request-key", false, move |_| {
                        Some(
                            gst::Caps::builder("application/x-srtp")
                                .field("srtp-key", &recv_key)
                                .field("srtp-cipher", gst_cipher)
                                .field("srtp-auth", gst_auth)
                                .field("srtcp-cipher", gst_cipher)
                                .field("srtcp-auth", gst_auth)
                                .build()
                                .to_value(),
                        )
                    });

                    self.bin.add(&srtpenc)?;
                    self.bin.add(&srtpdec)?;

                    rtcp_udpsrc
                        .static_pad("src")
                        .context("Missing static pad 'src'")?
                        .link(
                            &srtpdec
                                .static_pad("rtcp_sink")
                                .context("Failed to get static-pad 'rtcp_sink' of srtpdec")?,
                        )
                        .context("Failed to link rtcp_udpsrc to srtpdec")?;

                    srtpdec
                        .static_pad("rtcp_src")
                        .context("Missing static pad 'src'")?
                        .link(&recv_rtcp_sink)
                        .context("Failed to link srtpdec to recv_rtcp_sink")?;

                    // rtpbin -> srtpenc -> udpsink
                    let srtpenc_rtcp_sink = srtpenc
                        .request_pad_simple(&format!("rtcp_sink_{session_id}"))
                        .context("Failed to request pad 'rtcp_sink_%u'")?;
                    send_rtcp_src
                        .link(&srtpenc_rtcp_sink)
                        .context("Failed to link send_rtcp_srt to srtpenc.rtcp_sink")?;

                    let srtpenc_send_rtcp_src = srtpenc
                        .pads()
                        .into_iter()
                        .find(|p| p.name() == format!("rtcp_src_{session_id}"))
                        .context("Failed to find matching rtcp_src_%u pad")?;

                    srtpenc_send_rtcp_src
                        .link(
                            &rtcp_udpsink
                                .static_pad("sink")
                                .context("Failed to get static-pad 'sink' from rtcp-udpsink")?,
                        )
                        .context(
                            "Failed to link srtpenc's 'send_rtcp_src_%u' to rtcp-udpsink 'sink'",
                        )?;

                    srtpenc.set_state_async(gst::State::Playing).await?;
                    srtpdec.set_state_async(gst::State::Playing).await?;

                    Some(SrtpElements { srtpenc, srtpdec })
                } else {
                    rtcp_udpsrc
                        .static_pad("src")
                        .context("Failed to get static-pad 'src' from rtcp-udpsrc")?
                        .link(&recv_rtcp_sink)
                        .context("Failed to link rtcp-udpsrc 'src'to rtpbin 'recv_rtcp_sink_%u'")?;

                    send_rtcp_src
                        .link(
                            &rtcp_udpsink
                                .static_pad("sink")
                                .context("Failed to get static-pad 'sink' from rtcp-udpsink")?,
                        )
                        .context(
                            "Failed to link rtpbin 'send_rtcp_src_%u' to rtcp-udpsink 'sink'",
                        )?;

                    None
                };

                rtcp_udpsink.set_state_async(gst::State::Playing).await?;
                rtcp_udpsrc.set_state_async(gst::State::Playing).await?;

                let socket = rtcp_udpsrc.property::<gio::Socket>("socket");
                let socket_addr = socket.property::<gio::InetSocketAddress>("local-address");

                Ok(vacant.insert(Session {
                    rtcp_udpsink,
                    rtcp_udpsrc,
                    rtpbin_recv_rtcp_sink: recv_rtcp_sink,
                    rtpbin_send_rtcp_src: send_rtcp_src,
                    rtcp_socket_addr: socket_addr,
                    srtp,
                    sender: None,
                    receiver: None,
                }))
            }
        }
    }

    pub fn set_pt_caps(&mut self, pt: u32, caps: gst::Caps) {
        self.ptmap.lock().insert(pt, caps);
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_sender(
        &mut self,
        session_id: u32,
        srtp_info: Option<&SrtpInfo>,
        rtp_udpsink: gst::Element,
        rtcp_udpsrc: gst::Element,
        rtcp_udpsink: gst::Element,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<gst::GhostPad> {
        let self_rtpbin = self.rtpbin.clone();
        let self_bin = self.bin.clone();

        let session = self
            .get_or_create_session(
                session_id,
                srtp_info,
                rtcp_udpsrc,
                rtcp_udpsink,
                remote_rtcp_addr,
            )
            .await
            .context("Failed to get_or_create_session")?;

        ensure!(
            session.sender.is_none(),
            "Tried to create a second sender for the same session_id ({session_id})",
        );

        rtp_udpsink.set_property("host", remote_rtp_addr.ip().to_string());
        rtp_udpsink.set_property("port", remote_rtp_addr.port() as i32);

        // ghost-pad -> rtpbin.send_rtp_sink_%u
        let rtp_sink = self_rtpbin
            .request_pad_simple("send_rtp_sink_%u")
            .context("Failed to request pad 'send_rtp_sink_%u' on rtpbin")?;
        let ghost_pad =
            gst::GhostPad::with_target(&rtp_sink).context("Failed to create rtp-sink GhostPad")?;

        self_bin
            .add_pad(&ghost_pad)
            .context("Failed to add GhostPad to bin")?;

        // rtpbin.send_rtp_src_%u -> watchdog [-> srtpenc] -> udpsink
        let rtp_src = self_rtpbin
            .pads()
            .into_iter()
            .find(|p| p.name().starts_with("send_rtp_src_") && p.peer().is_none())
            .context("Failed to find 'send_rtp_src_%u' pad in rtpbin")?;

        let watchdog = gst::ElementFactory::make("watchdog")
            .name(format!("send-rtp-watchdog-{remote_rtp_addr}"))
            .property("timeout", Duration::from_secs(10).as_millis() as i32)
            .build()
            .context("Failed to build rtp sender watchdog")?;

        session.sender = Some(Sender {
            ghost_pad: ghost_pad.clone(),
            watchdog: watchdog.clone(),
            rtp_udpsink: rtp_udpsink.clone(),
        });

        self_bin
            .add_many([&watchdog, &rtp_udpsink])
            .context("Failed to add watchdog & rtp-udpsink to bin")?;

        let watchdog_sink = watchdog
            .static_pad("sink")
            .context("Failed to get static-pad sink in watchdog")?;
        rtp_src
            .link(&watchdog_sink)
            .context("Failed to link rtpbin.send_rtp_src_%u to watchdog.sink")?;

        if let Some(srtp) = &session.srtp {
            let srtpenc_rtp_sink = srtp
                .srtpenc
                .request_pad_simple(&format!("rtp_sink_{session_id}"))
                .context("Failed to request pad 'rtp_sink_%u'")?;

            watchdog
                .static_pad("src")
                .context("Failed to get static-pad 'src' from watchdog")?
                .link(&srtpenc_rtp_sink)
                .context("Failed to link rtp_src to srtpenc.rtp_sink")?;

            let srtpenc_send_rtp_src = srtp
                .srtpenc
                .pads()
                .into_iter()
                .find(|p| p.name() == format!("rtp_src_{session_id}"))
                .context("Failed to find pad 'rtp_src_%u'")?;

            srtpenc_send_rtp_src
                .link(
                    &rtp_udpsink
                        .static_pad("sink")
                        .context("Failed to get static-pad 'sink' on udpsink")?,
                )
                .context("Failed to link srtpenc.send_rtp_src to rtp_udpsink")?;
        } else {
            watchdog.link(&rtp_udpsink)?;
        }

        watchdog.set_state_async(gst::State::Playing).await?;
        rtp_udpsink.set_state_async(gst::State::Playing).await?;

        Ok(ghost_pad)
    }

    pub async fn add_receiver(
        &mut self,
        session_id: u32,
        srtp_info: Option<&SrtpInfo>,
        rtp_udpsrc: gst::Element,
        rtcp_udpsrc: gst::Element,
        rtcp_udpsink: gst::Element,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<()> {
        let self_bin = self.bin.clone();
        let self_rtpbin = self.rtpbin.clone();

        let session = self
            .get_or_create_session(
                session_id,
                srtp_info,
                rtcp_udpsrc,
                rtcp_udpsink,
                remote_rtcp_addr,
            )
            .await
            .context("Failed to get_or_create_session")?;

        ensure!(
            session.receiver.is_none(),
            "Tried to create a second receiver for the same session_id ({session_id})"
        );

        // link udpsrc -> watchdog [-> srtpdec] -> rtpbin.recv_rtp_sink_%u
        let watchdog = gst::ElementFactory::make("watchdog")
            .name(format!("receive-rtp-watchdog-{remote_rtcp_addr}"))
            .property("timeout", Duration::from_secs(10).as_millis() as i32)
            .build()
            .context("Failed to build rtp sender watchdog")?;

        self_bin
            .add_many([&watchdog, &rtp_udpsrc])
            .context("Failed to add watchdog & rtp-udpsrc to bin")?;

        rtp_udpsrc.link(&watchdog)?;

        let rtp_sink = self_rtpbin
            .request_pad_simple("recv_rtp_sink_%u")
            .context("Failed to request pad 'recv_rtp_sink_%u' on rtpbin")?;

        if let Some(srtp) = &session.srtp {
            let srtpdec_rtp_sink = srtp
                .srtpdec
                .static_pad("rtp_sink")
                .context("Failed to get static-pad rtp_sink")?;
            let srtpdec_rtp_src = srtp
                .srtpdec
                .static_pad("rtp_src")
                .context("Failed to get static-pad rtp_src")?;

            watchdog
                .static_pad("src")
                .context("Failed to get static-pad 'src' of watchdog")?
                .link(&srtpdec_rtp_sink)
                .context("Failed to link udpsrc.src to srtpdec.rtp_sink")?;

            srtpdec_rtp_src
                .link(&rtp_sink)
                .context("Failed to link srtpdec.rtp_src to rtpbin.recv_rtp_sink_%u")?;
        } else {
            watchdog
                .static_pad("src")
                .context("Failed to get static pad 'src' from watchdog")?
                .link(&rtp_sink)
                .context("Failed to link rtp-udpsrc to rtpbin.recv_rtp_sink_%u")?;
        }

        rtp_udpsrc.set_state_async(gst::State::Playing).await?;
        watchdog.set_state_async(gst::State::Playing).await?;

        session.receiver = Some(Receiver {
            rtp_udpsrc,
            watchdog,
            rtpbin_recv_rtp_sink: rtp_sink,
        });

        Ok(())
    }

    async fn remove_session(&mut self, session_id: u32) -> Result<()> {
        let session = self
            .sessions
            .remove(&session_id)
            .context("Tried to remove non-existing session")?;

        ensure!(
            session.sender.is_none() | session.receiver.is_none(),
            "Cannot remove session if there's still a sender or receiver connected"
        );

        if let Some(srtp) = &session.srtp {
            session.rtcp_udpsrc.unlink(&srtp.srtpdec);
            srtp.srtpdec.unlink(&self.rtpbin);

            self.rtpbin.unlink(&srtp.srtpenc);
            srtp.srtpenc.unlink(&session.rtcp_udpsink);
        } else {
            session.rtcp_udpsrc.unlink(&self.rtpbin);
            self.rtpbin.unlink(&session.rtcp_udpsink);
        }

        self.rtpbin
            .release_request_pad(&session.rtpbin_recv_rtcp_sink);
        self.rtpbin
            .release_request_pad(&session.rtpbin_send_rtcp_src);

        session
            .rtcp_udpsrc
            .set_state_async(gst::State::Null)
            .await?;
        session
            .rtcp_udpsink
            .set_state_async(gst::State::Null)
            .await?;

        self.bin
            .remove(&session.rtcp_udpsink)
            .context("Failed to remove rtcp_udpsink")?;
        self.bin
            .remove(&session.rtcp_udpsrc)
            .context("Failed to remove rtcp_udpsrc")?;

        if let Some(srtp) = &session.srtp {
            srtp.srtpdec.set_state_async(gst::State::Null).await?;
            srtp.srtpenc.set_state_async(gst::State::Null).await?;

            self.bin
                .remove(&srtp.srtpenc)
                .context("Failed to remove srtpenc from bin")?;
            self.bin
                .remove(&srtp.srtpdec)
                .context("Failed to remove srtpdec from bin")?;
        }

        Ok(())
    }

    pub async fn remove_sender(&mut self, session_id: u32) -> Result<()> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .context("Tried to remove sender from non-existing session")?;

        let sender = session
            .sender
            .take()
            .context("Tried to non-existing sender from existing session")?;

        // Remove ghostpad ! rtpbin.send_rtp_sink_%u
        let rtpbin_send_rtp_sink = sender
            .ghost_pad
            .target()
            .context("sender ghostpad has no target")?;
        sender.ghost_pad.set_target(gst::Pad::NONE)?;

        self.rtpbin.release_request_pad(&rtpbin_send_rtp_sink);

        // Remove rtpudpsink
        if let Some(srtp) = &session.srtp {
            sender.watchdog.unlink(&srtp.srtpenc);
            srtp.srtpenc.unlink(&sender.rtp_udpsink);
        }

        // Remove (rtpbin !) watchdog ! rtpudpsink
        let rtp_udpsink_pad = sender
            .watchdog
            .static_pad("sink")
            .context("Failed to get static-pad sink of watchdog")?;
        assert!(
            rtp_udpsink_pad.peer().is_none(),
            "send_rtp_sink_%u was released, so this cannot be linked anymore"
        );

        sender.watchdog.set_state_async(gst::State::Null).await?;
        sender.rtp_udpsink.set_state_async(gst::State::Null).await?;
        self.bin
            .remove_many([&sender.watchdog, &sender.rtp_udpsink])?;

        if session.receiver.is_none() {
            self.remove_session(session_id)
                .await
                .context("Failed to remove session")?;
        }

        Ok(())
    }

    pub async fn remove_receiver(&mut self, session_id: u32) -> Result<()> {
        let session = self
            .sessions
            .get_mut(&session_id)
            .context("Tried to remove receiver from non-existing session")?;

        let receiver = session
            .receiver
            .take()
            .context("Tried to non-existing receiver from existing session")?;

        // Remove udpsrc -> watchdog -> [-> srtpdec] -> rtpbin.recv_rtp_sink_%u
        if let Some(srtp) = &session.srtp {
            receiver.rtp_udpsrc.unlink(&receiver.watchdog);
            receiver.watchdog.unlink(&srtp.srtpdec);
            srtp.srtpdec.unlink(&self.rtpbin);
        } else {
            receiver.rtp_udpsrc.unlink(&receiver.watchdog);
            receiver.watchdog.unlink(&self.rtpbin);
        }

        self.rtpbin
            .release_request_pad(&receiver.rtpbin_recv_rtp_sink);
        receiver
            .rtp_udpsrc
            .set_state_async(gst::State::Null)
            .await?;
        receiver.watchdog.set_state_async(gst::State::Null).await?;

        self.bin.remove(&receiver.rtp_udpsrc)?;
        self.bin.remove(&receiver.watchdog)?;

        if session.sender.is_none() {
            self.remove_session(session_id)
                .await
                .context("Failed to remove session")?;
        }

        Ok(())
    }
}

/// Get the session id of the recv_rtp_src_%u_%u_%u which is (recv_rtp_src_{session_id}_{ssrc}_{payload})
pub(super) fn session_id_of_rtp_src_pad(name: &str) -> Result<u32> {
    let name = name.trim_start_matches("recv_rtp_src_");
    let (id, _) = name.split_once('_').context("Failed to split @ _")?;
    id.parse().context("Failed to parse session_id from string")
}

fn rtpbin_pad_added(
    rtpbin_src_pads: Arc<Mutex<HashMap<String, gst::GhostPad>>>,
) -> impl Fn(&gst::Element, &gst::Pad) {
    move |rtpbin, pad| {
        if !pad.name().starts_with("recv_rtp_src_") {
            return;
        }

        /// When encountering a new 'recv_rtp_src_%u' pad create a new GhostPad on the bin (rtpbin's parent)
        fn try_rtpbin_pad_added(
            rtpbin_src_pads: &Mutex<HashMap<String, gst::GhostPad>>,
            rtpbin: &gst::Element,
            pad: &gst::Pad,
        ) -> Result<()> {
            let parent = rtpbin.parent().context("rtpbin has no parent")?;
            let parent = parent
                .downcast::<gst::Bin>()
                .ok()
                .context("rtpbin's parent is not a bin")?;

            let ghost_pad = gst::GhostPad::builder_with_target(pad)
                .context("Failed to create GhostPad")?
                .name(pad.name())
                .build();
            parent
                .add_pad(&ghost_pad)
                .context("Failed to add GhostPad")?;

            // Save the ghostpad so we can remove it later in the pad-removed callback
            rtpbin_src_pads.lock().insert(pad.name().into(), ghost_pad);

            Ok(())
        }

        if let Err(e) = try_rtpbin_pad_added(&rtpbin_src_pads, rtpbin, pad) {
            log::error!("Failed to handle pad-added of rtpbin, {e:?}");
        }
    }
}

fn rtpbin_pad_removed(
    rtpbin_src_pads: Arc<Mutex<HashMap<String, gst::GhostPad>>>,
) -> impl Fn(&gst::Element, &gst::Pad) {
    move |rtpbin, pad| {
        if !pad.name().starts_with("recv_rtp_src_") {
            return;
        }

        /// When encountering a removed 'recv_rtp_src_%u' pad remove the ghostpad we created for it
        fn try_rtpbin_pad_removed(
            rtpbin_src_pads: &Mutex<HashMap<String, gst::GhostPad>>,
            rtpbin: &gst::Element,
            pad: &gst::Pad,
        ) -> Result<()> {
            let parent = rtpbin.parent().context("rtpbin has no parent")?;
            let parent = parent
                .downcast::<gst::Bin>()
                .ok()
                .context("rtpbin's parent is not a bin")?;

            let mut rtpbin_src_pads = rtpbin_src_pads.lock();

            let ghost_pad = rtpbin_src_pads
                .remove(pad.name().as_str())
                .context("Failed to find a GhostPad for removed recv_rtp_src_%u pad")?;

            parent
                .remove_pad(&ghost_pad)
                .context("Failed to remove GhostPad for removed recv_rtp_src_%u pad")?;

            Ok(())
        }

        if let Err(e) = try_rtpbin_pad_removed(&rtpbin_src_pads, rtpbin, pad) {
            log::error!("Failed to handle pad-added of rtpbin, {e:?}");
        }
    }
}

fn rtpbin_request_pt_map(
    ptmap: Arc<Mutex<HashMap<u32, gst::Caps>>>,
) -> impl Fn(&[glib::Value]) -> Option<glib::Value> {
    move |values| {
        let pt = values[2]
            .get::<u32>()
            .expect("values[2] of request-pt-map is always a u32 (guint)");

        let ptmap = ptmap.lock();

        if let Some(caps) = ptmap.get(&pt) {
            Some(caps.to_value())
        } else {
            log::error!(
                "requested caps for unknown pt {}, expected any of {:?}",
                pt,
                ptmap.keys().collect::<Vec<_>>()
            );

            Some(
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
                    .to_value(),
            )
        }
    }
}
