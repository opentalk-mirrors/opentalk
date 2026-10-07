// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::room::RoomClient,
};
use livekit_protocol::TrackSource;
use opentalk_roomserver_livekit_proxy::{
    ShutdownSender, build_livekit_rtc_url, connect_to_livekit, proxy_websocket,
};
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, NoSettings, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    LIVEKIT_SUBROOM_AUDIO_ROOM_DELIMITER,
    connection_id::ConnectionId,
    livekit_proxy::{
        LiveKitProxyRequest, LiveKitProxyTarget, PreparedSocket, websocket::LiveKitSocket,
    },
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_livekit::LiveKitSettings;
use opentalk_roomserver_types_subroom_audio::{
    SUBROOM_AUDIO_MODULE_ID, WhisperId,
    command::{ParticipantTargets, SubroomAudioCommand},
    event::{SubroomAudioError, SubroomAudioEvent, WhisperParticipantInfo},
    internal::SubroomAudioInternal,
    state::{WhisperGroup, WhisperState},
};
use opentalk_types_common::{modules::ModuleId, rooms::RoomId};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::oneshot;

use crate::loopback::SubroomAudioLoopback;

pub mod loopback;

const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(32);

pub struct SubroomAudioModule {
    settings: Arc<LiveKitSettings>,
    livekit_client: Arc<RoomClient>,
    whisper_rooms: HashMap<WhisperId, WhisperGroup>,
    proxy_shutdown: HashMap<(ParticipantId, ConnectionId), ShutdownSender>,
}

