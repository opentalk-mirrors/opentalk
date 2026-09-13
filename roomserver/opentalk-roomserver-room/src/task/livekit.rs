// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_roomserver_signaling::{
    event_origin::{EventOrigin, ParticipantOrigin},
    internal_module_message::InterModuleMessage,
};
use opentalk_roomserver_types::{
    core::CORE_MODULE_ID,
    livekit_proxy::{
        LiveKitProxyRequest, LiveKitProxyTarget, PreparedSocket, websocket::LiveKitSocket,
    },
    room_kind::RoomKind,
    signaling::{module_error::FatalError, websocket::SignalingSocket},
};
use opentalk_roomserver_types_livekit::{LIVEKIT_MODULE_ID, LiveKitInternal};
use opentalk_roomserver_types_subroom_audio::{
    SUBROOM_AUDIO_MODULE_ID, internal::SubroomAudioInternal,
};
use opentalk_types_common::time::Timestamp;
use tokio::sync::oneshot::{self, Sender};
use url::Url;

use crate::{RoomTaskApiError, task::RoomTask};

impl<Socket: SignalingSocket> RoomTask<Socket> {
    pub(super) fn connect_upstream_socket(
        &mut self,
        websocket_request: LiveKitProxyRequest,
        return_channel: Sender<Result<PreparedSocket, RoomTaskApiError>>,
    ) -> Result<(), FatalError> {
        let (tx, rx) = oneshot::channel();

        let connection_id = websocket_request.connection_id;
        let participant_id = websocket_request.participant_id;

        match websocket_request.proxy_target {
            LiveKitProxyTarget::LiveKit { room_kind } => {
                self.handle_internal_command(
                    InterModuleMessage {
                        sender: CORE_MODULE_ID,
                        receiver: LIVEKIT_MODULE_ID,
                        command: Box::new(LiveKitInternal::ConnectUpstreamSocket {
                            websocket_request: Box::new(websocket_request),
                            return_channel: tx,
                        }),
                    },
                    room_kind,
                    EventOrigin::Participant(ParticipantOrigin {
                        id: participant_id,
                        connection_id,
                        transaction_id: None,
                    }),
                    Timestamp::now(),
                )?;
            }
            LiveKitProxyTarget::SubroomAudio { .. } => {
                let Some(participant) = self
                    .participants
                    .all_unfiltered
                    .get(&websocket_request.participant_id)
                else {
                    return_channel
                        .send(Err(RoomTaskApiError::Unauthorized))
                        .ok();
                    return Ok(());
                };

                self.handle_internal_command(
                    InterModuleMessage {
                        sender: CORE_MODULE_ID,
                        receiver: SUBROOM_AUDIO_MODULE_ID,
                        command: Box::new(SubroomAudioInternal::ConnectUpstreamSocket {
                            websocket_request: Box::new(websocket_request),
                            return_channel: tx,
                        }),
                    },
                    participant.room,
                    EventOrigin::Participant(ParticipantOrigin {
                        id: participant_id,
                        connection_id,
                        transaction_id: None,
                    }),
                    Timestamp::now(),
                )?;
            }
        }

        tokio::spawn(async {
            let _ = match rx.await {
                Ok(Some(socket)) => return_channel.send(Ok(socket)),
                Ok(None) => return_channel.send(Err(RoomTaskApiError::Unauthorized)),
                Err(_) => return_channel.send(Err(RoomTaskApiError::Internal)),
            }
            .inspect_err(|_| tracing::debug!("failed to send response"));
        });
        Ok(())
    }

    pub(super) fn connect_downstream_socket(
        &mut self,
        websocket_request: LiveKitProxyRequest,
        upstream_socket: PreparedSocket,
        downstream_socket: Box<dyn LiveKitSocket>,
        return_channel: Sender<Result<(), RoomTaskApiError>>,
    ) -> Result<(), FatalError> {
        let (tx, rx) = oneshot::channel();

        let connection_id = websocket_request.connection_id;
        let participant_id = websocket_request.participant_id;

        match websocket_request.proxy_target {
            LiveKitProxyTarget::LiveKit { room_kind } => {
                self.handle_internal_command(
                    InterModuleMessage {
                        sender: CORE_MODULE_ID,
                        receiver: LIVEKIT_MODULE_ID,
                        command: Box::new(LiveKitInternal::ConnectDownstreamSocket {
                            websocket_request: Box::new(websocket_request),
                            upstream_socket: Box::new(upstream_socket),
                            downstream_socket,
                            return_channel: tx,
                        }),
                    },
                    room_kind,
                    EventOrigin::Participant(ParticipantOrigin {
                        id: participant_id,
                        connection_id,
                        transaction_id: None,
                    }),
                    Timestamp::now(),
                )?;
            }
            LiveKitProxyTarget::SubroomAudio { .. } => {
                let Some(participant) = self
                    .participants
                    .all_unfiltered
                    .get(&websocket_request.participant_id)
                else {
                    let _ = return_channel.send(Err(RoomTaskApiError::Unauthorized));
                    return Ok(());
                };

                self.handle_internal_command(
                    InterModuleMessage {
                        sender: CORE_MODULE_ID,
                        receiver: SUBROOM_AUDIO_MODULE_ID,
                        command: Box::new(SubroomAudioInternal::ConnectDownstreamSocket {
                            websocket_request: Box::new(websocket_request),
                            upstream_socket: Box::new(upstream_socket),
                            downstream_socket,
                            return_channel: tx,
                        }),
                    },
                    participant.room,
                    EventOrigin::Participant(ParticipantOrigin {
                        id: participant_id,
                        connection_id,
                        transaction_id: None,
                    }),
                    Timestamp::now(),
                )?;
            }
        }

        tokio::spawn(async {
            let _ = match rx.await {
                Ok(response) => return_channel.send(Ok(response)),
                Err(_) => return_channel.send(Err(RoomTaskApiError::Internal)),
            }
            .inspect_err(|_| tracing::debug!("failed to send response"));
        });
        Ok(())
    }

    pub(super) fn get_livekit_service_url(
        &mut self,
        return_channel: Sender<Result<Url, RoomTaskApiError>>,
    ) -> Result<(), FatalError> {
        let (tx, rx) = oneshot::channel();

        self.handle_internal_command(
            InterModuleMessage {
                sender: CORE_MODULE_ID,
                receiver: LIVEKIT_MODULE_ID,
                command: Box::new(LiveKitInternal::GetLivekitServiceUrl { return_channel: tx }),
            },
            RoomKind::Main,
            EventOrigin::Internal,
            Timestamp::now(),
        )?;

        tokio::spawn(async {
            let _ = match rx.await {
                Ok(response) => return_channel.send(Ok(response)),
                Err(_) => return_channel.send(Err(RoomTaskApiError::Internal)),
            }
            .inspect_err(|_| tracing::debug!("failed to send response"));
        });
        Ok(())
    }
}
