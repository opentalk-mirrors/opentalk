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
    media: &mut MediaPipeline,
    session_state: &mut SessionState,
    changes: Vec<MediaChange>,
) -> Result<()> {
    for change in changes {
        match change {
            MediaChange::AddReceiver(id) => {
                let media_session = session_state.media_by_id(id);
                let MediaSessionState::Established(established) = &media_session.state else {
                    unreachable!("Cannot receive changes for not-established media sessions");
                };

                media
                    .add_receiver(
                        id,
                        established.srtp.as_ref(),
                        &established.recv_codec,
                        media_session
                            .dtmf_events
                            .as_ref()
                            .map(|e| (e.pt, media.events_tx.clone())),
                        media_session.socket_pair.rtp_socket.clone(),
                        media_session.socket_pair.rtcp_socket.clone(),
                        established.remote_rtp_addr,
                        established.remote_rtcp_addr,
                    )
                    .await
                    .with_context(|| format!("Failed to add receiver for session_id={id}"))?;
            }
            MediaChange::AddSender(id) => {
                let media_session = session_state.media_by_id(id);
                let MediaSessionState::Established(established) = &media_session.state else {
                    unreachable!("Cannot receive changes for not-established media sessions");
                };

                media
                    .add_sender(
                        id,
                        established.srtp.as_ref(),
                        &established.send_codec,
                        media_session.socket_pair.rtp_socket.clone(),
                        media_session.socket_pair.rtcp_socket.clone(),
                        established.remote_rtp_addr,
                        established.remote_rtcp_addr,
                    )
                    .await
                    .with_context(|| format!("Failed to add sender for session {id}"))?;
            }
            MediaChange::RemoveReceiver(id) => {
                media
                    .remove_receiver(id)
                    .await
                    .with_context(|| format!("Failed to remove receiver for session {id}"))?;
            }
            MediaChange::RemoveSender(id) => {
                media
                    .remove_sender(id)
                    .await
                    .with_context(|| format!("Failed to remove sender for session {id}"))?;
            }
        }
    }

    media.post_update_cleanup();

    Ok(())
}

pub async fn handle_reinvite(
    media: &mut MediaPipeline,
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

    let changes = media_session.receive_offer(body).await?;

    apply_changes(media, media_session, changes).await?;

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
