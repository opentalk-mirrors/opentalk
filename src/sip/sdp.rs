// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::codec::{choose_audio_codec, choose_video_codec, GstCodecInfo};
use crate::media::codec::MediaType;
use crate::media::port_pool::PortPool;
use crate::settings::Settings;
use crate::sip::codec::available_video_codec;
use anyhow::{Context, Result};
use bytes::Bytes;
use bytesstr::BytesStr;
use sdp_types::attributes::direction::Direction;
use sdp_types::attributes::ice;
use sdp_types::attributes::rtcp::RtcpAttr;
use sdp_types::connection::Connection;
use sdp_types::media::{MediaDescription, MediaType as SdpMediaType, TransportProtocol};
use sdp_types::msg::MediaScope;
use sdp_types::msg::Message;
use sdp_types::origin::Origin;
use sdp_types::time::Time;
use sdp_types::TaggedAddress;
use std::io;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::time::{SystemTime, UNIX_EPOCH};

/// The SDP session info used to construct the tel_bin
pub struct SessionInfo {
    pub local_sdp: Message,

    pub remote_id: u64,
    pub remote_version: u64,

    pub telephone_event_pt: u32,
    pub media: Vec<SessionMedia>,
}

pub struct SessionMedia {
    pub media_type: MediaType,
    pub direction: Direction,

    pub rtp_udpsrc: gst::Element,
    pub rtp_udpsink: gst::Element,
    pub rtcp_udpsrc: gst::Element,
    pub rtcp_udpsink: gst::Element,

    pub remote_rtp_addr: SocketAddr,
    pub remote_rtcp_addr: SocketAddr,
    pub gst_elements: GstCodecInfo,
}

impl SessionInfo {
    pub fn create_from_offer(settings: &Settings, offer: Bytes, pub_ip: IpAddr) -> Result<Self> {
        let offer = BytesStr::from_utf8_bytes(offer)
            .context("INVITE body contains invalid UTF-8 characters")?;
        let offer = sdp_types::msg::parse::<sdp_types::msg::Builder>(&offer)
            .with_context(|| format!("Failed to parse SDP Offer:\n{offer}"))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = now;
        let version = now + 1;

        respond(settings, offer, pub_ip, id, version)
    }
}

