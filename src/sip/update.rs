// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::sdp::SessionInfo;
use crate::media::MediaPipeline;
use crate::settings::Settings;
use anyhow::{bail, Context, Result};
use sdp_types::attributes::direction::Direction;
use sip_types::{header::typed::ContentType, Code};
use sip_ua::invite::session::ReInviteReceived;
use std::net::IpAddr;

// TODO: rework this to only change local sockets when REALLY required
pub async fn handle_reinvite(
    settings: &Settings,
    handle: &mut MediaPipeline,
    event: ReInviteReceived<'_>,
    pub_ip: IpAddr,
) -> Result<()> {
    let request = &event.invite;

    let body_is_not_sdp = request
        .headers
        .get_named()
        .map(|content_type: ContentType| content_type.0 != "application/sdp")
        .unwrap_or_default();

    if body_is_not_sdp {
        let response = event
            .session
            .endpoint
            .create_response(request, Code::NOT_ACCEPTABLE, None);

        event.transaction.respond_failure(response).await?;

        bail!("failed to handle re-invite unknown content-type");
    }

    let body = request.body.clone();

    handle.tel_bin.remove_all_transceiver().await;

    let session_info =
        SessionInfo::create_from_offer(settings, body, pub_ip).context("Failed to handle offer")?;

    handle.tel_bin.rtp.set_pt_caps(
        session_info.telephone_event_pt,
        gst::Caps::builder("application/x-rtp")
            .field("media", "audio")
            .field("encoding-name", "TELEPHONE-EVENT")
            .field("clock-rate", 8000)
            .build(),
    );

    for (i, media) in session_info.media.iter().enumerate() {
        if matches!(media.direction, Direction::SendOnly | Direction::SendRecv) {
            handle
                .tel_bin
                .add_sender(
                    i as u32,
                    &media.gst_elements,
                    media.rtp_udpsink.clone(),
                    media.rtcp_udpsrc.clone(),
                    media.rtcp_udpsink.clone(),
                    media.remote_rtp_addr,
                    media.remote_rtcp_addr,
                )
                .await
                .with_context(|| format!("Failed to add sender for session {i}"))?;
        }

        if matches!(media.direction, Direction::RecvOnly | Direction::SendRecv) {
            handle
                .tel_bin
                .add_receiver(
                    i as u32,
                    media.gst_elements.codec,
                    media.gst_elements.payload,
                    media.rtp_udpsrc.clone(),
                    media.rtcp_udpsrc.clone(),
                    media.rtcp_udpsink.clone(),
                    media.remote_rtcp_addr,
                )
                .await
                .with_context(|| format!("Failed to add receiver for session {i}"))?;
        }
    }

    let sdp_response = session_info.local_sdp.to_string();

    let mut response = event
        .session
        .endpoint
        .create_response(request, Code::OK, None);

    response.msg.body = sdp_response.into();

    event.respond_success(response).await?;

    Ok(())
}
