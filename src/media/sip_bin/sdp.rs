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
use std::time::{SystemTime, UNIX_EPOCH};

/// The local info used to negotiate the SDP session
pub struct LocalInfo {
    pub offer: Message,

    pub rtp_addr: SocketAddr,
    pub rtcp_addr: SocketAddr,
}

/// The SDP session info used to construct the sip_bin
pub struct SessionInfo {
    pub answer: Message,

    pub telephone_event_pt: u32,
    pub rtp_addr: SocketAddr,
    pub rtcp_addr: SocketAddr,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid offer")]
    InvalidOffer,
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Negotiate a audio PCMA & DTMF (telephone-event) session
pub fn respond(mut local_info: LocalInfo) -> Result<SessionInfo, Error> {
    if local_info.offer.media_scopes.len() != 1 {
        return Err(Error::InvalidOffer);
    }

    let mut media = local_info.offer.media_scopes.remove(0);

    if media.desc.media_type != MediaType::Audio {
        return Err(Error::InvalidOffer);
    }

    let peer_rtp_port = media.desc.port;
    let peer_rtcp_port = media
        .rtcp_attr
        .map(|rtcp| rtcp.port)
        .unwrap_or_else(|| peer_rtp_port + 1);

    let peer_rtp_addr = tagged_to_socket_addr(&local_info.offer.origin.address, peer_rtp_port)?;
    let peer_rtcp_addr = SocketAddr::new(peer_rtp_addr.ip(), peer_rtcp_port);

    let mut answer_media = MediaScope {
        desc: MediaDescription {
            media_type: MediaType::Audio,
            port: local_info.rtp_addr.port(),
            ports_num: None,
            proto: TransportProtocol::RtpAvp,
            fmts: vec![8],
        },
        direction: Direction::SendRecv,
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
        rtpmaps: vec![],
        fmtps: vec![],
        ice_ufrag: None,
        ice_pwd: None,
        ice_candidates: vec![],
        ice_end_of_candidates: false,
        attributes: vec![],
    };

    let mut supports_pcma = false;
    let mut telephone_event_pt = None;

    for rtpmap in media.rtpmaps {
        if rtpmap.payload == 8 && rtpmap.encoding == "PCMA" && rtpmap.clock_rate == 8000 {
            answer_media.rtpmaps.push(rtpmap);
            supports_pcma = true;
        } else if rtpmap.encoding == "telephone-event" && rtpmap.clock_rate == 8000 {
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

    if !supports_pcma || telephone_event_pt.is_none() {
        return Err(Error::InvalidOffer);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let answer = Message {
        name: BytesStr::from_static("ezk"),
        origin: Origin {
            username: BytesStr::from_static("-"),
            session_id: now.to_string().into(),
            session_version: (now + 1).to_string().into(),
            address: local_info.rtp_addr.ip().into(),
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
        media_scopes: vec![answer_media],
    };

    Ok(SessionInfo {
        answer,
        // unwrap safe: checked earlier
        telephone_event_pt: telephone_event_pt.unwrap(),
        rtp_addr: peer_rtp_addr,
        rtcp_addr: peer_rtcp_addr,
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
