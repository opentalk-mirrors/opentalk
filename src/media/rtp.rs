// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::sip::{MediaSessionId, SrtpInfo};
use anyhow::{Context, Result, ensure};
use ezk::{BoxedSourceCancelSafe, Frame, NextEventIsCancelSafe, Source, SourceEvent, ValueRange};
use ezk_rtp::{Rtp, RtpConfig, RtpConfigRange, RtpPacket, Session, rtcp_types::Compound};
use owo_colors::OwoColorize;
use sdp_types::SrtpSuite;
use srtp::{CryptoPolicy, StreamPolicy};
use std::{collections::HashMap, future::pending, io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    net::UdpSocket,
    sync::mpsc,
    time::{Instant, interval, interval_at},
};

pub struct RtpSessions {
    on_err: Arc<Box<dyn Fn() + Send + Sync>>,
    sessions: HashMap<MediaSessionId, Entry>,
}

#[derive(Debug)]
struct Entry {
    tx: mpsc::Sender<RtpTaskCommand>,
    has_sender: bool,
    has_receiver: bool,
}

impl RtpSessions {
    pub fn new(on_err: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            on_err: Arc::new(Box::new(on_err)),
            sessions: HashMap::new(),
        }
    }

    #[expect(clippy::too_many_arguments, clippy::similar_names)]
    pub async fn add_sender(
        &mut self,
        session_id: MediaSessionId,
        srtp: Option<&SrtpInfo>,
        clock_rate: u32,
        payload: u8,
        rtp_socket: Arc<UdpSocket>,
        rtcp_socket: Arc<UdpSocket>,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
        mut source: BoxedSourceCancelSafe<Rtp>,
    ) -> Result<()> {
        let entry = self.get_or_create_session(
            session_id,
            srtp,
            clock_rate,
            &rtp_socket,
            &rtcp_socket,
            remote_rtp_addr,
            remote_rtcp_addr,
        );

        ensure!(
            !entry.has_sender,
            "Tried to add sender to session which already has a sender"
        );

        entry.has_sender = true;

        source
            .negotiate_config(vec![RtpConfigRange {
                pt: ValueRange::Value(payload),
            }])
            .await
            .context("Failed to negotiate rtp source config")?;

        entry
            .tx
            .try_send(RtpTaskCommand::AddSender(source))
            .expect("just spawned the rtp task, send must succeed");

        Ok(())
    }

    pub async fn remove_sender(&mut self, session_id: MediaSessionId) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.has_sender = false;
            session
                .tx
                .send(RtpTaskCommand::RemoveSender)
                .await
                .ok()
                .context("failed to send remove-sender command to rtp-task")?;
        }

        Ok(())
    }

    #[expect(clippy::too_many_arguments, clippy::similar_names)]
    pub async fn add_receiver(
        &mut self,
        session_id: MediaSessionId,
        srtp: Option<&SrtpInfo>,
        clock_rate: u32,
        payload: u8,
        rtp_socket: Arc<UdpSocket>,
        rtcp_socket: Arc<UdpSocket>,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
    ) -> Result<RtpMpscSource> {
        let entry = self.get_or_create_session(
            session_id,
            srtp,
            clock_rate,
            &rtp_socket,
            &rtcp_socket,
            remote_rtp_addr,
            remote_rtcp_addr,
        );

        ensure!(
            !entry.has_receiver,
            "Tried to add receiver to session which already has a receiver"
        );

        entry.has_receiver = true;

        let (tx, rx) = mpsc::channel(8);

        entry
            .tx
            .send(RtpTaskCommand::AddReceiver(tx))
            .await
            .ok()
            .context("Failed to send command to rtp task")?;

        Ok(RtpMpscSource { rx, pt: payload })
    }

    pub async fn remove_receiver(&mut self, session_id: MediaSessionId) -> Result<()> {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.has_receiver = false;
            session
                .tx
                .send(RtpTaskCommand::RemoveReceiver)
                .await
                .ok()
                .context("failed to send remove-receiver command to rtp-task")?;
        }

        Ok(())
    }

    #[expect(clippy::too_many_arguments, clippy::similar_names)]
    fn get_or_create_session(
        &mut self,
        session_id: MediaSessionId,
        srtp: Option<&SrtpInfo>,
        clock_rate: u32,
        rtp_socket: &Arc<UdpSocket>,
        rtcp_socket: &Arc<UdpSocket>,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
    ) -> &mut Entry {
        self.sessions.entry(session_id).or_insert_with(|| {
            let srtp_sessions = if let Some(srtp) = srtp {
                let policy = srtp_suite_to_policy(&srtp.suite);

                let tx = srtp::Session::with_outbound_template(StreamPolicy {
                    rtp: policy,
                    rtcp: policy,
                    key: &srtp.send_key,
                    ..Default::default()
                })
                .unwrap();

                let rx = srtp::Session::with_inbound_template(StreamPolicy {
                    rtp: policy,
                    rtcp: policy,
                    key: &srtp.recv_key,
                    allow_repeat_tx: true,
                    ..Default::default()
                })
                .unwrap();

                Some((tx, rx))
            } else {
                None
            };

            let (tx, rx) = mpsc::channel(1);

            tokio::spawn(
                RtpTask::new(
                    rx,
                    self.on_err.clone(),
                    Session::new(
                        rand::random(),
                        clock_rate,
                        format!("opentalk-obelisk-{:x}", rand::random::<u64>()),
                    ),
                    srtp_sessions,
                    rtp_socket.clone(),
                    rtcp_socket.clone(),
                    remote_rtp_addr,
                    remote_rtcp_addr,
                )
                .run(),
            );

            Entry {
                tx,
                has_sender: false,
                has_receiver: false,
            }
        })
    }

    pub fn cleanup_sessions_without_sender_and_receiver(&mut self) {
        self.sessions.retain(|id, session| {
            if session.has_sender || session.has_receiver {
                true
            } else {
                log::debug!("remove session_id={id}");
                false
            }
        });
    }
}

