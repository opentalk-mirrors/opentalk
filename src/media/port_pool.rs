// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::anyhow;
use anyhow::Result;
use gio::traits::SocketExt;
use gio::IOErrorEnum;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rand::prelude::IteratorRandom;
use std::collections::BTreeSet;

static INSTANCE: OnceCell<PortPool> = OnceCell::new();

// Contains the rtp as well as the related rtcp socket
pub struct SocketPair {
    pub rtp_socket: gio::Socket,
    pub rtp_port: u16,
    pub rtcp_socket: gio::Socket,
    pub rtcp_port: u16,
}

/// Manages the Obelisks SIP RTP/RTCP port pool
///
/// Tracks the RTP/RTCP ports that are used by the obelisk. Allows the creation
/// of [`SocketPairs`](SocketPair), the sockets will only bind to the ports in the
/// specified port range.
#[derive(Debug)]
pub struct PortPool {
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
    pub fn init(port_start: u16, port_end: u16) {
        let port_pairs = (port_end - port_start + 1) / 2;

        log::trace!(
            "Obelisk will use port {} - {} for SIP RTP/RTCP connections, {} available pairs",
            port_start,
            port_end,
            port_pairs
        );

        INSTANCE
            .set(Self {
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

            log::trace!(
                "Using port {} & {} for new RTP/RTCP connection (pair {}),",
                rtp_port,
                rtcp_port,
                pair
            );

            used_pairs.insert(pair);
            return Ok(Some(SocketPair {
                rtp_socket,
                rtp_port,
                rtcp_socket,
                rtcp_port,
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
