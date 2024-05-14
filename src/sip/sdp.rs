// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::codec::SdpCodecInfo;
use crate::media::codec::MediaType;
use crate::media::port_pool::{PortPool, SocketPair};
use crate::settings::Settings;
use crate::sip::codec::{available_video_codec, choose_audio_codec, choose_video_codec};
use anyhow::{bail, Context, Result};
use bytes::Bytes;
use bytesstr::BytesStr;
use sdp_types::attributes::fmtp::Fmtp;
use sdp_types::{
    attributes::{direction::Direction, rtcp::RtcpAttr, rtpmap::RtpMap},
    connection::Connection,
    media::{MediaDescription, MediaType as SdpMediaType, TransportProtocol},
    msg::{MediaScope, Message},
    origin::Origin,
    time::Time,
    TaggedAddress,
};
use std::{
    io,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    num::Wrapping,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct SessionState {
    settings: Arc<Settings>,
    pub_ip: IpAddr,

    id: u64,
    version: u64,

    next_media_session_id: Wrapping<u32>,

    media_sessions: Vec<MediaSession>,
}

pub struct MediaSession {
    // Internal id to avoid doubly matching a media session against a new offer
    id: u32,

    // Data to match against media streams in incoming offers
    media_type: MediaType,
    pub remote_rtp_addr: SocketAddr,
    pub remote_rtcp_addr: SocketAddr,

    local_rtp_addr: SocketAddr,
    local_rtcp_addr: SocketAddr,
    direction: Direction,

    // media ession's TELEPHONE-EVENT
    dtmf_events: Option<DtmfEvents>,

    pub socket_pair: SocketPair,
    pub codec: SdpCodecInfo,
}

struct DtmfEvents {
    pt: u32,
}

pub enum MediaChange {
    AddReceiver(u32),
    RemoveReceiver(u32),

    AddSender(u32),
    RemoveSender(u32),
}

impl SessionState {
    pub fn empty(settings: Arc<Settings>, pub_ip: IpAddr) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            settings,
            pub_ip,

            id: now,
            version: now + 1,

            next_media_session_id: Wrapping::default(),
            media_sessions: vec![],
        }
    }

    fn make_media_session_id(&mut self) -> u32 {
        let id = self.next_media_session_id;
        self.next_media_session_id += 1;
        id.0
    }

    pub fn media_by_id(&self, id: u32) -> &MediaSession {
        self.media_sessions
            .iter()
            .find(|m| m.id == id)
            .expect("Invalid media id")
    }

    pub fn has_video(&self) -> bool {
        self.media_sessions
            .iter()
            .any(|m| m.media_type == MediaType::Video)
    }

    pub fn receives_video(&self) -> bool {
        self.media_sessions
            .iter()
            .any(|m| is_receiving(m.direction) && m.media_type == MediaType::Video)
    }

    /// Iterate over all payload numbers that may appear as dtmf tones
    pub fn all_dtmf_payloads(&self) -> impl Iterator<Item = u32> + '_ {
        self.media_sessions
            .iter()
            .flat_map(|m| m.dtmf_events.as_ref().map(|e| e.pt))
    }

    pub fn receive_offer_bytes(&mut self, offer: Bytes) -> Result<Vec<MediaChange>> {
        let offer =
            BytesStr::from_utf8_bytes(offer).context("SDP contains invalid UTF-8 characters")?;
        let offer = sdp_types::msg::parse::<sdp_types::msg::Builder>(&offer)
            .with_context(|| format!("Failed to parse SDP Offer:\n{offer}"))?;

        self.receive_offer(offer)
    }

    pub fn receive_offer(&mut self, offer: Message) -> Result<Vec<MediaChange>> {
        // track which of the current media sessions have been already matched against the new ones
        let mut matched_media_sessions = vec![];

        let mut changes = vec![];

        for offer_media in offer.media_scopes {
            // Ignore inactive and invalid media
            if offer_media.desc.port == 0
                || offer_media.desc.fmts.is_empty()
                || matches!(offer_media.direction, Direction::Inactive)
            {
                continue;
            }

            let (remote_rtp_addr, remote_rtcp_addr) =
                remote_peer_addresses(offer.connection.as_ref(), &offer_media)?;

            let new_direction =
                flip_direction(offer_media.direction, remote_rtp_addr.ip().is_unspecified());

            // match against existing media session
            let matched_existing_media_session =
                self.media_sessions.iter().find(|existing_session| {
                    // Avoid double matching
                    if matched_media_sessions.contains(&existing_session.id) {
                        return false;
                    }

                    let same_media_type =
                        existing_session.media_type == offer_media.desc.media_type;

                    let addr_matches = existing_session.remote_rtp_addr == remote_rtp_addr
                        && existing_session.remote_rtcp_addr == remote_rtcp_addr;

                    let negotiated_codec_still_available = offer_media
                        .desc
                        .fmts
                        .contains(&existing_session.codec.gst_elements.payload);

                    same_media_type && addr_matches && negotiated_codec_still_available
                });

            let (id, existing_direction) =
                if let Some(matched_existing_media_session) = matched_existing_media_session {
                    matched_media_sessions.push(matched_existing_media_session.id);

                    (
                        matched_existing_media_session.id,
                        matched_existing_media_session.direction,
                    )
                } else {
                    // New media session, find suitable codec

                    match self.create_new_session(
                        &offer_media,
                        &mut matched_media_sessions,
                        remote_rtp_addr,
                        remote_rtcp_addr,
                        new_direction,
                    ) {
                        Ok((id, direction)) => (id, direction),
                        Err(e) => {
                            log::warn!("Failed to create new media session, {e:?}");
                            continue;
                        }
                    }
                };

            if is_sending(existing_direction) && !is_sending(new_direction) {
                changes.push(MediaChange::RemoveSender(id));
            } else if !is_sending(existing_direction) && is_sending(new_direction) {
                changes.push(MediaChange::AddSender(id));
            }

            if is_receiving(existing_direction) && !is_receiving(new_direction) {
                changes.push(MediaChange::RemoveReceiver(id));
            } else if !is_receiving(existing_direction) && is_receiving(new_direction) {
                changes.push(MediaChange::AddReceiver(id));
            }
        }

        // Remove any unmatched media sessions
        self.media_sessions.retain(|m| {
            if matched_media_sessions.contains(&m.id) {
                return true;
            }

            if is_sending(m.direction) {
                changes.push(MediaChange::RemoveSender(m.id));
            }

            if is_receiving(m.direction) {
                changes.push(MediaChange::RemoveReceiver(m.id));
            }

            false
        });

        Ok(changes)
    }

    fn create_new_session(
        &mut self,
        offer_media: &MediaScope,
        matched_media_sessions: &mut Vec<u32>,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
        new_direction: Direction,
    ) -> Result<(u32, Direction)> {
        let (media_type, codec) = match offer_media.desc.media_type {
            SdpMediaType::Audio => {
                let codec = choose_audio_codec(offer_media)
                    .context("No compatible audio codec in offer")?;

                (MediaType::Audio, codec)
            }
            SdpMediaType::Video => {
                let Some(codec) = choose_video_codec(&self.settings, offer_media) else {
                    bail!(
                        "No compatible video codec found (supported is: {}), continuing without video",
                        available_video_codec()
                    );
                };
                (MediaType::Video, codec)
            }
            ty => {
                bail!("Skipping unexpected offered media {ty}");
            }
        };

        let id = self.make_media_session_id();

        matched_media_sessions.push(id);

        let socket_pair = PortPool::instance()
            .create_rtp_socket_pair()
            .context("Failed to create RTP/RTCP sockets")?
            .context("No more RTP/RTCP ports are available")?;

        let local_rtp_addr = SocketAddr::new(self.pub_ip, socket_pair.rtp_port);
        let local_rtcp_addr = SocketAddr::new(self.pub_ip, socket_pair.rtcp_port);

        let dtmf_events = offer_media
            .rtpmaps
            .iter()
            .find(|rtpmap| {
                rtpmap.encoding.eq_ignore_ascii_case("telephone-event") && rtpmap.clock_rate == 8000
            })
            .map(|rtpmap| DtmfEvents { pt: rtpmap.payload });

        self.media_sessions.push(MediaSession {
            id,
            media_type,
            remote_rtp_addr,
            remote_rtcp_addr,
            local_rtp_addr,
            local_rtcp_addr,
            direction: new_direction,
            socket_pair,
            dtmf_events,
            codec,
        });

        Ok((id, Direction::Inactive))
    }

    pub fn build_sdp(&self) -> Message {
        let media_scopes = self
            .media_sessions
            .iter()
            .map(|media_session| {
                let mut rtpmaps = vec![media_session.codec.rtpmap.clone()];
                let mut fmtps = Vec::from_iter(media_session.codec.fmtp.clone());
                let mut fmts = vec![media_session.codec.gst_elements.payload];

                if let Some(dtmf_events) = &media_session.dtmf_events {
                    rtpmaps.push(RtpMap {
                        payload: dtmf_events.pt,
                        encoding: "telephone-event".into(),
                        clock_rate: 8000,
                        params: None,
                    });

                    fmtps.push(Fmtp {
                        format: dtmf_events.pt,
                        params: "0-12".into(),
                    });

                    fmts.push(dtmf_events.pt);
                }

                MediaScope {
                    desc: MediaDescription {
                        media_type: media_session.media_type.into(),
                        port: media_session.local_rtp_addr.port(),
                        ports_num: None,
                        proto: TransportProtocol::RtpAvp,
                        fmts,
                    },
                    direction: media_session.direction,
                    connection: Some(Connection {
                        address: media_session.local_rtp_addr.ip().into(),
                        ttl: None,
                        num: None,
                    }),
                    bandwidth: vec![],
                    rtcp_attr: Some(RtcpAttr {
                        port: media_session.local_rtcp_addr.port(),
                        address: None,
                    }),
                    rtpmaps,
                    fmtps,
                    ice_ufrag: None,
                    ice_pwd: None,
                    ice_candidates: vec![],
                    ice_end_of_candidates: false,
                    attributes: vec![],
                }
            })
            .collect();

        Message {
            name: BytesStr::from_static("opentalk-obelisk"),
            origin: Origin {
                username: BytesStr::from_static("-"),
                session_id: self.id.to_string().into(),
                session_version: self.version.to_string().into(),
                address: self.pub_ip.into(),
            },
            time: Time { start: 0, stop: 0 },
            direction: Direction::SendRecv,
            connection: None,
            bandwidth: vec![],
            ice_options: Default::default(),
            ice_lite: false,
            ice_ufrag: None,
            ice_pwd: None,
            attributes: vec![],
            media_scopes,
        }
    }
}

