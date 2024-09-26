// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{
    sdp::{MediaChange, SessionState},
    MediaSessionState,
};
use crate::media::MediaPipeline;
use anyhow::{bail, Context, Result};
use sip_types::{header::typed::ContentType, Code, Name};
use sip_ua::invite::session::ReInviteReceived;

pub async fn apply_changes(
    handle: &mut MediaPipeline,
    media_session: &mut SessionState,
    changes: Vec<MediaChange>,
) -> Result<()> {
    for dtmf_pt in media_session.all_dtmf_payloads() {
        handle.tel_bin.rtp.set_pt_caps(
            dtmf_pt,
            gst::Caps::builder("application/x-rtp")
                .field("media", "audio")
                .field("encoding-name", "TELEPHONE-EVENT")
                .field("clock-rate", 8000)
                .build(),
        );
    }

    for change in changes {
        match change {
            MediaChange::AddReceiver(id) => {
                let media = media_session.media_by_id(id);
                let MediaSessionState::Established(established) = &media.state else {
                    unreachable!("Cannot receive changes for not-established media sessions");
                };

                handle
                    .tel_bin
                    .add_receiver(
                        id,
                        established.srtp.as_ref(),
                        established.recv_codec.codec,
                        established.recv_codec.payload,
                        media.socket_pair.rtp_udpsrc.clone(),
                        media.socket_pair.rtcp_udpsrc.clone(),
                        media.socket_pair.rtcp_udpsink.clone(),
                        established.remote_rtcp_addr,
                    )
                    .await
                    .with_context(|| format!("Failed to add receiver for session {id}"))?;
            }
            MediaChange::AddSender(id) => {
                let media = media_session.media_by_id(id);
                let MediaSessionState::Established(established) = &media.state else {
                    unreachable!("Cannot receive changes for not-established media sessions");
                };

                handle
                    .tel_bin
                    .add_sender(
                        id,
                        established.srtp.as_ref(),
                        &established.send_codec,
                        media.socket_pair.rtp_udpsink.clone(),
                        media.socket_pair.rtcp_udpsrc.clone(),
                        media.socket_pair.rtcp_udpsink.clone(),
                        established.remote_rtp_addr,
                        established.remote_rtcp_addr,
                    )
                    .await
                    .with_context(|| format!("Failed to add sender for session {id}"))?;
            }
            MediaChange::RemoveReceiver(id) => {
                handle
                    .tel_bin
                    .remove_receiver(id)
                    .await
                    .with_context(|| format!("Failed to remove receiver for session {id}"))?;
            }
            MediaChange::RemoveSender(id) => {
                handle
                    .tel_bin
                    .remove_sender(id)
                    .await
                    .with_context(|| format!("Failed to remove sender for session {id}"))?;
            }
        }
    }

    Ok(())
}

pub async fn handle_reinvite(
    handle: &mut MediaPipeline,
    media_session: &mut SessionState,
    event: ReInviteReceived<'_>,
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

    let changes = media_session.receive_offer(body)?;

    // Check if video has been added but currently disabled in the media pipeline
    if !handle.with_video && media_session.has_video() {
        handle
            .enable_video()
            .await
            .context("Failed to enable video in media pipeline")?;
    }

    apply_changes(handle, media_session, changes).await?;

    let sdp_response = media_session.build_sdp().to_string();

    let mut response = event
        .session
        .dialog
        .create_response(request, Code::OK, None)
        .context("Failed to create 200 OK response to INVITE")?;
    response
        .msg
        .headers
        .insert(Name::CONTENT_TYPE, "application/sdp");

    response.msg.body = sdp_response.into();

    event.respond_success(response).await?;

    Ok(())
}
