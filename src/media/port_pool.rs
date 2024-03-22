// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings::Settings;
use anyhow::{anyhow, bail, Context, Result};
use gio::prelude::*;
use gio::traits::SocketExt;
use gio::{Cancellable, IOErrorEnum, InetSocketAddress};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rand::prelude::IteratorRandom;
use std::collections::BTreeSet;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use stun_types::attributes::{MappedAddress, Software, XorMappedAddress};
use stun_types::builder::MessageBuilder;
use stun_types::header::{Class, Method};
use stun_types::parse::ParsedMessage;

static INSTANCE: OnceCell<PortPool> = OnceCell::new();

// Contains the rtp as well as the related rtcp socket
pub struct SocketPair {
    pub rtp_port: u16,
    pub rtp_udpsink: gst::Element,
    pub rtp_udpsrc: gst::Element,

    pub rtcp_port: u16,
    pub rtcp_udpsink: gst::Element,
    pub rtcp_udpsrc: gst::Element,
}

/// Manages the Obelisks SIP RTP/RTCP port pool
///
/// Tracks the RTP/RTCP ports that are used by the obelisk. Allows the creation
/// of [`SocketPairs`](SocketPair), the sockets will only bind to the ports in the
/// specified port range.
#[derive(Debug)]
pub struct PortPool {
    settings: Arc<Settings>,

    /// Start of the port range
    port_start: u16,
    /// The number of port pairs
    port_pairs: u16,
    /// A list of port pairs known to be used by the obelisk
    used_pairs: Mutex<BTreeSet<u16>>,
}

impl PortPool {
    /// Get the global [`PortPool`] instance
    pub fn instance() -> &'static Self {
        INSTANCE.get().expect("PortManager was not initialized")
    }

    /// Initialize the [`PortPool`] instance with the provided range
    pub fn init(settings: Arc<Settings>) {
        let port_start = settings.sip.rtp_port_range.start;
        let port_end = settings.sip.rtp_port_range.end;

        let port_pairs = (port_end - port_start + 1) / 2;

        log::trace!(
            "Obelisk will use port {} - {} for SIP RTP/RTCP connections, {} available pairs",
            port_start,
            port_end,
            port_pairs
        );

        INSTANCE
            .set(Self {
                settings,
                port_start,
                port_pairs,
                used_pairs: Mutex::new(BTreeSet::new()),
            })
            .expect("PortManager was already initialized");
    }

    /// Creates a new pair of RTP/RTCP UDP sockets
    ///
    /// Draws a random port pair (even, even+1) from the pool, ignoring the currently known used
    /// port pairs. If the drawn pair is currently in use by another unknown process, the pair is
    /// cached and also removed from the possible pool for the purpose of drawing another one.
    /// This avoids contention under heavy load (many open connections)
    ///
    /// Returns Ok(None) when no more ports are available
    pub fn create_rtp_socket_pair(&self) -> Result<Option<SocketPair>> {
        let mut rng = rand::thread_rng();

        let mut failed_pairs = vec![];

        let mut used_pairs = self.used_pairs.lock();

        loop {
            // get a random available port pair
            let pair = (0..self.port_pairs)
                .filter(|p| !used_pairs.contains(p))
                .filter(|p| !failed_pairs.contains(p))
                .choose(&mut rng);

            // return if no available ports were found
            let pair = match pair {
                Some(pair) => pair,
                None => return Ok(None),
            };

            // get the actual port values for the pair
            let rtp_port = self.port_start + (pair * 2);
            let rtcp_port = rtp_port + 1;

            let rtp_socket = match create_udp_socket(rtp_port)? {
                Some(rtp_socket) => rtp_socket,
                None => {
                    failed_pairs.push(pair);
                    continue;
                }
            };

            let rtcp_socket = match create_udp_socket(rtcp_port)? {
                Some(rtcp_socket) => rtcp_socket,
                None => {
                    failed_pairs.push(pair);
                    continue;
                }
            };

            let (rtp_port, rtcp_port) = if let Some(stun_server) = &self.settings.sip.stun_server {
                let rtp_addr = resolve_public_addr(&rtp_socket, stun_server)
                    .context("Failed to discover public addr of RTP socket using STUN")?;
                let rtcp_addr = resolve_public_addr(&rtcp_socket, stun_server)
                    .context("Failed to discover public addr of RTCP socket using STUN")?;

                (rtp_addr.port(), rtcp_addr.port())
            } else {
                (rtp_port, rtcp_port)
            };

            log::trace!(
                "Using port {} & {} for new RTP/RTCP connection (pair {}),",
                rtp_port,
                rtcp_port,
                pair
            );

            used_pairs.insert(pair);

            return Ok(Some(SocketPair {
                rtp_port,
                rtp_udpsink: gst::ElementFactory::make("udpsink")
                    .property("socket", rtp_socket.clone())
                    .property("sync", false)
                    .build()?,
                rtp_udpsrc: gst::ElementFactory::make("udpsrc")
                    .property("socket", rtp_socket)
                    .build()?,

                rtcp_port,
                rtcp_udpsink: gst::ElementFactory::make("udpsink")
                    .property("socket", rtcp_socket.clone())
                    .property("sync", false)
                    .build()?,
                rtcp_udpsrc: gst::ElementFactory::make("udpsrc")
                    .property("socket", rtcp_socket)
                    .build()?,
            }));
        }
    }

    /// Removes the port-pair related to the provided port
    pub fn clear_port(&self, port: u16) {
        let mut used_pairs = self.used_pairs.lock();

        let pair = (port - self.port_start) / 2;

        used_pairs.remove(&pair);

        log::trace!("Removed port {} from PortPool (pair {})", port, pair);
    }
}