fn srtp_suite_to_policy(suite: &SrtpSuite) -> CryptoPolicy {
    match suite {
        sdp_types::SrtpSuite::AES_CM_128_HMAC_SHA1_80 => {
            srtp::CryptoPolicy::aes_cm_128_hmac_sha1_80()
        }
        sdp_types::SrtpSuite::AES_CM_128_HMAC_SHA1_32 => {
            srtp::CryptoPolicy::aes_cm_128_hmac_sha1_32()
        }
        sdp_types::SrtpSuite::AES_192_CM_HMAC_SHA1_80 => {
            srtp::CryptoPolicy::aes_cm_192_hmac_sha1_80()
        }
        sdp_types::SrtpSuite::AES_192_CM_HMAC_SHA1_32 => {
            srtp::CryptoPolicy::aes_cm_192_hmac_sha1_32()
        }
        sdp_types::SrtpSuite::AES_256_CM_HMAC_SHA1_80 => {
            srtp::CryptoPolicy::aes_cm_256_hmac_sha1_80()
        }
        sdp_types::SrtpSuite::AES_256_CM_HMAC_SHA1_32 => {
            srtp::CryptoPolicy::aes_cm_256_hmac_sha1_32()
        }
        sdp_types::SrtpSuite::AEAD_AES_128_GCM => srtp::CryptoPolicy::aes_gcm_128_16_auth(),
        sdp_types::SrtpSuite::AEAD_AES_256_GCM => srtp::CryptoPolicy::aes_gcm_256_16_auth(),
        _ => unimplemented!(),
    }
}

enum RtpTaskCommand {
    AddSender(BoxedSourceCancelSafe<Rtp>),
    RemoveSender,

    AddReceiver(mpsc::Sender<RtpPacket>),
    RemoveReceiver,
}

const RECV_BUFFER_SIZE: usize = 65535;

struct RtpTask {
    state: State,
    on_err: Arc<Box<dyn Fn() + Send + Sync>>,

    /// Channel to receive commands from the `RtpSessions` object from. When this returns None, the Task quits
    commands_rx: mpsc::Receiver<RtpTaskCommand>,

    /// RTP Session
    session: Session,

    /// SRTP (send, receive) session, are used, when set, to protect/unprotect RTP traffic
    srtp_sessions: Option<(srtp::Session, srtp::Session)>,