fn is_sending(direction: Direction) -> bool {
    matches!(direction, Direction::SendOnly | Direction::SendRecv)
}

fn is_receiving(direction: Direction) -> bool {
    matches!(direction, Direction::RecvOnly | Direction::SendRecv)
}

fn flip_direction(direction: Direction, origin_unspecified: bool) -> Direction {
    match direction {
        Direction::SendRecv if origin_unspecified => Direction::RecvOnly,
        Direction::SendRecv => Direction::SendRecv,
        Direction::RecvOnly if origin_unspecified => Direction::Inactive,
        Direction::RecvOnly => Direction::SendOnly,
        Direction::SendOnly => Direction::RecvOnly,
        Direction::Inactive => Direction::Inactive,
    }
}

fn remote_peer_addresses(
    connection: Option<&Connection>,
    media: &MediaScope,
) -> Result<(SocketAddr, SocketAddr)> {
    let peer_rtp_port = media.desc.port;
    let peer_rtcp_port = media
        .rtcp_attr
        .as_ref()
        .map(|rtcp| rtcp.port)
        .unwrap_or_else(|| peer_rtp_port + 1);
    let connection = media
        .connection
        .as_ref()
        .or(connection)
        .context("Missing connection attribute in offer")?;
    let peer_rtp_addr = tagged_to_socket_addr(&connection.address, peer_rtp_port)?;
    let peer_rtcp_addr = SocketAddr::new(peer_rtp_addr.ip(), peer_rtcp_port);
    Ok((peer_rtp_addr, peer_rtcp_addr))
}

fn tagged_to_socket_addr(tagged: &TaggedAddress, port: u16) -> io::Result<SocketAddr> {
    let addr = match tagged {
        TaggedAddress::IP4(ip) => SocketAddr::new(IpAddr::V4(*ip), port),
        TaggedAddress::IP6(ip) => SocketAddr::new(IpAddr::V6(*ip), port),
        TaggedAddress::IP4FQDN(name) | TaggedAddress::IP6FQDN(name) => format!("{}:{}", name, port)
            .to_socket_addrs()?
            .next()
            .expect("empty iterator"),
    };

    Ok(addr)
}
