// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{
    MediaSessionState, SdpSendCodecParams,
    sdp::{MediaChange, SessionState},
};
use crate::media::MediaPipeline;
use anyhow::{Context, Result, bail};
use sip_types::{CodeKind, Method, Name, StatusCode, header::typed::ContentType};
use sip_ua::invite::{
    create_ack,
    session::{ReInviteReceived, RefreshNeeded},
};

pub async fn apply_changes(
    media: &mut MediaPipeline,
    session_state: &mut SessionState,
    changes: Vec<MediaChange>,
) -> Result<()> {
    // Track the framerate at which the compositor should run, which will be the lowest of all video streams
    let mut target_fps = None;

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

                if let SdpSendCodecParams::H264 { fps, .. } = &established.send_codec.codec_params {
                    let current_target_fps = target_fps.unwrap_or(*fps);
                    target_fps = Some(current_target_fps.min(*fps));
                }

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

    if let Some(target_fps) = target_fps {
        media.target_fps = target_fps;
    }

    media.post_update_cleanup().await;

    Ok(())
}

pub async fn handle_reinvite(
    media: &mut MediaPipeline,
    media_session: &mut SessionState,
    event: ReInviteReceived<'_>,
) -> Result<()> {
    let request = &event.invite;

    let body_is_sdp = request
        .headers
        .get_named()
        .is_ok_and(|content_type: ContentType| content_type.0 == "application/sdp");

    if body_is_sdp {
        let body = request.body.clone();

        let changes = media_session.receive_offer(body).await?;

        apply_changes(media, media_session, changes).await?;

        let sdp_response = media_session.build_sdp().to_string();

        let mut response = event
            .session
            .dialog
            .create_response(request, StatusCode::OK, None)
            .context("Failed to create 200 OK response to INVITE")?;
        response
            .msg
            .headers
            .insert(Name::CONTENT_TYPE, "application/sdp");

        response.msg.body = sdp_response.into();

        event.respond_success(response).await?;
    } else {
        media_session.setup_offer().await?;

        let mut response = event
            .session
            .dialog
            .create_response(request, StatusCode::OK, None)
            .context("Failed to create 200 OK response to INVITE")?;
        response
            .msg
            .headers
            .insert(Name::CONTENT_TYPE, "application/sdp");
        response.msg.body = media_session.build_sdp().to_string().into();

        let ack = event.respond_success(response).await?;

        let ack_body_is_sdp = ack
            .headers
            .get_named()
            .is_ok_and(|content_type: ContentType| content_type.0 == "application/sdp");

        if !ack_body_is_sdp {
            bail!("Got ACK without SDP in body");
        }

        let changes = media_session.receive_answer(ack.body)?;

        apply_changes(media, media_session, changes).await?;
    }

    Ok(())
}

/// Related to the SIP `timer` extension, send an INVITE request inside an existing session to refresh it
pub async fn refresh_invite_session(
    media: &mut MediaPipeline,
    media_session: &mut SessionState,
    event: RefreshNeeded<'_>,
) -> Result<()> {
    let invite_session = event.session;

    invite_session.session_timer.reset();

    let mut invite = invite_session.dialog.create_request(Method::INVITE);
    invite_session.session_timer.populate_refresh(&mut invite);

    let mut target_tp_info = invite_session.dialog.target_tp_info.lock().await;
    let mut transaction = invite_session
        .endpoint
        .send_invite(invite, &mut target_tp_info)
        .await?;
    drop(target_tp_info);

    while let Some(response) = transaction.receive().await? {
        match response.line.code.kind() {
            CodeKind::Provisional => { /* ignore */ }
            CodeKind::Success => {
                let mut ack =
                    create_ack(&invite_session.dialog, response.base_headers.cseq.cseq).await?;

                // If the response to the INVITE contains SDP, handle it
                if let Ok(content_type) = response.headers.get_named::<ContentType>()
                    && content_type.0 == "application/sdp"
                {
                    let changes = media_session
                        .receive_offer(response.body)
                        .await
                        .context("Failed to handle SDP offer")?;

                    apply_changes(media, media_session, changes).await?;

                    let sdp_offer = media_session.build_sdp();

                    ack.msg.body = sdp_offer.to_string().into();
                    ack.msg
                        .headers
                        .insert(Name::CONTENT_TYPE, "application/sdp");
                }

                let endpoint = invite_session.endpoint.clone();

                endpoint
                    .send_outgoing_request(&mut ack)
                    .await
                    .map_err(sip_core::Error::from)?;

                // Resend ACK if the transaction receives additional responses
                tokio::spawn(async move {
                    while let Ok(Some(_)) = transaction.receive().await {
                        let _ = endpoint.send_outgoing_request(&mut ack).await;
                    }
                });

                return Ok(());
            }
            _ => bail!(
                "Got unexpected status code while refreshing INVITE session {:?}",
                response.line.code
            ),
        }
    }

    Ok(())
}
