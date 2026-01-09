// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings::Settings;
use anyhow::{Context, Result, anyhow, bail};
use once_cell::sync::OnceCell;
use rand::prelude::IteratorRandom;
use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use stun_types::attributes::{MappedAddress, Software, XorMappedAddress};
use stun_types::{Class, Message, MessageBuilder, Method, TransactionId};
use tokio::net::{UdpSocket, lookup_host};
use tokio::sync::Mutex;
use tokio::time::timeout;

static INSTANCE: OnceCell<PortPool> = OnceCell::new();

#[derive(Debug, Clone, Copy)]
pub enum AddressFamily {
    V4,
    V6,
}

impl AddressFamily {
    pub fn of(ip: IpAddr) -> AddressFamily {
        match ip {
            IpAddr::V4(_) => AddressFamily::V4,
            IpAddr::V6(_) => AddressFamily::V6,
        }
    }
}

// Contains the rtp as well as the related rtcp socket
pub struct SocketPair {
    /// Local UDP port on which RTP is sent/received
    pub local_rtp_port: u16,
    /// public RTP port resolved using the configured STUN server, or if none is set, same as `local_rtp_port`
    pub public_rtp_port: u16,
    /// public RTCP port, same as `public_rtp_port`
    pub public_rtcp_port: u16,

    pub rtp_socket: Arc<UdpSocket>,
    pub rtcp_socket: Arc<UdpSocket>,
}

impl Drop for SocketPair {
    fn drop(&mut self) {
        let local_rtp_port = self.local_rtp_port;

        tokio::spawn(async move {
            PortPool::instance().clear_port(local_rtp_port).await;
        });
    }
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

        let port_pairs = (port_end - port_start).div_ceil(2);

        log::trace!(
            "Obelisk will use port {port_start} - {port_end} for SIP RTP/RTCP connections, {port_pairs} available pairs"
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
    #[expect(clippy::similar_names)]
    pub async fn create_rtp_socket_pair(
        &self,
        address_family: AddressFamily,
    ) -> Result<Option<SocketPair>> {
        let mut failed_pairs = vec![];

        let mut used_pairs = self.used_pairs.lock().await;

        loop {
            // get a random available port pair
            let pair = (0..self.port_pairs)
                .filter(|p| !used_pairs.contains(p))
                .filter(|p| !failed_pairs.contains(p))
                .choose(&mut rand::rng());

            // return if no available ports were found
            let Some(pair) = pair else { return Ok(None) };

            // get the actual port values for the pair
            let local_rtp_port = self.port_start + (pair * 2);
            let local_rtcp_port = local_rtp_port + 1;

            let Some(rtp_socket) = create_udp_socket(local_rtp_port, address_family).await? else {
                failed_pairs.push(pair);
                continue;
            };

            let Some(rtcp_socket) = create_udp_socket(local_rtcp_port, address_family).await?
            else {
                failed_pairs.push(pair);
                continue;
            };

            log::trace!(
                "Using local port {local_rtp_port} & {local_rtcp_port} for new RTP/RTCP connection (pair {pair})"
            );

            let (stun_rtp_port, stun_rtcp_port) =
                if let Some(stun_server) = &self.settings.sip.stun_server {
                    let rtp_addr = resolve_public_addr(&rtp_socket, stun_server)
                        .await
                        .context("Failed to discover public addr of RTP socket using STUN")?;
                    let rtcp_addr = resolve_public_addr(&rtcp_socket, stun_server)
                        .await
                        .context("Failed to discover public addr of RTCP socket using STUN")?;

                    log::trace!(
                        "Resolved public port using STUN for RTP & RTCP - ({} & {})",
                        rtp_addr.port(),
                        rtcp_addr.port()
                    );

                    (rtp_addr.port(), rtcp_addr.port())
                } else {
                    (local_rtp_port, local_rtcp_port)
                };

            used_pairs.insert(pair);

            return Ok(Some(SocketPair {
                local_rtp_port,
                public_rtp_port: stun_rtp_port,
                public_rtcp_port: stun_rtcp_port,
                rtp_socket: Arc::new(rtp_socket),
                rtcp_socket: Arc::new(rtcp_socket),
            }));
        }
    }

    /// Removes the port-pair related to the provided port
    pub async fn clear_port(&self, port: u16) {
        let mut used_pairs = self.used_pairs.lock().await;

        let pair = (port - self.port_start) / 2;

        if used_pairs.remove(&pair) {
            log::trace!("Removed port {port} from PortPool (pair {pair})");
        } else {
            log::error!("Failed to remove port pair {pair}, used_pairs={used_pairs:?}");
        }
    }
}

/// Creates a new udp socket with the provided port
///
/// Returns Ok(None) if the provided port is already in use
async fn create_udp_socket(port: u16, address_family: AddressFamily) -> Result<Option<UdpSocket>> {
    let ip = match address_family {
        AddressFamily::V4 => Ipv4Addr::UNSPECIFIED.into(),
        AddressFamily::V6 => Ipv6Addr::UNSPECIFIED.into(),
    };

    match UdpSocket::bind(SocketAddr::new(ip, port)).await {
        Ok(socket) => Ok(Some(socket)),
        Err(e) if e.kind() == ErrorKind::AddrInUse => Ok(None),
        Err(e) => Err(anyhow!(e)),
    }
}

async fn resolve_public_addr(sock: &UdpSocket, stun_server: &str) -> Result<SocketAddr> {
    // Resolve stun server address
    let server = lookup_host(stun_server)
        .await
        .with_context(|| format!("Failed to resolve stun-server {stun_server:?}"))?;

    // Build stun request
    let mut msg = MessageBuilder::new(Class::Request, Method::Binding, TransactionId::random());
    msg.padding_in_value_len(true);
    msg.add_attr(Software::new("opentalk-obelisk"));
    let msg = msg.finish();

    let mut last_err = anyhow!("hostname {stun_server:?} didn't provide any addresses");

    for server in server {
        let addr = match send_and_receive_stun(sock, server, &msg).await {
            Ok(addr) => addr,
            Err(e) => {
                log::warn!("Failed to resolve public address of created rtp/rtcp port, {e:?}");
                last_err = e;
                continue;
            }
        };

        return Ok(addr);
    }

    Err(last_err)
}

async fn send_and_receive_stun(
    sock: &UdpSocket,
    server: SocketAddr,
    msg: &[u8],
) -> Result<SocketAddr> {
    let mut recv_buffer = vec![0u8; 1500];

    for i in 1..=8 {
        sock.send_to(msg, server)
            .await
            .context("Failed to send request")?;

        let recv = sock.recv(&mut recv_buffer);

        let len = match timeout(Duration::from_millis(50) * i, recv).await {
            Ok(Ok(len)) => len,
            Ok(Err(e)) => return Err(e).context("Failed to receive stun response")?,
            Err(_) => continue,
        };

        recv_buffer.truncate(len);

        let mut msg = Message::parse(recv_buffer).context("Failed to parse response")?;

        if let Some(Ok(attr)) = msg.attribute::<XorMappedAddress>() {
            return Ok(attr.0);
        }

        if let Some(Ok(attr)) = msg.attribute::<MappedAddress>() {
            return Ok(attr.0);
        }

        bail!("Stun response did not contain a mapped-address attribute")
    }

    bail!("Did not receive an answer from STUN server")
}