    rtp_socket: Arc<UdpSocket>,
    rtcp_socket: Arc<UdpSocket>,
    remote_rtp_addr: SocketAddr,
    remote_rtcp_addr: SocketAddr,

    /// Reusable buffer to serialize RTP/RTCP packets into
    encode_buf: Vec<u8>,

    /// The RTP task polls this source yielding RTP packets, which are then sent out using the `rtp_socket` to `remote_rtp_addr`
    rtp_sender_source: Option<BoxedSourceCancelSafe<Rtp>>,

    /// Sink to dump RTP packets into, that have been received on the `rtp_socket`
    rtp_receiver_sink: Option<mpsc::Sender<RtpPacket>>,

    /// Timestamp we've last received a RTP or RTCP package from the caller
    timestamp_last_activity_from_peer: Instant,
}

enum State {
    Ok,
    ExitErr,
    ExitOk,
}

impl RtpTask {
    #[expect(clippy::too_many_arguments, clippy::similar_names)]
    fn new(
        commands_rx: mpsc::Receiver<RtpTaskCommand>,
        on_err: Arc<Box<dyn Fn() + Send + Sync>>,
        session: Session,
        srtp_sessions: Option<(srtp::Session, srtp::Session)>,
        rtp_socket: Arc<UdpSocket>,
        rtcp_socket: Arc<UdpSocket>,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
    ) -> Self {
        Self {
            state: State::Ok,
            on_err,
            commands_rx,
            session,
            srtp_sessions,
            rtp_socket,
            rtcp_socket,
            remote_rtp_addr,
            remote_rtcp_addr,
            encode_buf: Vec::new(),
            rtp_sender_source: None,
            rtp_receiver_sink: None,
            timestamp_last_activity_from_peer: Instant::now(),
        }
    }

    #[expect(clippy::similar_names)]
    async fn run(mut self) {
        let mut rtcp_interval = interval_at(
            Instant::now() + Duration::from_secs(5),
            Duration::from_secs(5),
        );

        // TODO: expand the jitterbuffer API to know when to poll
        let mut poll_jitterbuffer_interval = interval(Duration::from_millis(10));

        let mut rtp_recv_buf = vec![0u8; RECV_BUFFER_SIZE];
        let mut rtcp_recv_buf = vec![0u8; RECV_BUFFER_SIZE];

        while let State::Ok = self.state {
            // Reset send & receive buffers
            self.encode_buf.clear();
            rtp_recv_buf.resize(RECV_BUFFER_SIZE, 0);
            rtcp_recv_buf.resize(RECV_BUFFER_SIZE, 0);

            tokio::select! {
                // Wait for external commands
                command = self.commands_rx.recv() => self.handle_command(command),

                // Check the jitterbuffer in fixed duration
                _ = poll_jitterbuffer_interval.tick() => self.poll_jitterbuffer(),

                // Wait for the application to generate RTP packets and send them out
                event = poll_source(&mut self.rtp_sender_source) => self.handle_rtp_source_event(event).await,

                // Send out RTCP packets in a fixed interval
                _ = rtcp_interval.tick() => {
                    // Use the interval to check for activity timeout
                    if self.timestamp_last_activity_from_peer.elapsed() > Duration::from_secs(5) {
                        log::warn!("terminating call, no media was received for some time");
                        (self.on_err)();
                        return;
                    }

                    self.send_rtcp().await;
                },

                // Receive incoming RTP packets from the peer
                res = self.rtp_socket.recv_from(&mut rtp_recv_buf) => self.handle_rtp_packet_recv(&mut rtp_recv_buf, res),

                // Receive incoming RTCP packets from the peer
                res = self.rtcp_socket.recv_from(&mut rtcp_recv_buf) => self.handle_rtcp_packet_recv(&mut rtcp_recv_buf, res),
            }
        }

        match self.state {
            State::Ok => unreachable!(),
            State::ExitErr => {
                log::warn!("exited rtp task, due to error");
                (self.on_err)();
            }
            State::ExitOk => log::debug!("exited rtp task gracefully"),
        }
    }

