// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

use anyhow::{Context, anyhow};
use opentalk_etherpad_client::EtherpadClient;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_meeting_notes::{
    MEETING_NOTES_MODULE_ID, MeetingNotesCommand, MeetingNotesError, MeetingNotesEvent,
    MeetingNotesPeerState, MeetingNotesSettings,
};
use opentalk_types_common::{modules::ModuleId, rooms::RoomId, users::DisplayName};
use opentalk_types_signaling::ParticipantId;
use url::Url;

use crate::loopback::{GenerateUrlFailed, MeetingNotesLoopback};

mod loopback;

const PAD_NAME: &str = "meeting_notes";

#[derive(Debug)]
enum InitState {
    Initializing,
    Initialized {
        group_id: String,
        sessions: BTreeMap<ConnectionId, SessionInfo>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    id: String,
    readonly: bool,
}

#[derive(Debug, Clone)]
struct CreateSession {
    participant_id: ParticipantId,
    connection_id: ConnectionId,
    readonly: bool,
    display_name: DisplayName,
    existing_session_id: Option<String>,
}

#[derive(PartialEq, Eq)]
pub struct SessionUrl {
    session: SessionInfo,
    participant_id: ParticipantId,
    url: Url,
}

pub struct MeetingNotesModule {
    etherpad: Arc<EtherpadClient>,
    etherpad_rooms: HashMap<RoomKind, InitState>,
}

impl SignalingModuleDescription for MeetingNotesModule {
    const MODULE_ID: ModuleId = MEETING_NOTES_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles meeting note editing and viewing functionality";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = MeetingNotesSettings;
}

impl SignalingModule for MeetingNotesModule {
    const NAMESPACE: ModuleId = MEETING_NOTES_MODULE_ID;

    type Incoming = MeetingNotesCommand;

    type Outgoing = MeetingNotesEvent;

    type Internal = NoOp;

    type Loopback = Result<MeetingNotesLoopback, SignalingModuleError<MeetingNotesError>>;

    type JoinInfo = ();

    type PeerJoinInfo = MeetingNotesPeerState;

    type Error = MeetingNotesError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let settings = init_data
            .room_parameters
            .module_settings
            .get::<MeetingNotesSettings>()
            .ok()??;
        let etherpad = EtherpadClient::new(settings.base_url, settings.api_key);

