// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::codec::{SdpRecvCodec, SdpSendCodec, offer_all_audio_codecs, offer_all_video_codecs};
use crate::media::codec::MediaType;
use crate::media::{AddressFamily, PortPool, SocketPair};
use crate::settings::Settings;
use crate::sip::codec::{available_video_codec, choose_audio_codec, choose_video_codec};
use anyhow::{Context, Result, bail};
use base64::prelude::*;
use bytes::Bytes;
use bytesstr::BytesStr;
use core::fmt;
use rand::RngCore;
use sdp_types::{
    Connection, Direction, Fmtp, IceOptions, Media, MediaDescription, MediaType as SdpMediaType,
    Origin, Rtcp, RtpMap, SessionDescription, SrtpCrypto, SrtpKeyingMaterial, SrtpSuite,
    TaggedAddress, Time, TransportProtocol,
};
use std::{
    io,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    num::Wrapping,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MediaSessionId(u32);

impl fmt::Display for MediaSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub struct SessionState {
    settings: Arc<Settings>,
    pub_ip: IpAddr,

    sdp_id: u64,
    sdp_version: u64,

    next_media_session_id: Wrapping<u32>,

    media_sessions: Vec<MediaSession>,
}

pub struct MediaSession {
    // Internal id to avoid doubly matching a media session against a new offer
    id: MediaSessionId,

    // Data to match against media streams in incoming offers
    media_type: MediaType,
    pub state: MediaSessionState,

    local_rtp_addr: SocketAddr,
    local_rtcp_addr: SocketAddr,
    direction: Direction,

    // media ession's TELEPHONE-EVENT
    pub dtmf_events: Option<DtmfEvents>,

    pub socket_pair: SocketPair,
}

pub enum MediaSessionState {
    Offered(Offered),
    Established(Established),
}

impl MediaSessionState {
    fn is_offered(&self) -> bool {
        matches!(self, Self::Offered(..))
    }

    fn is_established(&self) -> bool {
        matches!(self, Self::Established(..))
    }
}

pub struct Offered {
    pub codec: Vec<SdpRecvCodec>,
    pub srtp: Vec<SrtpInfo>,
}

pub struct Established {
    pub remote_rtp_addr: SocketAddr,
    pub remote_rtcp_addr: SocketAddr,

    pub send_codec: SdpSendCodec,
    pub recv_codec: SdpRecvCodec,

    pub srtp: Option<SrtpInfo>,
}

#[derive(Clone)]
pub struct SrtpInfo {
    pub tag: u32,
    pub suite: SrtpSuite,

    pub send_key: Vec<u8>,
    pub recv_key: Vec<u8>,
}

pub struct DtmfEvents {
    pub pt: u8,
    fmtp: Option<Fmtp>,
}

pub enum MediaChange {
    AddReceiver(MediaSessionId),
    RemoveReceiver(MediaSessionId),

    AddSender(MediaSessionId),
    RemoveSender(MediaSessionId),
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

            sdp_id: now,
            sdp_version: now + 1,

            next_media_session_id: Wrapping::default(),
            media_sessions: vec![],
        }
    }

    fn make_media_session_id(&mut self) -> MediaSessionId {
        let id = self.next_media_session_id;
        self.next_media_session_id += 1;
        MediaSessionId(id.0)
    }

    pub fn media_by_id(&self, id: MediaSessionId) -> &MediaSession {
        self.media_sessions
            .iter()
            .find(|m| m.id == id)
            .expect("Invalid media id")
    }

    pub async fn setup_offer(&mut self) -> Result<()> {
        let audio_session_exist = self
            .media_sessions
            .iter()
            .any(|m| m.media_type == MediaType::Audio);
        let video_session_exists = self
            .media_sessions
            .iter()
            .any(|m| m.media_type == MediaType::Video);

        if !audio_session_exist {
            let audio_rtp_pair = PortPool::instance()
                .create_rtp_socket_pair(AddressFamily::of(self.pub_ip))
                .await
                .context("Failed to create RTP sockets")?
                .context("No more available RTP ports")?;

            let id = self.make_media_session_id();
            self.media_sessions.push(MediaSession {
                id,
                media_type: MediaType::Audio,
                state: MediaSessionState::Offered(Offered {
                    codec: offer_all_audio_codecs(),
                    srtp: if self.settings.sip.offer_srtp {
                        offer_srtp()
                    } else {
                        vec![]
                    },
                }),
                local_rtp_addr: (self.pub_ip, audio_rtp_pair.public_rtp_port).into(),
                local_rtcp_addr: (self.pub_ip, audio_rtp_pair.public_rtcp_port).into(),
                direction: Direction::SendRecv,
                dtmf_events: Some(DtmfEvents {
                    pt: 101,
                    fmtp: None,
                }),
                socket_pair: audio_rtp_pair,
            });
        }

        if !video_session_exists {
            let video_rtp_pair: SocketPair = PortPool::instance()
                .create_rtp_socket_pair(AddressFamily::of(self.pub_ip))
                .await
                .context("Failed to create RTP sockets")?
                .context("No more available RTP ports")?;

            let id = self.make_media_session_id();
            self.media_sessions.push(MediaSession {
                id,
                media_type: MediaType::Video,
                state: MediaSessionState::Offered(Offered {
                    codec: offer_all_video_codecs(),
                    srtp: if self.settings.sip.offer_srtp {
                        offer_srtp()
                    } else {
                        vec![]
                    },
                }),
                local_rtp_addr: (self.pub_ip, video_rtp_pair.public_rtp_port).into(),
                local_rtcp_addr: (self.pub_ip, video_rtp_pair.public_rtcp_port).into(),
                direction: Direction::SendRecv,
                dtmf_events: None,
                socket_pair: video_rtp_pair,
            });
        }

        Ok(())
    }

    #[expect(clippy::similar_names)]
    pub fn receive_answer(&mut self, answer: Bytes) -> Result<Vec<MediaChange>> {
        let answer =
            BytesStr::from_utf8_bytes(answer).context("SDP contains invalid UTF-8 characters")?;
        let answer = SessionDescription::parse(&answer)
            .with_context(|| format!("Failed to parse SDP Answer:\n{answer}"))?;

        let mut changes = vec![];

        for answer_media in answer.media_descriptions {
            // Ignore inactive and invalid media
            if answer_media.media.port == 0
                || answer_media.media.fmts.is_empty()
                || matches!(answer_media.direction, Direction::Inactive)
            {
                continue;
            }

            let (remote_rtp_addr, remote_rtcp_addr) =
                remote_peer_addresses(answer.connection.as_ref(), &answer_media)?;

            let new_direction = flip_direction(
                answer_media.direction,
                remote_rtp_addr.ip().is_unspecified(),
            );

            // match against existing media session
            let media_session = self
                .media_sessions
                .iter_mut()
                .find(|existing_session| {
                    let same_media_type =
                        existing_session.media_type == answer_media.media.media_type;

                    same_media_type && existing_session.state.is_offered()
                })
                .context("Got media answer which wasn't offered")?;

            let MediaSessionState::Offered(offered) = &media_session.state else {
                unreachable!()
            };

            let srtp = find_matching_crypto_in_answer(&offered.srtp, &answer_media)?;

            if is_sending(new_direction) {
                changes.push(MediaChange::AddSender(media_session.id));
            }
            if is_receiving(new_direction) {
                changes.push(MediaChange::AddReceiver(media_session.id));
            }

            let send_codec = match answer_media.media.media_type {
                SdpMediaType::Audio => {
                    choose_audio_codec(&answer_media)
                        .context("No compatible audio codec in offer")?
                }
                SdpMediaType::Video => {
                    choose_video_codec(&self.settings, &answer_media).with_context(|| {
                        format!("No compatible video codec found (supported is: {}), continuing without video", available_video_codec())
                    })?.0
                }
                ty =>{
                    log::warn!("Skipping unexpected offered media {ty}");
                    continue;
                },
            };

            let recv_codec = offered
                .codec
                .iter()
                .find(|c| c.codec == send_codec.codec)
                .context("SDP response didn't contain matching codec in media description")?
                .clone();

            let dtmf_events = answer_media
                .rtpmap
                .iter()
                .find(|rtpmap| {
                    rtpmap.encoding.eq_ignore_ascii_case("telephone-event")
                        && rtpmap.clock_rate == recv_codec.codec.clock_rate()
                })
                .map(|rtpmap| DtmfEvents {
                    pt: rtpmap.payload,
                    fmtp: None,
                });

            media_session.state = MediaSessionState::Established(Established {
                remote_rtp_addr,
                remote_rtcp_addr,
                send_codec,
                recv_codec,
                srtp,
            });

            media_session.dtmf_events = dtmf_events;
        }

        // Remove any unmatched media sessions
        self.media_sessions.retain(|m| m.state.is_established());

        Ok(changes)
    }

    #[expect(clippy::similar_names)]
    pub async fn receive_offer(&mut self, offer: Bytes) -> Result<Vec<MediaChange>> {
        let offer =
            BytesStr::from_utf8_bytes(offer).context("SDP contains invalid UTF-8 characters")?;
        let offer = SessionDescription::parse(&offer)
            .with_context(|| format!("Failed to parse SDP Offer:\n{offer}"))?;

        // track which of the current media sessions have been already matched against the new ones
        let mut matched_media_sessions = vec![];

        let mut changes = vec![];

        for offer_media in offer.media_descriptions {
            // Ignore inactive and invalid media
            if offer_media.media.port == 0
                || offer_media.media.fmts.is_empty()
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
                        existing_session.media_type == offer_media.media.media_type;

                    let MediaSessionState::Established(existing_session) = &existing_session.state
                    else {
                        return false;
                    };

                    let addr_matches = existing_session.remote_rtp_addr == remote_rtp_addr
                        && existing_session.remote_rtcp_addr == remote_rtcp_addr;

                    let negotiated_codec_still_available = offer_media
                        .media
                        .fmts
                        .contains(&existing_session.send_codec.payload);

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

                    match self
                        .create_new_session(
                            &offer_media,
                            &mut matched_media_sessions,
                            remote_rtp_addr,
                            remote_rtcp_addr,
                            new_direction,
                        )
                        .await
                    {
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

    #[expect(clippy::similar_names)]
    async fn create_new_session(
        &mut self,
        offer_media: &MediaDescription,
        matched_media_sessions: &mut Vec<MediaSessionId>,
        remote_rtp_addr: SocketAddr,
        remote_rtcp_addr: SocketAddr,
        new_direction: Direction,
    ) -> Result<(MediaSessionId, Direction)> {
        let (media_type, send_codec, recv_codec) = match offer_media.media.media_type {
            SdpMediaType::Audio => {
                let send_codec = choose_audio_codec(offer_media)
                    .context("No compatible audio codec in offer")?;

                let recv_codec = SdpRecvCodec {
                    payload: send_codec.payload,
                    codec: send_codec.codec,
                    fmtp: None,
                };

                (MediaType::Audio, send_codec, recv_codec)
            }
            SdpMediaType::Video => {
                let Some((send_codec, recv_codec)) =
                    choose_video_codec(&self.settings, offer_media)
                else {
                    bail!(
                        "No compatible video codec found (supported is: {}), continuing without video",
                        available_video_codec()
                    );
                };
                (MediaType::Video, send_codec, recv_codec)
            }
            ty => {
                bail!("Skipping unexpected offered media {ty}");
            }
        };

        let srtp = match &offer_media.media.proto {
            TransportProtocol::RtpAvp => None,
            TransportProtocol::RtpSavp => setup_srtp(offer_media)?,
            p => bail!("Unsupported transport protocol: {p:?}"),
        };

        let id = self.make_media_session_id();

        matched_media_sessions.push(id);

        let socket_pair = PortPool::instance()
            .create_rtp_socket_pair(AddressFamily::of(remote_rtp_addr.ip()))
            .await
            .context("Failed to create RTP/RTCP sockets")?
            .context("No more RTP/RTCP ports are available")?;

        let local_rtp_addr = SocketAddr::new(self.pub_ip, socket_pair.public_rtp_port);
        let local_rtcp_addr = SocketAddr::new(self.pub_ip, socket_pair.public_rtcp_port);

        // Find DTMF caps
        let mut dtmf_events = None;

        for rtpmap in &offer_media.rtpmap {
            if rtpmap.encoding.eq_ignore_ascii_case("telephone-event") && rtpmap.clock_rate == 8000
            {
                let fmtp = offer_media
                    .fmtp
                    .iter()
                    .find(|fmtp| fmtp.format == rtpmap.payload);

                dtmf_events = Some(DtmfEvents {
                    pt: rtpmap.payload,
                    fmtp: fmtp.cloned(),
                });
            }
        }

        self.media_sessions.push(MediaSession {
            id,
            media_type,
            state: MediaSessionState::Established(Established {
                remote_rtp_addr,
                remote_rtcp_addr,
                recv_codec,
                send_codec,
                srtp,
            }),
            local_rtp_addr,
            local_rtcp_addr,
            direction: new_direction,
            socket_pair,
            dtmf_events,
        });

        Ok((id, Direction::Inactive))
    }

    pub fn build_sdp(&self) -> SessionDescription {
        let media_descriptions = self
            .media_sessions
            .iter()
            .map(determine_media_description)
            .collect();

        SessionDescription {
            origin: Origin {
                username: BytesStr::from_static("-"),
                session_id: self.sdp_id.to_string().into(),
                session_version: self.sdp_version.to_string().into(),
                address: self.pub_ip.into(),
            },
            name: BytesStr::from_static("opentalk-obelisk"),
            connection: None,
            bandwidth: vec![],
            time: Time { start: 0, stop: 0 },
            direction: Direction::SendRecv,
            group: vec![],
            extmap: vec![],
            extmap_allow_mixed: false,
            ice_lite: false,
            ice_options: IceOptions::default(),
            ice_ufrag: None,
            ice_pwd: None,
            setup: None,
            fingerprint: vec![],
            attributes: vec![],
            media_descriptions,
        }
    }
}

#[expect(clippy::similar_names)]
fn determine_media_description(media_session: &MediaSession) -> MediaDescription {
    let mut rtpmaps = vec![];
    let mut fmtps = vec![];
    let mut fmts = vec![];
    let mut crypto = vec![];

    let proto = match &media_session.state {
        MediaSessionState::Offered(offered) => {
            for codec in &offered.codec {
                rtpmaps.push(codec.make_rtpmap());
                fmts.push(codec.payload);

                if let Some(fmtp) = &codec.fmtp {
                    fmtps.push(Fmtp {
                        format: codec.payload,
                        params: fmtp.as_str().into(),
                    });
                }
            }

            for srtp in &offered.srtp {
                populate_crypto_attributes(srtp, &mut crypto);
            }

            if offered.srtp.is_empty() {
                TransportProtocol::RtpAvp
            } else {
                TransportProtocol::RtpSavp
            }
        }
        MediaSessionState::Established(established) => {
            rtpmaps.push(established.recv_codec.make_rtpmap());
            fmts.push(established.recv_codec.payload);

            if let Some(fmtp) = &established.recv_codec.fmtp {
                fmtps.push(Fmtp {
                    format: established.recv_codec.payload,
                    params: fmtp.as_str().into(),
                });
            }

            if let Some(srtp) = &established.srtp {
                populate_crypto_attributes(srtp, &mut crypto);

                TransportProtocol::RtpSavp
            } else {
                TransportProtocol::RtpAvp
            }
        }
    };

    if let Some(dtmf_events) = &media_session.dtmf_events {
        rtpmaps.push(RtpMap {
            payload: dtmf_events.pt,
            encoding: "telephone-event".into(),
            clock_rate: 8000,
            params: None,
        });

        // Copy over the fmtp field from offer for best compatibility
        if let Some(fmtp) = &dtmf_events.fmtp {
            fmtps.push(fmtp.clone());
        }

        fmts.push(dtmf_events.pt);
    }

    MediaDescription {
        media: Media {
            media_type: media_session.media_type.into(),
            port: media_session.local_rtp_addr.port(),
            ports_num: None,
            proto,
            fmts,
        },
        connection: Some(Connection {
            address: media_session.local_rtp_addr.ip().into(),
            ttl: None,
            num: None,
        }),
        bandwidth: vec![],
        direction: media_session.direction,
        rtcp: Some(Rtcp {
            port: media_session.local_rtcp_addr.port(),
            address: None,
        }),
        rtcp_mux: false,
        rtcp_rsize: false,
        mid: None,
        msid: None,
        rtpmap: rtpmaps,
        fmtp: fmtps,
        rtcp_fb: vec![],
        ice_ufrag: None,
        ice_pwd: None,
        ice_candidates: vec![],
        ice_end_of_candidates: false,
        crypto,
        extmap: vec![],
        extmap_allow_mixed: false,
        ssrc: vec![],
        setup: None,
        fingerprint: vec![],
        imageattr: vec![],
        attributes: vec![],
    }
}

fn populate_crypto_attributes(srtp: &SrtpInfo, crypto: &mut Vec<SrtpCrypto>) {
    let key_and_salt = BASE64_STANDARD.encode(&srtp.send_key);

    crypto.push(SrtpCrypto {
        tag: srtp.tag,
        suite: srtp.suite.clone(),
        keys: vec![SrtpKeyingMaterial {
            key_and_salt: key_and_salt.into(),
            lifetime: None,
            mki: None,
        }],
        params: vec![],
    });
}

fn offer_srtp() -> Vec<SrtpInfo> {
    use SrtpSuite::{
        AES_256_CM_HMAC_SHA1_32, AES_256_CM_HMAC_SHA1_80, AES_CM_128_HMAC_SHA1_32,
        AES_CM_128_HMAC_SHA1_80,
    };

    // key_len, suite
    let suites = [
        (46, AES_256_CM_HMAC_SHA1_80),
        (46, AES_256_CM_HMAC_SHA1_32),
        (30, AES_CM_128_HMAC_SHA1_80),
        (30, AES_CM_128_HMAC_SHA1_32),
    ];

    suites
        .into_iter()
        .enumerate()
        .map(|(tag, (key_len, suite))| {
            let mut send_key = vec![0u8; key_len];
            rand::rng().fill_bytes(&mut send_key);

            SrtpInfo {
                tag: u32::try_from(tag).expect("conversion to u32 not possible"),
                suite,
                send_key,
                recv_key: vec![],
            }
        })
        .collect()
}

fn find_matching_crypto_in_answer(
    offered: &[SrtpInfo],
    answer: &MediaDescription,
) -> Result<Option<SrtpInfo>> {
    for offered_srtp in offered {
        let Some(crypto) = answer
            .crypto
            .iter()
            .find(|attr| attr.tag == offered_srtp.tag && attr.suite == offered_srtp.suite)
        else {
            continue;
        };

        let recv_key = BASE64_STANDARD
            .decode(&crypto.keys[0].key_and_salt)
            .context("Failed to base64 decode key_and_salt of crypto")?;

        let mut srtp = offered_srtp.clone();
        srtp.recv_key = recv_key;

        return Ok(Some(srtp));
    }

    Ok(None)
}

fn setup_srtp(offer_media: &MediaDescription) -> Result<Option<SrtpInfo>> {
    use SrtpSuite::{
        AES_256_CM_HMAC_SHA1_32, AES_256_CM_HMAC_SHA1_80, AES_CM_128_HMAC_SHA1_32,
        AES_CM_128_HMAC_SHA1_80,
    };

    // Find best suite to use
    let choice1 = offer_media
        .crypto
        .iter()
        .find(|c| c.suite == AES_256_CM_HMAC_SHA1_80);
    let choice2 = offer_media
        .crypto
        .iter()
        .find(|c| c.suite == AES_256_CM_HMAC_SHA1_32);
    let choice3 = offer_media
        .crypto
        .iter()
        .find(|c| c.suite == AES_CM_128_HMAC_SHA1_80);
    let choice4 = offer_media
        .crypto
        .iter()
        .find(|c| c.suite == AES_CM_128_HMAC_SHA1_32);

    let crypto = choice1
        .or(choice2)
        .or(choice3)
        .or(choice4)
        .context("No crypto with compatible suite found")?;

    let recv_key = BASE64_STANDARD
        .decode(&crypto.keys[0].key_and_salt)
        .context("Failed to base64 decode key_and_salt of crypto")?;

    // Make our own key for sending data
    let mut send_key = vec![0; recv_key.len()];
    rand::rng().fill_bytes(&mut send_key);

    Ok(Some(SrtpInfo {
        tag: crypto.tag,
        suite: crypto.suite.clone(),
        send_key,
        recv_key,
    }))
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

#[expect(clippy::similar_names)]
fn remote_peer_addresses(
    connection: Option<&Connection>,
    media_desc: &MediaDescription,
) -> Result<(SocketAddr, SocketAddr)> {
    let peer_rtp_port = media_desc.media.port;
    let peer_rtcp_port = media_desc
        .rtcp
        .as_ref()
        .map_or_else(|| peer_rtp_port + 1, |rtcp| rtcp.port);
    let connection = media_desc
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
        TaggedAddress::IP4FQDN(name) | TaggedAddress::IP6FQDN(name) => format!("{name}:{port}")
            .to_socket_addrs()?
            .next()
            .expect("empty iterator"),
    };

    Ok(addr)
}
