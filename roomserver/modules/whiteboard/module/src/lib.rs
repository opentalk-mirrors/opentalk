// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use anyhow::anyhow;
use futures::{StreamExt, stream};
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
    storage::assets::ModuleAssetStorage,
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_whiteboard::{
    WHITEBOARD_MODULE_ID, WhiteboardCommand, WhiteboardError, WhiteboardEvent, WhiteboardSettings,
    WhiteboardState, state::SpaceInfo,
};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;
use tracing::{Instrument, Span};

use crate::{client::SpacedeckClient, loopback::WhiteboardLoopback};

const PARALLEL_UPDATES: usize = 25;

pub mod client;
mod loopback;

enum InitState {
    Initializing,
    Initialized(SpaceInfo),
}

impl From<&InitState> for WhiteboardState {
    fn from(value: &InitState) -> Self {
        match value {
            InitState::Initializing => Self::Initializing,
            InitState::Initialized(SpaceInfo { url, .. }) => Self::Initialized(url.clone()),
        }
    }
}

pub struct WhiteboardModule {
    state: HashMap<RoomKind, InitState>,
    client: Arc<SpacedeckClient>,
}

impl SignalingModuleDescription for WhiteboardModule {
    const MODULE_ID: ModuleId = WHITEBOARD_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles whiteboard integration. The whiteboard is a collaborative drawing board that can be used during the meeting.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = WhiteboardSettings;
}

impl SignalingModule for WhiteboardModule {
    const NAMESPACE: ModuleId = WHITEBOARD_MODULE_ID;

    type Incoming = WhiteboardCommand;

    type Outgoing = WhiteboardEvent;

    type Internal = NoOp;

    type Loopback = Result<WhiteboardLoopback, SignalingModuleError<WhiteboardError>>;

    type JoinInfo = WhiteboardState;

    type PeerJoinInfo = ();

    type Error = WhiteboardError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let settings = init_data
            .room_parameters
            .module_settings
            .get::<WhiteboardSettings>()
            .inspect_err(|err| {
                tracing::error!("Failed to deserialize whiteboard settings: {err:?}")
            })
            .ok()??;
        let spacedeck = SpacedeckClient::new(settings.base_url, settings.api_key);

        Some(Self {
            state: HashMap::new(),
            client: Arc::new(spacedeck),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        if let Some(state) = self.state.get(&ctx.room) {
            Ok(ModuleJoinData {
                join_success: Some(state.into()),
                peer_events: PeerDataMap::default(),
                peer_data: PeerDataMap::default(),
            })
        } else {
            Ok(ModuleJoinData::default())
        }
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            WhiteboardCommand::Initialize => self.initialize(ctx, sender),
            WhiteboardCommand::GeneratePdf => self.generate_pdf(ctx, sender),
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            Ok(WhiteboardLoopback::SpaceCreated { info }) => {
                tracing::debug!(
                    "Spacedeck space for room {:?} created: {:?}",
                    ctx.room,
                    info.id
                );
                let url = info.url.clone();
                let previous = self.state.insert(ctx.room, InitState::Initialized(info));
                if let Some(previous) = previous
                    && matches!(previous, InitState::Initialized(..))
                {
                    tracing::warn!("Spacedeck created, but a previous one already existed");
                }
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    WhiteboardEvent::Initialized { url },
                )?;
            }
            Ok(WhiteboardLoopback::PdfCreated { asset }) => ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                WhiteboardEvent::PdfCreated {
                    filename: asset.filename,
                    asset_id: asset.id,
                },
            )?,
            Err(SignalingModuleError::Module(WhiteboardError::InitializationFailed)) => {
                self.state.remove(&ctx.room);
                return Err(WhiteboardError::InitializationFailed.into());
            }
            Err(err) => return Err(err),
        };

        Ok(())
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        let Some(state) = self.state.get(&new_room) else {
            // There is no whiteboard in the new room, nothing to do.
            return Ok(ModuleSwitchData::default());
        };

        let switch_success: BTreeMap<ConnectionId, Option<WhiteboardState>> = ctx
            .participant_state(participant_id)
            .ok_or(FatalError(anyhow!(
                "Participant {participant_id:?} switched without participant state"
            )))?
            .connections()
            .map(|connection_id| (connection_id, Some(state.into())))
            .collect();

        Ok(ModuleSwitchData {
            switch_success,
            peer_events: PeerDataMap::default(),
            peer_data: PeerDataMap::default(),
        })
    }

    fn on_breakout_closed(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let breakout_rooms: Vec<String> = self
            .state
            .extract_if(|&room, _| room != RoomKind::Main)
            .filter_map(|(.., state)| match state {
                InitState::Initializing => None,
                InitState::Initialized(SpaceInfo { id, .. }) => Some(id),
            })
            .collect();
        Self::delete_spaces(
            Arc::clone(&self.client),
            ctx.assets(),
            breakout_rooms,
            ctx.timestamp,
        );

        Ok(())
    }

    fn on_closing(&mut self, ctx: &mut ModuleContext<'_, Self>) -> anyhow::Result<()> {
        let spaces: Vec<String> = self
            .state
            .drain()
            .filter_map(|(.., state)| match state {
                InitState::Initializing => None,
                InitState::Initialized(SpaceInfo { id, .. }) => Some(id),
            })
            .collect();
        Self::delete_spaces(
            Arc::clone(&self.client),
            ctx.assets(),
            spaces,
            Timestamp::now(),
        );

        Ok(())
    }
}