        Some(Self {
            etherpad: Arc::new(etherpad),
            etherpad_rooms: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let Some(InitState::Initialized { group_id, sessions }) =
            self.etherpad_rooms.get(&RoomKind::Main)
        else {
            // Meeting notes not active, nothing to do
            return Ok(ModuleJoinData::default());
        };

        // Create a new session for the new connection
        let display_name = ctx
            .participant_state(participant_id)
            .with_context(|| {
                anyhow!("Participant {participant_id:?} joined without participant state")
            })?
            .display_name
            .clone();
        ctx.spawn(loopback::generate_url(
            Arc::clone(&self.etherpad),
            participant_id,
            connection_id,
            !ctx.is_moderator(participant_id),
            display_name,
            None,
            group_id.to_owned(),
        ));

        let peer_data = Self::peer_data(ctx, participant_id, sessions)?;

        Ok(ModuleJoinData {
            join_success: None,
            peer_events: PeerDataMap::default(),
            peer_data,
        })
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.delete_session(ctx, ctx.room, connection_id);
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
            MeetingNotesCommand::GrantWriteAccess { participant_ids } => {
                self.grant_write_access(ctx, sender, &participant_ids)
            }
            MeetingNotesCommand::RevokeWriteAccess { participant_ids } => {
                self.revoke_write_access(ctx, sender, &participant_ids)
            }
            MeetingNotesCommand::GeneratePdf => self.generate_pdf(ctx, sender, connection_id),
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            Ok(MeetingNotesLoopback::Initialized { group_id, results }) => {
                let Some(init_state) = self.etherpad_rooms.get_mut(&ctx.room) else {
                    // This can be the case when the breakout room has been closed while
                    // initialization was in progress In this case the sessions
                    // do not need to be deleted because the group has already been deleted,
                    // which removes all sessions. We also do not want to send a WebSocket message
                    // here.
                    return Ok(());
                };
                *init_state = InitState::Initialized {
                    group_id,
                    sessions: BTreeMap::new(),
                };
                self.handle_write_access_change(ctx, results)?;
            }
            Ok(MeetingNotesLoopback::WritersUpdated { results }) => {
                self.handle_write_access_change(ctx, results)?;
            }
            Ok(MeetingNotesLoopback::PdfGenerated { asset }) => {
                tracing::debug!("Generated meeting notes: {asset:?}");
                ctx.send_ws_message(
                    ctx.participants.connected().ids(),
                    MeetingNotesEvent::PdfCreated {
                        filename: asset.filename,
                        asset_id: asset.id,
                    },
                )?;
            }
            Err(SignalingModuleError::Module(MeetingNotesError::FailedInitialization)) => {
                self.etherpad_rooms.remove(&ctx.room);
                return Err(MeetingNotesError::FailedInitialization.into());
            }
            Err(e) => return Err(e),
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
        // If there is a meeting notes session in the old room
        if let Some(InitState::Initialized {
            sessions: old_room_sessions,
            ..
        }) = self.etherpad_rooms.get_mut(&old_room)
        {
            // Delete the sessions associated with the participant
            let state = ctx
                .participant_state(participant_id)
                .ok_or(FatalError(anyhow!(
                    "Participant {participant_id:?} switched without participant state"
                )))?;

            let session_ids = state
                .connections()
                .filter_map(|connection_id| old_room_sessions.get(&connection_id))
                .map(|session| session.id.clone())
                .collect();

            let client = Arc::clone(&self.etherpad);
            ctx.spawn_optional(loopback::delete_sessions(client, session_ids));
        }

        let Some(InitState::Initialized {
            group_id: new_room_group_id,
            sessions: new_room_sessions,
        }) = self.etherpad_rooms.get(&new_room)
        else {
            // Meeting notes not active in the new room, nothing to do
            return Ok(ModuleSwitchData::default());
        };

        // Create new sessions in the new room
        self.generate_urls(
            ctx,
            new_room_group_id.clone(),
            None,
            [&participant_id],
            !ctx.is_moderator(participant_id),
        );

        let peer_data = Self::peer_data(ctx, participant_id, new_room_sessions)?;

        Ok(ModuleSwitchData {
            switch_success: BTreeMap::default(),
            peer_events: PeerDataMap::default(),
            peer_data,
        })
    }

    fn on_breakout_closed(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let breakout_rooms: Vec<InitState> = self
            .etherpad_rooms
            .extract_if(|&room, _| room != RoomKind::Main)
            .map(|(.., state)| state)
            .collect();
        let client = Arc::clone(&self.etherpad);

        ctx.spawn_optional(async move {
            loopback::delete_pads(client, breakout_rooms.into_iter()).await;
            None
        });

        Ok(())
    }

    fn on_closing(&mut self, ctx: &mut ModuleContext<'_, Self>) -> anyhow::Result<()> {
        let rooms: Vec<InitState> = self
            .etherpad_rooms
            .drain()
            .map(|(.., state)| state)
            .collect();
        let client = Arc::clone(&self.etherpad);

        ctx.spawn_optional(async move {
            loopback::delete_pads(client, rooms.into_iter()).await;
            None
        });

        Ok(())
    }
}

impl MeetingNotesModule {
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn initialize_etherpad(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        writers: &BTreeSet<ParticipantId>,
    ) {
        let client = Arc::clone(&self.etherpad);
        let participants = ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .iter()
            .flat_map(|(participant_id, state)| {
                state
                    .connections
                    .iter()
                    .map(|(connection_id, ..)| CreateSession {
                        participant_id: *participant_id,
                        connection_id: *connection_id,
                        readonly: !writers.contains(participant_id),
                        display_name: state.display_name.clone(),
                        existing_session_id: None,
                    })
            })
            .collect();
        ctx.spawn(loopback::initialize_etherpad(
            client,
            build_pad_mapped_id(ctx.room_id, ctx.room),
            participants,
        ));
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn grant_write_access(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        writers: &BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<MeetingNotesError>> {
        if !ctx.is_moderator(sender) {
            return Err(MeetingNotesError::InsufficientPermissions.into());
        }

        if !Self::verify_selection(ctx, writers) {
            return Err(MeetingNotesError::InvalidParticipantSelection.into());
        }

        match &self.etherpad_rooms.get(&ctx.room) {
            None => {
                // Initialize etherpad
                self.etherpad_rooms
                    .insert(ctx.room, InitState::Initializing);
                self.initialize_etherpad(ctx, writers);
            }
            Some(InitState::Initializing) => {
                // Initialization is already in progress
                return Err(MeetingNotesError::CurrentlyInitializing.into());
            }
            Some(InitState::Initialized { group_id, sessions }) => {
                // Generate a new url for the new writers
                self.generate_urls(ctx, group_id.clone(), Some(sessions), writers.iter(), false);
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn revoke_write_access(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        participant_ids: &BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<MeetingNotesError>> {
        if !ctx.is_moderator(sender) {
            return Err(MeetingNotesError::InsufficientPermissions.into());
        }

        if !Self::verify_selection(ctx, participant_ids) {
            return Err(MeetingNotesError::InvalidParticipantSelection.into());
        }

        let (group_id, sessions) = self.room_session(ctx.room)?;

        self.generate_urls(
            ctx,
            group_id.to_owned(),
            Some(sessions),
            participant_ids,
            true,
        );

        Ok(())
    }

    /// Handles the results of a write access change operation, sending the appropriate WebSocket
    /// messages to the participants whose access changed and the moderators. Also cleans up
    /// sessions for connections that disconnected before receiving their URL.
    #[tracing::instrument(skip_all, level = "debug")]
    fn handle_write_access_change(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        results: BTreeMap<ConnectionId, Result<SessionUrl, GenerateUrlFailed>>,
    ) -> Result<(), SignalingModuleError<MeetingNotesError>> {
        let Some(InitState::Initialized { sessions, .. }) = self.etherpad_rooms.get_mut(&ctx.room)
        else {
            tracing::debug!("Received writers update for uninitialized room");
            // Delete all sessions
            let session_ids: Vec<String> = results
                .into_values()
                .filter_map(Result::ok)
                .map(|s| s.session.id)
                .collect();
            ctx.spawn_optional(loopback::delete_sessions(
                Arc::clone(&self.etherpad),
                session_ids,
            ));
            return Ok(());
        };

        let mut disconnected_connections = Vec::new();
        let mut failed_connections = Vec::new();
        let mut readers = Vec::new();
        let mut writers = Vec::new();

        for (connection_id, result) in results {
            match result {
                Ok(SessionUrl {
                    session,
                    participant_id,
                    url,
                }) => {
                    if !ctx
                        .participant_state(participant_id)
                        .is_some_and(|state| state.connections.contains_key(&connection_id))
                    {
                        tracing::debug!(
                            "Connection '{connection_id}' of participant '{participant_id}' disconnected before receiving etherpad session",
                        );
                        disconnected_connections.push(session.id);
                        continue;
                    }

                    let msg = if session.readonly {
                        tracing::debug!(
                            "Read-only URL successfully generated for connection '{connection_id}' of participant '{participant_id}'",
                        );
                        readers.push(connection_id);
                        MeetingNotesEvent::ReadAccessReceived {
                            url: url.to_string(),
                        }
                    } else {
                        tracing::debug!(
                            "Read-write URL successfully generated for connection '{connection_id}' of participant '{participant_id}'",
                        );
                        writers.push(connection_id);
                        MeetingNotesEvent::WriteAccessReceived {
                            url: url.to_string(),
                        }
                    };
                    sessions.insert(connection_id, session);
                    ctx.send_ws_message_to_connections([connection_id], msg)?;
                }
                Err(GenerateUrlFailed { participant_id }) => {
                    tracing::error!(
                        "Failed to generate etherpad URL for connection '{connection_id}' of participant '{participant_id}'",
                    );
                    failed_connections.push(connection_id);
                }
            }
        }

        // Notify moderators about successful changes
        if !readers.is_empty() || !writers.is_empty() {
            ctx.send_ws_message(
                ctx.participants
                    .in_room(ctx.room)
                    .moderators()
                    .connected()
                    .ids(),
                MeetingNotesEvent::AccessChanged { readers, writers },
            )?;
        }

        // Notify moderators about failed changes
        if !failed_connections.is_empty() {
            ctx.send_ws_message(
                ctx.participants
                    .in_room(ctx.room)
                    .moderators()
                    .connected()
                    .ids(),
                MeetingNotesEvent::Error(MeetingNotesError::FailedToGenerateUrl {
                    connection_ids: failed_connections,
                }),
            )?;
        }

        // Clean up sessions for connections that disconnected before receiving their session
        ctx.spawn_optional(loopback::delete_sessions(
            Arc::clone(&self.etherpad),
            disconnected_connections,
        ));

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn generate_pdf(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<MeetingNotesError>> {
        if !ctx.is_moderator(sender) {
            return Err(MeetingNotesError::InsufficientPermissions.into());
        }

        let (group_id, sessions) = self.room_session(ctx.room)?;
        let session_id = sessions
            .get(&connection_id)
            .map(|session| session.id.clone())
            .ok_or(anyhow!(
                "Participant '{sender:?}' is requesting PDF generation but has no etherpad session"
            ))?;
        let pad_id = pad_name(group_id);

        ctx.spawn(loopback::generate_pdf(
            Arc::clone(&self.etherpad),
            ctx.assets(),
            pad_id,
            session_id,
            ctx.timestamp,
        ));

        Ok(())
    }

    /// Returns `true` when all participants are present in the room the request is coming from. At
    /// least one participant must be selected.
    fn verify_selection(
        ctx: &ModuleContext<'_, Self>,
        selection: &BTreeSet<ParticipantId>,
    ) -> bool {
        if selection.is_empty() {
            return false;
        }

        selection.iter().all(|&id| {
            ctx.participant_state(id)
                .is_some_and(|state| state.room == ctx.room)
        })
    }

    /// Generates etherpad access URLs
    ///
    /// * `group_id` - The etherpad group ID
    /// * `room_sessions` - Existing sessions in the room, if any. Used to revoke prior sessions.
    /// * `participants` - The participants for whom to generate URLs
    /// * `readonly` - Whether to generate read-only URLs
    #[tracing::instrument(skip(self, ctx, room_sessions, participants), level = "debug")]
    fn generate_urls<'a>(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        group_id: String,
        room_sessions: Option<&BTreeMap<ConnectionId, SessionInfo>>,
        participants: impl IntoIterator<Item = &'a ParticipantId>,
        readonly: bool,
    ) {
        let client = Arc::clone(&self.etherpad);
        let requests = participants
            .into_iter()
            .filter_map(|participant_id| {
                let state = ctx.participant_state(*participant_id)?;
                let requests = state.connections().map(|connection_id| CreateSession {
                    participant_id: *participant_id,
                    connection_id,
                    readonly,
                    display_name: state.display_name.clone(),
                    existing_session_id: room_sessions
                        .and_then(|sessions| sessions.get(&connection_id).map(|s| s.id.clone())),
                });
                Some(requests)
            })
            .flatten()
            .collect();
        ctx.spawn(loopback::generate_urls(client, group_id, requests));
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn delete_session(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        room: RoomKind,
        connection_id: ConnectionId,
    ) {
        let Some(InitState::Initialized { sessions, .. }) = self.etherpad_rooms.get_mut(&room)
        else {
            // No sessions for this room, nothing to do
            return;
        };

        let Some(session) = sessions.remove(&connection_id) else {
            tracing::warn!("Connection '{connection_id}' had no etherpad session");
            return;
        };

        let client = Arc::clone(&self.etherpad);
        ctx.spawn_optional(async move {
            if let Err(e) = loopback::delete_session(&client, session.id).await {
                return Some(Err(e.into()));
            }
            None
        });
    }

    fn peer_data(
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        room_sessions: &BTreeMap<ConnectionId, SessionInfo>,
    ) -> Result<PeerDataMap<Self>, FatalError> {
        let mut peer_data = PeerDataMap::default();
        if ctx.is_moderator(participant_id) {
            // Send the current access state of every participant when the joining participant is a
            // moderator
            for (participant_id, state) in ctx.participants.connected().iter() {
                let readonly = state
                    .connections()
                    .filter_map(|connection_id| room_sessions.get(&connection_id))
                    .all(|session| session.readonly);

                peer_data.insert(*participant_id, MeetingNotesPeerState { readonly })?;
            }
        }
        Ok(peer_data)
    }

    fn room_session(
        &self,
        room: RoomKind,
    ) -> Result<(&str, &BTreeMap<ConnectionId, SessionInfo>), MeetingNotesError> {
        match self.etherpad_rooms.get(&room) {
            None => Err(MeetingNotesError::NotInitialized),
            Some(InitState::Initializing) => Err(MeetingNotesError::CurrentlyInitializing),
            Some(InitState::Initialized { group_id, sessions }) => Ok((group_id, sessions)),
        }
    }
}

fn pad_name(group_id: &str) -> String {
    format!("{group_id}${PAD_NAME}")
}

fn build_pad_mapped_id(room_id: RoomId, room_kind: RoomKind) -> String {
    match room_kind {
        RoomKind::Breakout(breakout_id) => format!("{room_id}:{breakout_id}"),
        RoomKind::Main => format!("{room_id}:main"),
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::MeetingNotesModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&MeetingNotesModule::example_config()).unwrap();

        assert_snapshot!(produced, @r#"
        [meeting_notes]
        api_key = "secret"
        base_url = "http://localhost:9001/"
        "#);
    }
}