impl SignalingModuleDescription for SubroomAudioModule {
    const MODULE_ID: ModuleId = SUBROOM_AUDIO_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles sub-room audio, allowing participants to talk to each other in a separate audio group.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for SubroomAudioModule {
    const NAMESPACE: ModuleId = SUBROOM_AUDIO_MODULE_ID;

    type Incoming = SubroomAudioCommand;

    type Outgoing = SubroomAudioEvent;

    type Internal = SubroomAudioInternal;

    type Loopback = Result<SubroomAudioLoopback, SubroomAudioError>;

    type JoinInfo = ();

    type PeerJoinInfo = ();

    type Error = SubroomAudioError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let livekit_settings = (init_data
            .room_parameters
            .module_settings
            .get::<LiveKitSettings>()
            .ok()?)?;

        let livekit_client = RoomClient::with_api_key(
            livekit_settings.service_url.as_str(),
            &livekit_settings.api_key,
            &livekit_settings.api_secret,
        );

        Some(Self {
            settings: Arc::new(livekit_settings.clone()),

            livekit_client: Arc::new(livekit_client),

            whisper_rooms: HashMap::new(),
            proxy_shutdown: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        Ok(ModuleJoinData::default())
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.cancel_proxies_for_connection(participant_id, connection_id);

        let result = self.leave_all_whisper_groups(ctx, participant_id);

        if result.is_err() {
            tracing::debug!(
                "Failed to remove participant {participant_id} from all whisper groups: {:?}",
                result.err()
            );
        }

        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        match payload {
            SubroomAudioCommand::LeaveWhisperGroup { whisper_id } => {
                self.leave_whisper_group(ctx, sender, whisper_id)
            }
            SubroomAudioCommand::CreateWhisperGroup { participant_ids } => {
                self.create_whisper_group(ctx, (sender, connection_id), participant_ids)
            }
            SubroomAudioCommand::InviteToWhisperGroup(participant_targets) => {
                self.invite_to_whisper_group(ctx, sender, participant_targets)
            }
            SubroomAudioCommand::KickWhisperParticipants(participant_targets) => {
                self.kick_whisper_participants(ctx, sender, participant_targets)
            }
            SubroomAudioCommand::AcceptWhisperInvite { whisper_id } => {
                self.accept_whisper_invite(ctx, (sender, connection_id), whisper_id)
            }
            SubroomAudioCommand::DeclineWhisperInvite { whisper_id } => {
                self.decline_whisper_invite(ctx, sender, whisper_id)
            }
        }
    }

    fn on_internal_command(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        command: Self::Internal,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match command {
            SubroomAudioInternal::ConnectUpstreamSocket {
                websocket_request,
                return_channel,
            } => self.connect_upstream_socket(ctx, *websocket_request, return_channel),
            SubroomAudioInternal::ConnectDownstreamSocket {
                websocket_request,
                upstream_socket,
                downstream_socket,
                return_channel,
            } => self.connect_downstream_socket(
                ctx,
                *websocket_request,
                *upstream_socket,
                downstream_socket,
                return_channel,
            ),
        }

        Ok(())
    }

    fn on_closing(&mut self, _ctx: &mut ModuleContext<'_, Self>) -> Result<(), anyhow::Error> {
        self.proxy_shutdown.clear();

        Ok(())
    }
}

impl SubroomAudioModule {
    fn connect_upstream_socket(
        &self,
        ctx: &ModuleContext<'_, Self>,
        websocket_request: LiveKitProxyRequest,
        return_channel: oneshot::Sender<Option<PreparedSocket>>,
    ) {
        if !self.is_proxy_request_authorized(ctx, &websocket_request) {
            tracing::debug!("participant is unauthorized");
            let _ = return_channel
                .send(None)
                .inspect_err(|_| tracing::debug!("failed to send response"));
            return;
        }

        let raw_query = websocket_request.raw_query;
        let headers = websocket_request.headers;
        let Ok(livekit_rtc_url) = build_livekit_rtc_url(&self.settings.service_url) else {
            tracing::warn!(?self.settings.service_url, "invalid livekit service URL");
            let _ = return_channel
                .send(None)
                .inspect_err(|_| tracing::debug!("failed to send response"));
            return;
        };

        tokio::spawn(async move {
            match connect_to_livekit(livekit_rtc_url, raw_query, headers).await {
                Ok(upstream_socket) => {
                    let _ = return_channel
                        .send(Some(upstream_socket))
                        .inspect_err(|_| tracing::debug!("failed to send response"));
                }
                Err(err) => {
                    tracing::warn!("failed to connect to upstream livekit: {err:?}");
                    let _ = return_channel
                        .send(None)
                        .inspect_err(|_| tracing::debug!("failed to send response"));
                }
            }
        });
    }

    fn connect_downstream_socket(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        websocket_request: LiveKitProxyRequest,
        upstream_socket: PreparedSocket,
        downstream_socket: Box<dyn LiveKitSocket>,
        return_channel: oneshot::Sender<()>,
    ) {
        let participant_id = websocket_request.participant_id;
        let connection_id = websocket_request.connection_id;

        let (shutdown_tx, shutdown_rx) = ShutdownSender::new();
        let key = (participant_id, connection_id);
        self.proxy_shutdown.insert(key, shutdown_tx);

        if return_channel.send(()).is_err() {
            self.proxy_shutdown.remove(&key);
            return;
        }

        ctx.spawn(async move {
            if let Err(err) = proxy_websocket(upstream_socket, downstream_socket, shutdown_rx).await
            {
                tracing::warn!(
                    ?participant_id,
                    ?connection_id,
                    "subroom-audio livekit websocket proxy stopped with error: {err:?}"
                );
            }

            Ok(SubroomAudioLoopback::ProxySocketClosed)
        });
    }

    fn is_proxy_request_authorized(
        &self,
        ctx: &ModuleContext<'_, Self>,
        websocket_request: &LiveKitProxyRequest,
    ) -> bool {
        let LiveKitProxyTarget::SubroomAudio { whisper_id } = &websocket_request.proxy_target
        else {
            return false;
        };
        let whisper_id = WhisperId::from(*whisper_id);

        let Some(participant) = ctx.participant_state(websocket_request.participant_id) else {
            return false;
        };

        if participant.in_waiting_room
            || !participant
                .connections
                .contains_key(&websocket_request.connection_id)
        {
            return false;
        }

        let Some(whisper_group) = self.whisper_rooms.get(&whisper_id) else {
            return false;
        };

        matches!(
            whisper_group
                .participants
                .get(&websocket_request.participant_id),
            Some(WhisperState::Creator | WhisperState::Accepted)
        )
    }

    fn cancel_proxies_for_connection(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) {
        self.proxy_shutdown.remove(&(participant_id, connection_id));
    }

    fn get_whisper_group(
        &mut self,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<&mut WhisperGroup, SignalingModuleError<SubroomAudioError>> {
        let Some(whisper_group) = self.whisper_rooms.get_mut(&whisper_id) else {
            return Err(SubroomAudioError::InvalidWhisperId.into());
        };

        if !whisper_group
            .participants
            .iter()
            .any(|(participant_id, _)| participant_id == &sender)
        {
            return Err(SubroomAudioError::NotInvited.into());
        }

        Ok(whisper_group)
    }

    fn create_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: (ParticipantId, ConnectionId),
        participant_ids: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        if participant_ids.is_empty() {
            return Err(SubroomAudioError::EmptyParticipantList.into());
        }

        self.participant_targets_valid(ctx, &participant_ids)?;

        let whisper_id = WhisperId::generate();

        let token = self.create_room_and_access_token(ctx, sender, whisper_id)?;

        let mut whisper_participants = participant_ids
            .into_iter()
            .map(|participant_id| (participant_id, WhisperState::default()))
            .collect::<BTreeMap<_, _>>();

        whisper_participants.insert(sender.0, WhisperState::Creator);

        let whisper_group = WhisperGroup {
            whisper_id,
            participants: whisper_participants.clone(),
        };

        self.whisper_rooms.insert(whisper_id, whisper_group.clone());

        ctx.send_ws_message(
            whisper_participants
                .keys()
                .filter(|&&p| p != sender.0)
                .copied(),
            SubroomAudioEvent::WhisperInvite {
                issuer: sender.0,
                group: whisper_group.clone().into(),
            },
        )?;

        ctx.send_ws_message(
            [sender.0],
            SubroomAudioEvent::WhisperGroupCreated {
                token,
                group: whisper_group.into(),
            },
        )?;

        Ok(())
    }

    fn accept_whisper_invite(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: (ParticipantId, ConnectionId),
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        let token = self.create_access_token(ctx.room_id, sender, whisper_id)?;
        let whisper_group = self.get_whisper_group(sender.0, whisper_id)?;

        if whisper_group.has_accepted(&sender.0) {
            return Err(SubroomAudioError::AlreadyAccepted.into());
        }

        if let Some(state) = whisper_group.participants.get_mut(&sender.0) {
            *state = WhisperState::Accepted;
        } else {
            return Err(SubroomAudioError::NotInvited.into());
        }

        ctx.send_ws_message(
            [sender.0],
            SubroomAudioEvent::WhisperToken { whisper_id, token },
        )?;

        ctx.send_ws_message(
            whisper_group
                .participants
                .keys()
                .filter(|&&p| p != sender.0)
                .copied(),
            SubroomAudioEvent::WhisperInviteAccepted {
                whisper_id,
                participant_id: sender.0,
            },
        )?;

        Ok(())
    }

    fn decline_whisper_invite(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        let whisper_group = self.get_whisper_group(sender, whisper_id)?;

        whisper_group.participants.remove(&sender);

        ctx.send_ws_message(
            whisper_group
                .participants
                .keys()
                .filter(|&&p| p != sender)
                .copied(),
            SubroomAudioEvent::WhisperInviteDeclined(WhisperParticipantInfo {
                whisper_id,
                participant_id: sender,
            }),
        )?;

        Ok(())
    }

    fn leave_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        whisper_id: WhisperId,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        let livekit_client = Arc::clone(&self.livekit_client);
        let whisper_group = self.get_whisper_group(sender, whisper_id)?;
        let has_token = whisper_group.has_accepted(&sender);
        let livekit_room_id = build_livekit_whisper_room_id(ctx.room_id, whisper_id);

        if has_token {
            ctx.spawn(loopback::remove_participant(
                livekit_client,
                livekit_room_id.clone(),
                sender.to_string(),
            ));
        }

        whisper_group.participants.remove(&sender);

        if whisper_group.participants.is_empty() {
            ctx.spawn(loopback::destroy_room(
                Arc::clone(&self.livekit_client),
                livekit_room_id,
            ));

            self.whisper_rooms.remove(&whisper_id);
        } else {
            ctx.send_ws_message(
                whisper_group
                    .participants
                    .keys()
                    .filter(|&&p| p != sender)
                    .copied(),
                SubroomAudioEvent::LeftWhisperGroup(WhisperParticipantInfo {
                    whisper_id,
                    participant_id: sender,
                }),
            )?;
        }

        Ok(())
    }

    fn leave_all_whisper_groups(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        for whisper_id in self.whisper_rooms.keys().copied().collect::<Vec<_>>() {
            let Ok(whisper_group) = self.get_whisper_group(sender, whisper_id) else {
                continue;
            };

            if whisper_group
                .participants
                .iter()
                .any(|(partipand_id, _)| partipand_id == &sender)
            {
                self.leave_whisper_group(ctx, sender, whisper_id)?;
            }
        }

        Ok(())
    }

    fn invite_to_whisper_group(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        participant_targets: ParticipantTargets,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        self.participant_targets_valid(ctx, &participant_targets.participant_ids)?;
        let whisper_group = self.get_whisper_group(sender, participant_targets.whisper_id)?;

        if !is_group_creator(&sender, whisper_group) {
            return Err(SubroomAudioError::InsufficientPermissions.into());
        }

        let new_participants = participant_targets
            .participant_ids
            .iter()
            .filter(|&&participant_id| {
                let invitable_participant =
                    !whisper_group.contains(&participant_id) && participant_id != sender;

                if !invitable_participant {
                    tracing::debug!("Skip to invite participant {participant_id}");
                }

                invitable_participant
            })
            .map(|participant_id| (*participant_id, WhisperState::Invited))
            .collect::<BTreeMap<_, _>>();

        if new_participants.is_empty() {
            return Ok(());
        }

        let original_participant_ids: Vec<_> = whisper_group
            .participants
            .keys()
            .copied()
            .filter(|&p| p != sender)
            .collect();
        let new_participant_ids: Vec<_> = new_participants.keys().copied().collect();

        for participant_id in new_participant_ids.clone() {
            whisper_group
                .participants
                .insert(participant_id, WhisperState::Invited);
        }

        ctx.send_ws_message(
            new_participant_ids.clone(),
            SubroomAudioEvent::WhisperInvite {
                issuer: sender,
                group: whisper_group.clone().into(),
            },
        )?;

        // This is only send to the original whisper group participants
        ctx.send_ws_message(
            original_participant_ids,
            SubroomAudioEvent::ParticipantsInvited {
                whisper_id: participant_targets.whisper_id,
                participant_ids: new_participant_ids,
            },
        )?;

        Ok(())
    }

    fn kick_whisper_participants(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        participant_targets: ParticipantTargets,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        self.participant_targets_valid(ctx, &participant_targets.participant_ids)?;

        let whisper_group = self.get_whisper_group(sender, participant_targets.whisper_id)?;
        let participants_to_kick: BTreeSet<_> = participant_targets
            .participant_ids
            .into_iter()
            .filter(|&p| p != sender)
            .collect();

        if !is_group_creator(&sender, whisper_group) {
            return Err(SubroomAudioError::InsufficientPermissions.into());
        }

        whisper_group
            .participants
            .retain(|participant_id, _| !participants_to_kick.contains(participant_id));

        ctx.send_ws_message(
            participants_to_kick,
            SubroomAudioEvent::Kicked {
                whisper_id: participant_targets.whisper_id,
            },
        )?;

        Ok(())
    }

    fn participant_targets_valid(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        participants: &BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<SubroomAudioError>> {
        if participants.is_empty() {
            return Err(SubroomAudioError::EmptyParticipantList.into());
        }

        let room_participants = &ctx.participants.connected().ids().collect();
        let invalid_participants: Vec<_> = participants
            .difference(room_participants)
            .copied()
            .collect();

        if !invalid_participants.is_empty() {
            return Err(SubroomAudioError::InvalidParticipantTargets {
                participant_ids: invalid_participants,
            }
            .into());
        }

        Ok(())
    }

    fn create_room_and_access_token(
        &self,
        ctx: &ModuleContext<'_, Self>,
        sender: (ParticipantId, ConnectionId),
        whisper_id: WhisperId,
    ) -> Result<String, SignalingModuleError<SubroomAudioError>> {
        let livekit_room_id = build_livekit_whisper_room_id(ctx.room_id, whisper_id);

        ctx.spawn(loopback::create_room(
            Arc::clone(&self.livekit_client),
            livekit_room_id,
        ));

        let token = self.create_access_token(ctx.room_id, sender, whisper_id)?;

        Ok(token)
    }

    fn create_access_token(
        &self,
        room_id: RoomId,
        sender: (ParticipantId, ConnectionId),
        whisper_id: WhisperId,
    ) -> Result<String, SignalingModuleError<SubroomAudioError>> {
        let identity = build_livekit_participant_id(sender.0, sender.1);

        let access_token =
            AccessToken::with_api_key(&self.settings.api_key, &self.settings.api_secret)
                .with_name(&identity)
                .with_identity(&identity)
                .with_grants(VideoGrants {
                    room_create: false,
                    room_list: false,
                    room_record: false,
                    room_admin: false,
                    room_join: true,
                    room: build_livekit_whisper_room_id(room_id, whisper_id),
                    destination_room: String::new(),
                    can_publish: Some(true),
                    can_subscribe: Some(true),
                    can_publish_data: Some(false),
                    can_publish_sources: vec![TrackSource::Microphone.as_str_name().to_lowercase()],
                    can_update_own_metadata: Some(false),
                    ingress_admin: false,
                    hidden: false,
                    recorder: false,
                    agent: false,
                })
                .with_ttl(ACCESS_TOKEN_TTL)
                .to_jwt()
                .context("Failed to create livekit access-token")?;

        Ok(access_token)
    }
}

fn build_livekit_whisper_room_id(room_id: RoomId, whisper_id: WhisperId) -> String {
    format!("{room_id}{LIVEKIT_SUBROOM_AUDIO_ROOM_DELIMITER}{whisper_id}")
}

fn is_group_creator(sender: &ParticipantId, whisper_group: &WhisperGroup) -> bool {
    matches!(
        whisper_group.participants.get(sender),
        Some(WhisperState::Creator)
    )
}

fn build_livekit_participant_id(participant: ParticipantId, connection: ConnectionId) -> String {
    format!("{participant}:{connection}")
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::SubroomAudioModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&SubroomAudioModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[subroom_audio]");
    }
}