/// Negotiate audio & video codecs
fn respond(
    settings: &Settings,
    offer: Message,
    pub_ip: IpAddr,
    id: u64,
    version: u64,
) -> Result<SessionInfo> {
    let port_pool = PortPool::instance();

    let remote_id: u64 = offer
        .origin
        .session_id
        .parse()
        .context("Got non-u64 session id in offer")?;

    let remote_version: u64 = offer
        .origin
        .session_version
        .parse()
        .context("Got non-u64 session version in offer")?;

    let mut media_scopes = vec![];
    let mut session_media = vec![];
    let mut telephone_event_pt = None;

    for mut media in offer.media_scopes {
        // Check if the media is disabled
        if media.desc.port == 0
            || media.desc.fmts.is_empty()
            || matches!(media.direction, Direction::Inactive)
        {
            continue;
        }

        let (media_type, codec_info) = match media.desc.media_type {
            SdpMediaType::Audio => {
                let codec =
                    choose_audio_codec(&media).context("No compatible audio codec in offer")?;

                (MediaType::Audio, codec)
            }
            SdpMediaType::Video => {
                let Some(codec) = choose_video_codec(settings, &media) else {
                    log::debug!("No compatible video codec found (supported is: {}), continuing without video", available_video_codec());
                    continue;
                };
                (MediaType::Video, codec)
            }
            ty => {
                log::warn!("Skipping unexpected offered media {ty}");
                continue;
            }
        };

        let peer_rtp_port = media.desc.port;
        let peer_rtcp_port = media
            .rtcp_attr
            .map(|rtcp| rtcp.port)
            .unwrap_or_else(|| peer_rtp_port + 1);

        let connection = media
            .connection
            .or(offer.connection.clone())
            .context("Missing connection attribute in offer")?;

        let peer_rtp_addr = tagged_to_socket_addr(&connection.address, peer_rtp_port)?;
        let peer_rtcp_addr = SocketAddr::new(peer_rtp_addr.ip(), peer_rtcp_port);

        let origin_unspecified = peer_rtp_addr.ip().is_unspecified();

        let direction = match media.direction {
            Direction::SendRecv if origin_unspecified => Direction::RecvOnly,
            Direction::SendRecv => Direction::SendRecv,
            Direction::RecvOnly if origin_unspecified => Direction::Inactive,
            Direction::RecvOnly => Direction::SendOnly,
            Direction::SendOnly => Direction::RecvOnly,
            Direction::Inactive => Direction::Inactive,
        };

        let socket_pair = port_pool
            .create_rtp_socket_pair()
            .context("Failed to create RTP/RTCP sockets")?
            .context("No more RTP/RTCP ports are available")?;

        let local_rtp_addr = SocketAddr::new(pub_ip, socket_pair.rtp_port);
        let local_rtcp_addr = SocketAddr::new(pub_ip, socket_pair.rtcp_port);

        let mut answer_media = MediaScope {
            desc: MediaDescription {
                media_type: media.desc.media_type,
                port: local_rtp_addr.port(),
                ports_num: None,
                proto: TransportProtocol::RtpAvp,
                fmts: vec![codec_info.rtpmap.payload],
            },
            direction,
            connection: Some(Connection {
                address: local_rtp_addr.ip().into(),
                ttl: None,
                num: None,
            }),
            bandwidth: vec![],
            rtcp_attr: Some(RtcpAttr {
                port: local_rtcp_addr.port(),
                address: Some(local_rtcp_addr.ip().into()),
            }),
            rtpmaps: vec![codec_info.rtpmap],
            fmtps: vec![],
            ice_ufrag: None,
            ice_pwd: None,
            ice_candidates: vec![],
            ice_end_of_candidates: false,
            attributes: vec![],
        };

        if let Some(fmtp) = codec_info.fmtp {
            answer_media.fmtps.push(fmtp);
        }

        // Try to find dtmf event in the media
        for rtpmap in media.rtpmaps {
            if rtpmap.encoding.eq_ignore_ascii_case("telephone-event") && rtpmap.clock_rate == 8000
            {
                let fmtp = media
                    .fmtps
                    .iter()
                    .position(|fmtp| fmtp.format == rtpmap.payload);

                telephone_event_pt = Some(rtpmap.payload);
                answer_media.desc.fmts.push(rtpmap.payload);
                answer_media.rtpmaps.push(rtpmap);

                if let Some(fmtp) = fmtp {
                    answer_media.fmtps.push(media.fmtps.remove(fmtp));
                }
            }
        }

        media_scopes.push(answer_media);
        session_media.push(SessionMedia {
            media_type,
            direction,

            rtp_udpsrc: socket_pair.rtp_udpsrc,
            rtp_udpsink: socket_pair.rtp_udpsink,
            rtcp_udpsrc: socket_pair.rtcp_udpsrc,
            rtcp_udpsink: socket_pair.rtcp_udpsink,

            remote_rtp_addr: peer_rtp_addr,
            remote_rtcp_addr: peer_rtcp_addr,
            gst_elements: codec_info.gst_elements,
        });
    }

    let telephone_event_pt = telephone_event_pt.context("No telephone-event in offer")?;

    let local_sdp = Message {
        name: BytesStr::from_static("opentalk-obelisk"),
        origin: Origin {
            username: BytesStr::from_static("-"),
            session_id: id.to_string().into(),
            session_version: version.to_string().into(),
            address: pub_ip.into(),
        },
        time: Time { start: 0, stop: 0 },
        direction: Direction::SendRecv,
        connection: None,
        bandwidth: vec![],
        ice_options: ice::Options::default(),
        ice_lite: false,
        ice_ufrag: None,
        ice_pwd: None,
        attributes: vec![],
        media_scopes,
    };

    Ok(SessionInfo {
        local_sdp,

        remote_id,
        remote_version,

        telephone_event_pt,
        media: session_media,
    })
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
