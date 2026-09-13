// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    sync::Arc,
};

use anyhow::Context as _;
use futures::FutureExt as _;
use livekit_api::{
    access_token::{AccessToken, VideoGrants},
    services::{ServiceError, TwirpError, TwirpErrorCode, room::RoomClient},
};
use livekit_protocol::TrackSource;
use opentalk_roomserver_signaling::{
    module_context::{ChannelDroppedError, ModuleContext},
    signaling_module::{ModuleJoinData, PeerDataMap, SignalingModule},
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, room_kind::RoomKind, signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_livekit::{
    Credentials, LiveKitError, LiveKitEvent, LiveKitSettings, LiveKitState,
    MicrophoneRestrictionError, MicrophoneRestrictionErrorKind, MicrophoneRestrictionState,
};
use opentalk_types_common::rooms::RoomId;
use opentalk_types_signaling::ParticipantId;
use tokio::sync::{Mutex, oneshot};

use crate::{
    ACCESS_TOKEN_TTL, LIVEKIT_MEDIA_SOURCES, LiveKitModule, build_livekit_participant_id, loopback,
};

#[derive(Debug, Clone)]
pub struct LiveKitSubroom {
    /// The default screenshare permission. If the moderator didn't explicitly set a policy,
    /// this will be used to grant or deny screensharing privileges.
    ///
    /// `True` - all participants are allowed to screenshare
    /// `False` - only moderators are allowed to screenshare
    default_screenshare_permission: bool,

    settings: Arc<LiveKitSettings>,

    /// LiveKit API client used to communicate with the LiveKit server
    livekit_client: Arc<RoomClient>,
    /// The record of all issued tokens for each participant.
    token_identities: BTreeMap<(ParticipantId, ConnectionId), BTreeSet<String>>,

    /// Activating the microphone can be restricted by the moderator. A subset of users might still
    /// be allowed to unmute.
    microphone_restrictions: MicrophoneRestrictionState,

    /// There must only be one background task updating the microphone restrictions at any given
    /// time.
    ongoing_microphone_restrictions: Arc<Mutex<()>>,

    subroom_id: String,
}

impl LiveKitSubroom {
    pub fn new(
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        default_screenshare_permission: bool,
        settings: Arc<LiveKitSettings>,
        livekit_client: Arc<RoomClient>,
        room_id: RoomId,
        room_kind: RoomKind,
    ) -> Self {
        let subroom_id = build_subroom_id(room_id, room_kind);
        {
            let subroom_id = subroom_id.clone();
            let livekit_client = Arc::clone(&livekit_client);
            ctx.spawn(loopback::create_room(livekit_client, subroom_id));
        }

        Self {
            default_screenshare_permission,
            settings,
            livekit_client,
            token_identities: BTreeMap::new(),
            microphone_restrictions: MicrophoneRestrictionState::Disabled,
            ongoing_microphone_restrictions: Arc::new(Mutex::new(())),
            subroom_id,
        }
    }

    pub fn join_info(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<ModuleJoinData<LiveKitModule>, SignalingModuleError<LiveKitError>> {
        let credentials = self.create_new_access_token(ctx, participant_id, connection_id)?;
        Ok(ModuleJoinData {
            join_success: Some(LiveKitState {
                credentials,
                microphone_restriction_state: self.microphone_restrictions.clone(),
            }),
            peer_events: PeerDataMap::default(),
            peer_data: PeerDataMap::default(),
        })
    }

    pub fn identifier(&self) -> &str {
        &self.subroom_id
    }

    #[tracing::instrument(level = "debug", skip(self, ctx), fields(room = self.subroom_id))]
    pub fn create_new_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        participant: ParticipantId,
        connection: ConnectionId,
    ) -> Result<Credentials, SignalingModuleError<<LiveKitModule as SignalingModule>::Error>> {
        let mut available_sources = LIVEKIT_MEDIA_SOURCES.to_vec();
        if let MicrophoneRestrictionState::Enabled {
            unrestricted_participants,
        } = &self.microphone_restrictions
            && !ctx.is_moderator(participant)
            && !unrestricted_participants.contains(&participant)
        {
            available_sources.retain(|s| s != &TrackSource::Microphone);
        }

        if !self.default_screenshare_permission && !ctx.is_moderator(participant) {
            available_sources
                .retain(|s| s != &TrackSource::ScreenShare && s != &TrackSource::ScreenShareAudio);
        }

        let can_publish_sources = available_sources
            .into_iter()
            .map(|s| TrackSource::as_str_name(&s).to_lowercase())
            .collect();

        let identity = build_livekit_participant_id(participant, connection);

        let hidden = !ctx
            .participant_state(participant)
            .with_context(|| format!("Participant '{participant}' has no state"))?
            .is_visible();

        let token = AccessToken::with_api_key(&self.settings.api_key, &self.settings.api_secret)
            .with_name(&identity)
            .with_identity(&identity)
            .with_grants(VideoGrants {
                room_create: true,
                room_list: false,
                room_record: false,
                room_admin: false,
                room_join: true,
                room: self.identifier().to_string(),
                can_publish: true,
                can_subscribe: true,
                can_publish_data: false,
                can_publish_sources,
                can_update_own_metadata: false,
                ingress_admin: false,
                hidden,
                recorder: false,
                destination_room: String::new(),
            })
            .with_ttl(ACCESS_TOKEN_TTL)
            .to_jwt()
            .context("Failed to create LiveKit access-token")?;
        self.token_identities
            .entry((participant, connection))
            .or_default()
            .insert(identity);

        Ok(Credentials {
            room: self.identifier().to_string(),
            token,
            public_url: self.settings.public_url.clone(),
            service_url: None,
        })
    }

    #[tracing::instrument(level = "debug", skip(self), fields(room = self.subroom_id))]
    pub fn note_revoked_tokens(
        &mut self,
        revoked_token_identities: BTreeSet<String>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let entry = self.token_identities.entry((participant_id, connection_id));

        if let Entry::Occupied(mut occupied) = entry {
            occupied
                .get_mut()
                .retain(|item| !revoked_token_identities.contains(item));
            if occupied.get().is_empty() {
                occupied.remove();
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx), fields(room = self.subroom_id))]
    pub fn issue_popout_stream_access_token(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        let identity = format!(
            "{}-popout{}",
            build_livekit_participant_id(participant_id, connection_id),
            self.token_identities
                .get(&(participant_id, connection_id))
                .map(BTreeSet::len)
                .unwrap_or_default()
        );

        let token = AccessToken::with_api_key(&self.settings.api_key, &self.settings.api_secret)
            .with_name(&identity)
            .with_identity(&identity)
            .with_grants(VideoGrants {
                room_create: false,
                room_list: false,
                room_record: false,
                room_admin: false,
                room_join: true,
                room: self.identifier().to_string(),
                can_publish: false,
                can_subscribe: true,
                can_publish_data: false,
                can_publish_sources: vec![],
                can_update_own_metadata: false,
                ingress_admin: false,
                hidden: true,
                recorder: false,
                destination_room: String::new(),
            })
            .with_ttl(ACCESS_TOKEN_TTL)
            .to_jwt()
            .map_err(|err| {
                tracing::error!("failed to create popout stream access token: {}", err);
                LiveKitError::LivekitUnavailable
            })?;

        self.token_identities
            .entry((participant_id, connection_id))
            .or_default()
            .insert(identity);

        ctx.send_ws_message(
            [participant_id],
            LiveKitEvent::PopoutStreamAccessToken { token },
        )?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx), fields(room = self.subroom_id))]
    pub fn update_microphone_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        new_state: MicrophoneRestrictionState,
        return_channel: oneshot::Sender<
            Result<MicrophoneRestrictionState, MicrophoneRestrictionError>,
        >,
    ) -> Result<(), ChannelDroppedError> {
        let local_lock = Arc::clone(&self.ongoing_microphone_restrictions);
        let Ok(guard) = local_lock.try_lock_owned() else {
            tracing::debug!(
                "Received microphone restriction request during ongoing restriction update"
            );
            return return_channel
                .send(Err(MicrophoneRestrictionError {
                    sender,
                    error: MicrophoneRestrictionErrorKind::ConflictingTask,
                }))
                .map_err(|_| ChannelDroppedError);
        };

        let room = self.identifier().to_string();
        let livekit_client = Arc::clone(&self.livekit_client);
        let connections = ctx.participants.connections();

        // update the state now so that the rule is already applied to new participants.
        self.microphone_restrictions = new_state.clone();

        ctx.spawn_optional({
            loopback::update_restricted_microphones(
                livekit_client,
                room,
                sender,
                new_state,
                connections,
            )
            .map(move |res| {
                drop(guard);
                if return_channel.send(res).is_err() {
                    tracing::error!("Channel dropped when sending microphone restriction result");
                }
                None
            })
        });

        Ok(())
    }

    fn notify_unknown_participants(
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        unknown_participants: BTreeSet<ParticipantId>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        if !unknown_participants.is_empty() {
            ctx.send_ws_message(
                [sender],
                LiveKitError::UnknownParticipant {
                    participant: unknown_participants,
                }
                .into(),
            )?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx), fields(room = self.subroom_id))]
    pub(crate) fn start_revoke_participant_access(
        &self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) {
        // When a participant leaves, we revoke their access to livekit.
        // The identities will be removed from the set when the loopback task was successful.
        let Some(token_identities) = self
            .token_identities
            .get(&(participant_id, connection_id))
            .cloned()
        else {
            tracing::warn!("No livekit token identities found");
            return;
        };
        let livekit_client = self.livekit_client.clone();

        ctx.spawn(loopback::revoke_token(
            livekit_client,
            participant_id,
            connection_id,
            self.identifier().to_string(),
            token_identities,
        ));
    }

    #[tracing::instrument(level = "debug", skip(self, ctx), fields(room = self.subroom_id))]
    pub(crate) fn set_screenshare_permissions(
        &self,
        ctx: &mut ModuleContext<'_, LiveKitModule>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
        grant: bool,
    ) -> Result<(), SignalingModuleError<LiveKitError>> {
        if !ctx.is_moderator(sender) {
            tracing::debug!(
                "Participant has insufficient permission to grant screen sharing rights: {sender}"
            );
            return Err(LiveKitError::InsufficientPermissions.into());
        }

        // check that we know all participants and query their connection ids
        let known_participants: BTreeSet<_> = ctx
            .participants
            .connected()
            .ids()
            .filter(|p| participants.contains(p))
            .collect();
        let unknown_participants: BTreeSet<_> = participants
            .difference(&known_participants)
            .copied()
            .collect();

        let mut connections = ctx.participants.connections();
        connections.retain(|p, _| known_participants.contains(p));
        Self::notify_unknown_participants(ctx, unknown_participants, sender)?;

        ctx.spawn(loopback::set_screenshare_permissions(
            Arc::clone(&self.livekit_client),
            self.identifier().to_string(),
            sender,
            connections,
            grant,
        ));

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(room = self.subroom_id))]
    pub async fn cleanup_room(self) {
        match self.livekit_client.delete_room(self.identifier()).await {
            Ok(()) => {
                tracing::debug!("Destroyed livekit room");
            }
            Err(ServiceError::Twirp(TwirpError::Twirp(code)))
                if code.code == TwirpErrorCode::NOT_FOUND =>
            {
                tracing::debug!("Livekit room was already destroyed");
            }
            Err(e) => {
                tracing::error!("Failed to destroy livekit room: {}", e);
            }
        }
    }
}

fn build_subroom_id(room_id: RoomId, room_kind: RoomKind) -> String {
    match room_kind {
        RoomKind::Breakout(breakout_id) => format!("{room_id}:{breakout_id}"),
        RoomKind::Main => format!("{room_id}:main"),
    }
}

/// Information about a participant's connection to a LiveKit room.
pub struct LiveKitConnection {
    pub participant_id: ParticipantId,
    pub livekit_participant_id: String,
    pub livekit_room: String,
}

impl LiveKitConnection {
    pub fn new(
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        room_id: RoomId,
        room_kind: RoomKind,
    ) -> LiveKitConnection {
        Self {
            participant_id,
            livekit_participant_id: build_livekit_participant_id(participant_id, connection_id),
            livekit_room: build_subroom_id(room_id, room_kind),
        }
    }
}