impl WhiteboardModule {
    fn initialize(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !ctx.is_moderator(sender) {
            return Err(WhiteboardError::InsufficientPermissions.into());
        }

        if let Some(state) = self.state.get(&ctx.room) {
            match state {
                InitState::Initializing => {
                    return Err(WhiteboardError::CurrentlyInitializing.into());
                }
                InitState::Initialized(..) => {
                    return Err(WhiteboardError::AlreadyInitialized.into());
                }
            }
        }

        self.state.insert(ctx.room, InitState::Initializing);
        ctx.spawn(loopback::create_space(
            Arc::clone(&self.client),
            ctx.room_id,
            ctx.room,
        ));

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            WhiteboardEvent::InitializationStarted,
        )?;

        Ok(())
    }

    fn generate_pdf(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !ctx.is_moderator(sender) {
            return Err(WhiteboardError::InsufficientPermissions.into());
        }

        let id = match self.state.get(&ctx.room) {
            Some(InitState::Initialized(SpaceInfo { id, .. })) => id.to_owned(),
            Some(InitState::Initializing) => {
                return Err(WhiteboardError::CurrentlyInitializing.into());
            }
            None => return Err(WhiteboardError::NotInitialized.into()),
        };
        ctx.spawn(loopback::generate_pdf(
            Arc::clone(&self.client),
            ctx.assets(),
            id,
            ctx.timestamp,
        ));

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(spaces), level = "debug")]
    fn delete_spaces(
        client: Arc<SpacedeckClient>,
        storage: ModuleAssetStorage,
        spaces: Vec<String>,
        timestamp: Timestamp,
    ) {
        let span = Span::current();
        let future = stream::iter(spaces)
            .map(move |id| {
                let client = Arc::clone(&client);
                let storage_client = storage.clone();
                let span = span.clone();

                async move {
                    loopback::delete_space(client, storage_client, id, timestamp).await;
                }
                .instrument(span)
            })
            .buffer_unordered(PARALLEL_UPDATES)
            .collect::<Vec<()>>()
            .in_current_span();
        tokio::spawn(future);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::WhiteboardModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&WhiteboardModule::example_config()).unwrap();

        assert_snapshot!(produced, @r#"
        [whiteboard]
        api_key = "secret"
        base_url = "http://localhost:9666/"
        "#);
    }
}
