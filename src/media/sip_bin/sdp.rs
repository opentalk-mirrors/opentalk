// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::codec::{choose_codec, GstCodecInfo};
use bytesstr::BytesStr;
use sdp_types::attributes::direction::Direction;
use sdp_types::attributes::ice;
use sdp_types::attributes::rtcp::RtcpAttr;
use sdp_types::connection::Connection;
use sdp_types::media::{MediaDescription, MediaType, TransportProtocol};
use sdp_types::msg::MediaScope;
use sdp_types::msg::Message;
use sdp_types::origin::Origin;
use sdp_types::time::Time;
use sdp_types::TaggedAddress;
use std::io;
use std::net::SocketAddr;
use std::net::{IpAddr, ToSocketAddrs};

/// The local info used to negotiate the SDP session
pub struct LocalInfo {
    pub offer: Message,

    pub id: u64,
    pub version: u64,

    pub rtp_addr: SocketAddr,
    pub rtcp_addr: SocketAddr,
}

/// The SDP session info used to construct the sip_bin
pub struct SessionInfo {
    pub local_sdp: Message,

    pub remote_id: u64,
    pub remote_version: u64,

    pub telephone_event_pt: u32,
    pub rtp_addr: SocketAddr,
    pub rtcp_addr: SocketAddr,

    pub gst_elements: GstCodecInfo,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid offer")]
    InvalidOffer,
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Negotiate a audio codec & DTMF (telephone-event) session
pub fn respond(local_info: LocalInfo) -> Result<SessionInfo, Error> {
    let mut offer = local_info.offer;

    if offer.media_scopes.len() != 1 {
        return Err(Error::InvalidOffer);
    }

    let mut media = offer.media_scopes.remove(0);

    if media.desc.media_type != MediaType::Audio {
        return Err(Error::InvalidOffer);
    }

    let codec_info = choose_codec(&media).ok_or(Error::InvalidOffer)?;

    let remote_id: u64 = offer
        .origin
        .session_id
        .parse()
        .map_err(|_| Error::InvalidOffer)?;

    let remote_version: u64 = offer
        .origin
        .session_version
        .parse()
        .map_err(|_| Error::InvalidOffer)?;

    let peer_rtp_port = media.desc.port;
    let peer_rtcp_port = media
        .rtcp_attr
        .map(|rtcp| rtcp.port)
        .unwrap_or_else(|| peer_rtp_port + 1);

    let connection = media
        .connection
        .or(offer.connection)
        .ok_or(Error::InvalidOffer)?;

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

    let mut answer_media = MediaScope {
        desc: MediaDescription {
            media_type: MediaType::Audio,
            port: local_info.rtp_addr.port(),
            ports_num: None,
            proto: TransportProtocol::RtpAvp,
            fmts: vec![codec_info.rtpmap.payload],
        },
        direction,
        connection: Some(Connection {
            address: local_info.rtp_addr.ip().into(),
            ttl: None,
            num: None,
        }),
        bandwidth: vec![],
        rtcp_attr: Some(RtcpAttr {
            port: local_info.rtcp_addr.port(),
            address: Some(local_info.rtcp_addr.ip().into()),
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
        media.fmtps.push(fmtp);
    }

    let mut telephone_event_pt = None;

    for rtpmap in media.rtpmaps {
        if rtpmap.encoding.eq_ignore_ascii_case("telephone-event") && rtpmap.clock_rate == 8000 {
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

    let telephone_event_pt = telephone_event_pt.ok_or(Error::InvalidOffer)?;

    let local_sdp = Message {
        name: BytesStr::from_static("opentalk-obelisk"),
        origin: Origin {
            username: BytesStr::from_static("-"),
            session_id: local_info.id.to_string().into(),
            session_version: local_info.version.to_string().into(),
            address: local_info.rtp_addr.ip().into(),
        },
        time: Time { start: 0, stop: 0 },
        direction,
        connection: None,
        bandwidth: vec![],
        ice_options: ice::Options::default(),
        ice_lite: false,
        ice_ufrag: None,
        ice_pwd: None,
        attributes: vec![],
        media_scopes: vec![answer_media],
    };

    Ok(SessionInfo {
        local_sdp,

        remote_id,
        remote_version,

        telephone_event_pt,
        rtp_addr: peer_rtp_addr,
        rtcp_addr: peer_rtcp_addr,

        gst_elements: codec_info.gst_elements,
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