    fn handle_command(&mut self, command: Option<RtpTaskCommand>) {
        use RtpTaskCommand::{AddReceiver, AddSender, RemoveReceiver, RemoveSender};

        match command {
            Some(AddSender(new_source)) => self.rtp_sender_source = Some(new_source),
            Some(RemoveSender) => self.rtp_sender_source = None,

            Some(AddReceiver(new_receiver)) => self.rtp_receiver_sink = Some(new_receiver),
            Some(RemoveReceiver) => self.rtp_receiver_sink = None,

            None => {
                log::debug!("commands_tx dropped, exiting rtp task");

                self.state = State::ExitOk;
            }
        }
    }

    fn poll_jitterbuffer(&mut self) {
        if let Some(tx) = &self.rtp_receiver_sink {
            while let Ok(permit) = tx.try_reserve() {
                if let Some(packet) = self.session.pop_rtp() {
                    permit.send(packet);
                } else {
                    break;
                }
            }
        } else {
            while self.session.pop_rtp().is_some() {}
        }
    }

    async fn handle_rtp_source_event(&mut self, event: ezk::Result<SourceEvent<Rtp>>) {
        let event = match event {
            Ok(event) => event,
            Err(e) => {
                log::error!("rtp task's source encountered an error, removing source - {e:?}");
                self.rtp_sender_source = None;
                return;
            }
        };

        let frame = match event {
            SourceEvent::Frame(frame) => frame,
            SourceEvent::RenegotiationNeeded => {
                unreachable!("rtp sources should not need renegotiation");
            }
            SourceEvent::EndOfData => {
                log::debug!("rtp source end of data, removing");
                self.rtp_sender_source = None;
                return;
            }
        };

        let mut packet = frame.into_data();
        let mut packet_mut = packet.get_mut();

        // Set missing packet header fields
        packet_mut.set_ssrc(self.session.ssrc());
        packet_mut
            .as_builder()
            .write_into_vec(&mut self.encode_buf)
            .expect("buffer of 65535 bytes must be large enough");

        self.session.send_rtp(&packet);

        // Turn the packet into SRTP packet, if configured
        if let Some((send_session, _)) = &mut self.srtp_sessions
            && let Err(e) = send_session.protect(&mut self.encode_buf)
        {
            log::error!(
                "Failed to protect RTP packet of len={}, {e}",
                self.encode_buf.len()
            );
            return;
        }

        if let Err(e) = self
            .rtp_socket
            .send_to(&self.encode_buf, self.remote_rtp_addr)
            .await
        {
            log::warn!(
                "Failed to send RTP packet of length={} to addr={}, {e}",
                self.encode_buf.len(),
                self.remote_rtp_addr
            );
            self.state = State::ExitErr;
        }
    }

    async fn send_rtcp(&mut self) {
        // make_rtcp needs a mut slice to write into, resize encode buf accordingly
        self.encode_buf.resize(65535, 0);
        let len = match self.session.write_rtcp_report(&mut self.encode_buf) {
            Ok(len) => len,
            Err(e) => {
                log::warn!("Failed to write RTCP packet, {e:?}");
                return;
            }
        };
        self.encode_buf.truncate(len);

        // Turn the packet into SRTCP packet, if configured
        if let Some((send_session, _)) = &mut self.srtp_sessions
            && let Err(e) = send_session.protect_rtcp(&mut self.encode_buf)
        {
            log::error!(
                "Failed to protect RTCP packet of len={}, {e}",
                self.encode_buf.len()
            );
            return;
        }

        if let Err(e) = self
            .rtcp_socket
            .send_to(&self.encode_buf, self.remote_rtcp_addr)
            .await
        {
            log::warn!(
                "Failed to send RTP packet of length={} to addr={}, {e}",
                self.encode_buf.len(),
                self.remote_rtp_addr
            );
            self.state = State::ExitErr;
        }
    }

    fn handle_rtp_packet_recv(&mut self, buf: &mut Vec<u8>, res: io::Result<(usize, SocketAddr)>) {
        let Some(len) = self.handle_recv_io_result(res) else {
            return;
        };

        buf.truncate(len);

        if let Some((_, recv_session)) = &mut self.srtp_sessions
            && let Err(e) = recv_session.unprotect(buf)
        {
            log::debug!("Failed to unprotect RTP, {e}");
            return;
        }

        self.timestamp_last_activity_from_peer = Instant::now();

        match RtpPacket::parse(&buf[..]) {
            Ok(pkg) => self.session.recv_rtp(pkg),
            Err(e) => log::debug!("Failed to parse rtp-packet, {e:?}"),
        }
    }

