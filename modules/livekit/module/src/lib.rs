// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use futures::{StreamExt as _, stream};
use livekit_api::services::room::RoomClient;
use livekit_protocol::TrackSource;
use opentalk_roomserver_livekit_proxy::{
    ShutdownSender, build_livekit_rtc_url, connect_to_livekit, proxy_websocket,
};
use opentalk_roomserver_signaling::{
    module_context::{ChannelDroppedError, ModuleContext},
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    breakout::BreakoutRoom,
    connection_id::ConnectionId,
    livekit_proxy::{
        LiveKitProxyRequest, LiveKitProxyTarget, PreparedSocket, websocket::LiveKitSocket,
    },
    room_kind::RoomKind,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_livekit::{
    LiveKitCommand, LiveKitError, LiveKitEvent, LiveKitInternal, LiveKitSettings, LiveKitState,
    MicrophoneRestrictionError, MicrophoneRestrictionState, ParticipantsMuted,
};
use opentalk_types_common::modules::{ModuleId, module_id};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::oneshot;
use url::Url;

use crate::{
    loopback::LiveKitLoopback,
    room::{LiveKitConnection, LiveKitSubroom},
};

pub mod loopback;
mod room;

const LIVEKIT_MODULE_ID: ModuleId = module_id!("livekit");

const PARALLEL_UPDATES: usize = 25;
const ACCESS_TOKEN_TTL: Duration = Duration::from_secs(32);
const LIVEKIT_MEDIA_SOURCES: [TrackSource; 4] = [
    TrackSource::Camera,
    TrackSource::Microphone,
    TrackSource::ScreenShare,
    TrackSource::ScreenShareAudio,
];

pub struct LiveKitModule {
    settings: Arc<LiveKitSettings>,

    /// The default screenshare permission. If the moderator didn't explicitly set a policy,
    /// this will be used to grant or deny screensharing privileges.
    ///
    /// `True` - all participants are allowed to screenshare
    /// `False` - only moderators are allowed to screenshare
    default_screenshare_permission: bool,

    /// LiveKit API client used to communicate with the LiveKit server
    livekit_client: Arc<RoomClient>,

    rooms: HashMap<RoomKind, LiveKitSubroom>,

    /// One connection can own several proxied sockets at once (the main media stream and any
    /// popout streams), so every shutdown sender is kept instead of replacing the previous one.
    proxy_shutdown: HashMap<(ParticipantId, ConnectionId), Vec<ShutdownSender>>,
}

impl SignalingModuleDescription for LiveKitModule {
    const MODULE_ID: ModuleId = LIVEKIT_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles Livekit media streams coordination and integration";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];
}

impl SignalingModule for LiveKitModule {
    const NAMESPACE: ModuleId = LIVEKIT_MODULE_ID;

    type Incoming = LiveKitCommand;

    type Outgoing = LiveKitEvent;

    type Internal = LiveKitInternal;

    type Loopback = Result<LiveKitLoopback, LiveKitError>;

    type JoinInfo = LiveKitState;

    type PeerJoinInfo = ();

    type Error = LiveKitError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let livekit_settings = (init_data
            .room_parameters
            .module_settings
            .get::<LiveKitSettings>()
            .ok()?)?;

        let default_screenshare_permission = init_data
            .settings
            .defaults
            .as_ref()
            .is_some_and(|d| !d.screen_share_requires_permission);

        let livekit_client = RoomClient::with_api_key(
            livekit_settings.service_url.as_str(),
            &livekit_settings.api_key,
            &livekit_settings.api_secret,
        );

        Some(Self {
            settings: Arc::new(livekit_settings.clone()),
            default_screenshare_permission,

            livekit_client: Arc::new(livekit_client),

            rooms: HashMap::new(),
            proxy_shutdown: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let room = self.rooms.entry(ctx.room).or_insert_with(|| {
            LiveKitSubroom::new(
                ctx,
                self.default_screenshare_permission,
                Arc::clone(&self.settings),
                Arc::clone(&self.livekit_client),
                ctx.room_id,
                ctx.room,
            )
        });

        room.join_info(ctx, participant_id, connection_id)
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.cancel_proxies_for_connection(participant_id, connection_id);

        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.start_revoke_participant_access(ctx, participant_id, connection_id);
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            LiveKitCommand::CreateNewAccessToken => {
                self.issue_access_token(ctx, sender, connection_id)
            }
            LiveKitCommand::GrantScreenSharePermission { participants } => {
                self.set_screenshare_permissions(ctx, sender, participants, true)
            }
            LiveKitCommand::RevokeScreenSharePermission { participants } => {
                self.set_screenshare_permissions(ctx, sender, participants, false)
            }
            LiveKitCommand::RequestPopoutStreamAccessToken => {
                self.issue_popout_stream_access_token(ctx, sender, connection_id)
            }
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event? {
            LiveKitLoopback::RoomCreated
            | LiveKitLoopback::RoomRemoved
            | LiveKitLoopback::ProxySocketClosed => Ok(()),
            LiveKitLoopback::NoteRevokedTokens {
                token_identities,
                participant_id,
                connection_id,
            } => self.note_revoked_tokens(ctx, token_identities, participant_id, connection_id),
            LiveKitLoopback::ScreenSharePermissionsUpdated {
                sender,
                participants,
                grant,
            } => Self::notify_screen_share_permission_update(ctx, sender, participants, grant),
        }
    }

    fn on_internal_command(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        command: Self::Internal,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match command {
            LiveKitInternal::Mute {
                sender,
                participants,
                return_channel,
            } => self.mute(ctx, sender, participants, return_channel),
            LiveKitInternal::UpdateMicrophoneRestrictions {
                sender,
                new_state,
                return_channel,
            } => self.update_microphone_restrictions(ctx, sender, new_state, return_channel)?,
            LiveKitInternal::ConnectUpstreamSocket {
                websocket_request,
                return_channel,
            } => self.connect_upstream_socket(ctx, *websocket_request, return_channel),
            LiveKitInternal::ConnectDownstreamSocket {
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
            LiveKitInternal::GetLivekitServiceUrl { return_channel } => {
                self.get_livekit_service_url(return_channel)
            }
        }
        Ok(())
    }

    fn on_closing(&mut self, ctx: &mut ModuleContext<'_, Self>) -> Result<(), anyhow::Error> {
        self.proxy_shutdown.clear();

        let rooms = self.rooms.drain().collect();
        Self::cleanup_rooms(ctx, rooms);

        Ok(())
    }

    fn on_breakout_start(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        _duration: Option<Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        for room in rooms {
            self.rooms
                .entry(RoomKind::Breakout(room.id))
                .or_insert_with(|| {
                    let room_kind = RoomKind::Breakout(room.id);
                    tracing::debug!("create room: {:?}", room_kind);
                    LiveKitSubroom::new(
                        ctx,
                        self.default_screenshare_permission,
                        self.settings.clone(),
                        Arc::clone(&self.livekit_client),
                        ctx.room_id,
                        room_kind,
                    )
                });
        }
        Ok(())
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        let connections = ctx.participants.connections();
        let connections = connections.get(&participant_id).ok_or_else(|| {
            anyhow::anyhow!("Unknown participant can't switch breakout rooms {participant_id}")
        })?;

        for connection_id in connections {
            self.cancel_proxies_for_connection(participant_id, *connection_id);
        }

        let Some(room) = self.rooms.get_mut(&old_room) else {
            return Err(anyhow::anyhow!(
                "Source room not found while switching breakout rooms ({old_room:?})"
            )
            .into());
        };
        for connection_id in connections {
            room.start_revoke_participant_access(ctx, participant_id, *connection_id);
        }

        let Some(room) = self.rooms.get_mut(&new_room) else {
            return Err(anyhow::anyhow!(
                "Destination room not found while switching breakout rooms ({new_room:?})"
            )
            .into());
        };
        let mut switch_success = BTreeMap::new();
        for &connection_id in connections {
            let join_info = room
                .join_info(ctx, participant_id, connection_id)?
                .join_success;
            switch_success.insert(connection_id, join_info);
        }
        Ok(ModuleSwitchData {
            switch_success,
            ..Default::default()
        })
    }

    fn on_breakout_closed(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let breakout_rooms = self
            .rooms
            .extract_if(|kind, _| *kind != RoomKind::Main)
            .collect();
        Self::cleanup_rooms(ctx, breakout_rooms);

        Ok(())
    }
}

impl LiveKitModule {
    fn connect_upstream_socket(
        &self,
        ctx: &ModuleContext<'_, Self>,
        websocket_request: LiveKitProxyRequest,
        return_channel: oneshot::Sender<Option<PreparedSocket>>,
    ) {
        if !Self::is_proxy_request_authorized(ctx, &websocket_request) {
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
                    let _ = return_channel.send(Some(upstream_socket));
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

        // Channel used to shutdown the proxy task. This happens when the participant leaves the
        // meeting
        let (shutdown_tx, shutdown_rx) = ShutdownSender::new();
        let shutdown_key = (participant_id, connection_id);

        if return_channel.send(()).is_err() {
            return;
        }

        self.proxy_shutdown
            .entry(shutdown_key)
            .or_default()
            .push(shutdown_tx);

        ctx.spawn(async move {
            if let Err(err) = proxy_websocket(upstream_socket, downstream_socket, shutdown_rx).await
            {
                tracing::warn!(
                    ?participant_id,
                    ?connection_id,
                    "livekit websocket proxy stopped with error: {err:?}"
                );
            }

            Ok(LiveKitLoopback::ProxySocketClosed)
        });
    }

    fn is_proxy_request_authorized(
        ctx: &ModuleContext<'_, Self>,
        websocket_request: &LiveKitProxyRequest,
    ) -> bool {
        let LiveKitProxyTarget::LiveKit { room_kind } = &websocket_request.proxy_target else {
            return false;
        };

        ctx.participant_state(websocket_request.participant_id)
            .is_some_and(|participant| {
                !participant.in_waiting_room
                    && participant.room == *room_kind
                    && participant
                        .connections
                        .contains_key(&websocket_request.connection_id)
            })
    }

    fn cancel_proxies_for_connection(
        &mut self,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) {
        self.proxy_shutdown.remove(&(participant_id, connection_id));
    }

    fn get_livekit_service_url(&self, return_channel: oneshot::Sender<Url>) {
        let _ = return_channel
            .send(self.settings.service_url.clone())
            .inspect_err(|_| tracing::debug!("failed to send response"));
    }
    /// creates a new access token and sends it to the participant
    fn issue_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        participant: ParticipantId,
        connection: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        tracing::debug!("Issue access token to {participant:?}");
        let credentials = room.create_new_access_token(ctx, participant, connection)?;
        ctx.send_ws_message([participant], LiveKitEvent::Credentials(credentials))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx, return_channel))]
    pub fn mute(
        &self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: Option<ParticipantId>,
        participants: BTreeSet<ParticipantId>,
        return_channel: oneshot::Sender<ParticipantsMuted>,
    ) {
        let connections = ctx
            .participants
            .all_unfiltered
            .iter()
            .filter(|(participant_id, _)| participants.contains(participant_id))
            .flat_map(|(participant_id, state)| {
                state.connections().map(|connection_id| {
                    LiveKitConnection::new(*participant_id, connection_id, ctx.room_id, state.room)
                })
            })
            .collect();

        tracing::debug!("spawn background task to mute participants");
        let livekit_client = Arc::clone(&self.livekit_client);
        ctx.spawn_optional(async move {
            let muted = loopback::mute_participants(livekit_client, sender, connections).await;
            if return_channel.send(muted).is_err() {
                tracing::error!("Channel dropped when muting participants");
            }
            None
        });
    }

    fn note_revoked_tokens(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        revoked_token_identities: BTreeSet<String>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.note_revoked_tokens(revoked_token_identities, participant_id, connection_id)
    }

    fn set_screenshare_permissions(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
        grant: bool,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.set_screenshare_permissions(ctx, sender, participants, grant)
    }

    fn notify_screen_share_permission_update(
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
        grant: bool,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        ctx.send_ws_message(
            [sender],
            LiveKitEvent::ScreenSharePermissionsUpdated {
                grant,
                participants,
            },
        )?;
        Ok(())
    }

    fn issue_popout_stream_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.issue_popout_stream_access_token(ctx, participant_id, connection_id)
    }

    fn update_microphone_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        new_state: MicrophoneRestrictionState,
        return_channel: oneshot::Sender<
            Result<MicrophoneRestrictionState, MicrophoneRestrictionError>,
        >,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let Some(room) = self.rooms.get_mut(&ctx.room) else {
            return Err(anyhow::anyhow!("Unknown room").into());
        };
        room.update_microphone_restrictions(ctx, sender, new_state, return_channel)
            .map_err(|ChannelDroppedError| {
                SignalingModuleError::Internal(anyhow!(
                    "Channel dropped when restricting microphone permissions"
                ))
            })
    }

    fn cleanup_rooms(ctx: &mut ModuleContext<'_, Self>, rooms: HashMap<RoomKind, LiveKitSubroom>) {
        ctx.spawn(async {
            stream::iter(rooms.into_iter().map(|(id, r)| async move {
                r.cleanup_room().await;
                tracing::debug!("LiveKitRoom removed: {id:?}");
            }))
            .buffer_unordered(PARALLEL_UPDATES)
            .collect::<Vec<()>>()
            .await;

            Ok(LiveKitLoopback::RoomRemoved)
        });
    }
}

fn build_livekit_participant_id(participant: ParticipantId, connection: ConnectionId) -> String {
    format!("{participant}:{connection}")
}