/// Creates a new udp socket with the provided port
///
/// Returns Ok(None) if the provided port is already in use
fn create_udp_socket(port: u16) -> Result<Option<gio::Socket>> {
    let socket = gio::Socket::new(
        gio::SocketFamily::Ipv4,
        gio::SocketType::Datagram,
        gio::SocketProtocol::Udp,
    )?;

    let addr = gio::InetAddress::new_any(gio::SocketFamily::Ipv4);

    let addr = gio::InetSocketAddress::new(&addr, port);

    if let Err(error) = socket.bind(&addr, false) {
        if let Some(IOErrorEnum::AddressInUse) = error.kind() {
            return Ok(None);
        } else {
            return Err(anyhow!(error));
        }
    }

    Ok(Some(socket))
}

fn resolve_public_addr(sock: &gio::Socket, stun_server: &str) -> Result<SocketAddr> {
    // Wait max 1 sec for a packet before returning an error
    let prev_timeout = sock.timeout();
    sock.set_timeout(1);

    // Resolve stun server address
    let server = stun_server
        .to_socket_addrs()
        .with_context(|| format!("Failed to resolve stun-server {stun_server:?}"))?
        .map(InetSocketAddress::from);

    // Build stun request
    let mut msg = MessageBuilder::new(Class::Request, Method::Binding, rand::random());
    msg.add_attr(&Software::new("opentalk-obelisk")).unwrap();
    let msg = msg.finish();

    let mut last_err = anyhow!("hostname {stun_server:?} didn't provide any addresses");

    for server in server {
        let addr = match send_and_receive_stun(sock, server, &msg) {
            Ok(addr) => addr,
            Err(e) => {
                log::warn!("Failed to resolve public address of created rtp/rtcp port, {e:?}");
                last_err = e;
                continue;
            }
        };

        // Restore previous timeout
        sock.set_timeout(prev_timeout);

        return Ok(addr);
    }

    // Restore previous timeout
    sock.set_timeout(prev_timeout);

    Err(last_err)
}

fn send_and_receive_stun(
    sock: &gio::Socket,
    server: InetSocketAddress,
    msg: &Vec<u8>,
) -> Result<SocketAddr> {
    let mut recv_buffer = vec![0u8; 1500];

    sock.send_to(Some(&server), msg, Cancellable::NONE)
        .context("Failed to send request")?;

    let len = sock
        .receive(&mut recv_buffer, Cancellable::NONE)
        .context("Failed to receive response")?;

    recv_buffer.truncate(len);

    let mut msg = ParsedMessage::parse(recv_buffer).context("Failed to parse response")?;

    if let Some(Ok(attr)) = msg.get_attr::<XorMappedAddress>() {
        return Ok(attr.0);
    }

    if let Some(Ok(attr)) = msg.get_attr::<MappedAddress>() {
        return Ok(attr.0);
    }

    bail!("Stun response didn't contain a mapped-address attribute")
}