    fn handle_rtcp_packet_recv(&mut self, buf: &mut Vec<u8>, res: io::Result<(usize, SocketAddr)>) {
        let Some(len) = self.handle_recv_io_result(res) else {
            return;
        };

        buf.truncate(len);

        if let Some((_, recv_session)) = &mut self.srtp_sessions
            && let Err(e) = recv_session.unprotect_rtcp(buf)
        {
            log::debug!("Failed to unprotect RTCP, {e}");
            return;
        }

        self.timestamp_last_activity_from_peer = Instant::now();

        let compound = match Compound::parse(buf) {
            Ok(compound) => compound,
            Err(e) => {
                log::debug!("Failed to parse rtcp-packet, {e:?}");
                return;
            }
        };

        for pkg in compound {
            match pkg {
                Ok(packet) => {
                    log_report(&packet);
                    self.session.recv_rtcp(packet);
                }
                Err(e) => log::debug!("Encountered invalid RTCP packet in compound packet, {e:?}"),
            }
        }
    }

    fn handle_recv_io_result(&mut self, res: io::Result<(usize, SocketAddr)>) -> Option<usize> {
        match res {
            Ok((len, _)) => Some(len),
            Err(e) => {
                log::warn!("Failed to read from udpsocket, {e}");
                self.state = State::ExitErr;
                None
            }
        }
    }
}

async fn poll_source(
    source: &mut Option<BoxedSourceCancelSafe<Rtp>>,
) -> ezk::Result<SourceEvent<Rtp>> {
    match source {
        Some(source) => source.next_event().await,
        None => pending().await,
    }
}

pub(super) struct RtpMpscSource {
    pub(super) rx: mpsc::Receiver<RtpPacket>,
    pub(super) pt: u8,
}

impl Source for RtpMpscSource {
    type MediaType = Rtp;

    async fn capabilities(&mut self) -> ezk::Result<Vec<RtpConfigRange>> {
        Ok(vec![RtpConfigRange {
            pt: ValueRange::Value(self.pt),
        }])
    }

    async fn negotiate_config(&mut self, available: Vec<RtpConfigRange>) -> ezk::Result<RtpConfig> {
        assert!(available.iter().any(|r| r.pt.contains(&self.pt)));
        Ok(RtpConfig { pt: self.pt })
    }

    async fn next_event(&mut self) -> ezk::Result<SourceEvent<Rtp>> {
        match self.rx.recv().await {
            Some(packet) => Ok(SourceEvent::Frame(Frame::new(packet, 0))),
            None => Ok(SourceEvent::EndOfData),
        }
    }
}

impl NextEventIsCancelSafe for RtpMpscSource {}

fn log_report(packet: &ezk_rtp::rtcp_types::Packet<'_>) {
    let report_blocks = match packet {
        ezk_rtp::rtcp_types::Packet::Rr(receiver_report) => {
            Some(receiver_report.report_blocks().collect::<Vec<_>>())
        }
        ezk_rtp::rtcp_types::Packet::Sr(sender_report) => {
            Some(sender_report.report_blocks().collect::<Vec<_>>())
        }
        _ => None,
    };

    if let Some(report_blocks) = report_blocks {
        for report_block in report_blocks {
            log::debug!(
                "{}{}{}\n\tfraction_lost: {}; culmulative_lost: {}; extended_sequence_number: {}; interarrival_jitter: {}; last_sender_report_timestamp: {}; delay_since_last_sender_report_timestamp: {}",
                "[RTCP REPORT ssrc=".yellow(),
                report_block.ssrc().bright_yellow(),
                "]".yellow(),
                format!(
                    "{}%",
                    ((f32::from(report_block.fraction_lost()) / 255.0) * 100.0)
                )
                .bright_cyan(),
                report_block.cumulative_lost().bright_cyan(),
                report_block.extended_sequence_number().bright_cyan(),
                report_block.interarrival_jitter().bright_cyan(),
                report_block.last_sender_report_timestamp().bright_cyan(),
                report_block
                    .delay_since_last_sender_report_timestamp()
                    .bright_cyan()
            );
        }
    }
}
