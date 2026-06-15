// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::BTreeMap, fmt::Debug, marker::PhantomData, sync::Arc, time::Duration};

use anyhow::{Context, anyhow};
use opentalk_roomserver_common::settings::Task;
use opentalk_roomserver_types::{
    breakout::BreakoutRoom,
    connection_id::ConnectionId,
    room_kind::RoomKind,
    room_parameters::RoomParameters,
    shared_json::SharedJson,
    signaling::module_error::{FatalError, ModuleError, SignalingModuleError},
};
use opentalk_types_common::{features::FeatureId, modules::ModuleId};
use opentalk_types_signaling::{ParticipantId, SignalingModuleFrontendData};
use serde::{Deserialize, Serialize};
use serde_json::to_value;

use super::module_context::ModuleContext;
use crate::participant_state::ParticipantState;

/// The trait that defines a signaling module
///
/// Implementors can be added as a module to the room task. The room task will forward signaling
/// events to the module with the corresponding [`SignalingModule::NAMESPACE`]. All event calls are
/// handled in sequence on the same task. Signaling modules are expected to spawn separate tasks
/// when compute intense or long-running operations need to be executed (See
/// [`SignalingModule::Loopback`] for more details).
pub trait SignalingModule: Send + Sync + Sized + SignalingModuleDescription {
    /// The unique namespace for the module
    ///
    /// This is used as a general identifier to dispatch incoming signaling messages to the correct
    /// module.
    const NAMESPACE: ModuleId;

    /// The incoming websocket payload which is received as in
    /// [`SignalingModule::on_websocket_message`]
    type Incoming: for<'de> Deserialize<'de> + Send + CreateReplica<Self::Outgoing>;

    /// The outgoing websocket payload that is sent to the clients
    type Outgoing: Serialize + PartialEq + Debug + From<Self::Error> + Send;

    /// The incoming command which is received from other [`SignalingModule`]s
    type Internal: InternalCommand;

    /// Internal result type for asynchronous tasks
    ///
    /// These are received in the [`SignalingModule::on_loopback_event`] when an asynchronous task
    /// created by the module finishes.
    ///
    /// Tasks can be created with [`ModuleContext::spawn`] or [`ModuleContext::spawn_blocking`].
    type Loopback: Send + 'static;

    /// Namespaced data that can be attached to a participants `JoinSuccess` message
    type JoinInfo: SignalingModuleFrontendData + Clone + Send;

    /// Namespaced data that can be attached to the `Joined` message
    ///
    /// When a participant connects they trigger a `Joined` event for all other participants in the
    /// conference. Modules can append this type to the message to communicate module specific
    /// state of a new participant to the other participants.
    type PeerJoinInfo: Serialize + Send + 'static;

    /// The non-fatal error that can be returned from signaling module event handlers
    ///
    /// Is converted into a websocket event and returned to the command issuing participant
    ///
    /// Use [`Infallible`](std::convert::Infallible) if there is no error case.
    type Error: ModuleError;

    /// Creates an instance of the interface to access the module
    fn init(init_data: SignalingModuleInitData) -> Option<Self>;

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>>;

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>>;

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>>;

    #[allow(unused_variables)]
    fn on_websocket_message_lobby(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Err(SignalingModuleError::NotSupported)
    }

    #[allow(unused_variables)]
    fn on_breakout_start(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        duration: Option<Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        Ok(ModuleSwitchData::default())
    }

    #[allow(unused_variables)]
    fn on_breakout_closing(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_breakout_closed(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn on_internal_command(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        command: Self::Internal,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Err(anyhow!(
            "Received internal command in module {} that does not support internal commands",
            Self::NAMESPACE
        )
        .into())
    }

    /// Destroy the module and remove all associated resources
    ///
    /// Loopback tasks are awaited before the room fully closes.
    #[allow(unused_variables)]
    fn on_closing(&mut self, ctx: &mut ModuleContext<'_, Self>) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub trait CreateReplica<T> {
    fn replicate(&self) -> Option<T>;
}

/// Trait that allows dynamic dispatch of [`SignalingModule::Internal`]
pub trait InternalCommand: Send + 'static {}

pub enum NoOp {}

impl InternalCommand for NoOp {}

pub struct ModuleJoinData<M: SignalingModule> {
    /// Module specific data that will be attached to the participants [`JoinSuccess::module_data`]
    /// event.
    ///
    /// [`JoinSuccess::module_data`]: opentalk_roomserver_types::join::join_success::JoinSuccess::module_data
    pub join_success: Option<M::JoinInfo>,

    /// Module specific data that will be attached to other participants
    /// [`ParticipantConnected::peer_data`] event.
    ///
    /// [`ParticipantConnected::peer_data`]: opentalk_roomserver_types::core::CoreEvent::ParticipantConnected::peer_data
    pub peer_events: PeerDataMap<M>,

    /// Module specific data that will be attached to the information about other participants
    /// inside the [`JoinSuccess::participants`] (specifically [`Participant::module_data`]).
    ///
    /// [`JoinSuccess::participants`]: opentalk_roomserver_types::join::join_success::JoinSuccess::participants
    /// [`Participant::module_data`]: opentalk_roomserver_types::join::participant::Participant::module_data
    pub peer_data: PeerDataMap<M>,
}

impl<M: SignalingModule> Default for ModuleJoinData<M> {
    fn default() -> Self {
        Self {
            join_success: Default::default(),
            peer_events: Default::default(),
            peer_data: Default::default(),
        }
    }
}

/// Similar to [`ModuleJoinData`], but with a `switch_success` for each connection of
/// the switching participant.
///
/// Different to a join, during a switch all connections join the new room and
/// therefore all need initial information.
pub struct ModuleSwitchData<M: SignalingModule> {
    /// Module specific data that will be attached to the participants [`SwitchedRoom::own_data`]
    /// event.
    ///
    /// [`SwitchedRoom::own_data`]: opentalk_roomserver_types::breakout::event::BreakoutEvent::SwitchedRoom::own_data
    pub switch_success: BTreeMap<ConnectionId, Option<<M as SignalingModule>::JoinInfo>>,

    /// Module specific data that will be attached to other participants
    /// [`ParticipantSwitchedRoom::module_data`] event.
    ///
    /// [`ParticipantSwitchedRoom::module_data`]: opentalk_roomserver_types::breakout::event::BreakoutEvent::ParticipantSwitchedRoom::module_data
    pub peer_events: PeerDataMap<M>,

    /// Module specific data that will be attached to the information about other participants
    /// inside the [`SwitchedRoom::peer_data`].
    ///
    /// [`SwitchedRoom::peer_data`]: opentalk_roomserver_types::breakout::event::BreakoutEvent::SwitchedRoom::peer_data
    pub peer_data: PeerDataMap<M>,
}

impl<M: SignalingModule> ModuleSwitchData<M> {
    pub fn new() -> Self {
        Self {
            switch_success: Default::default(),
            peer_events: Default::default(),
            peer_data: Default::default(),
        }
    }
}

impl<M: SignalingModule> Default for ModuleSwitchData<M> {
    fn default() -> Self {
        Self::new()
    }
}

/// A map of participants and their [`SignalingModule::PeerJoinInfo`]
///
/// When a `Joined` message is sent to a participant, the participants associated
/// [`SignalingModule::PeerJoinInfo`] will be attached to it.
pub struct PeerDataMap<M: SignalingModule> {
    pub map: BTreeMap<ParticipantId, SharedJson>,
    _m: PhantomData<M>,
}

impl<M: SignalingModule> PeerDataMap<M> {
    pub fn new(map: BTreeMap<ParticipantId, SharedJson>) -> Self {
        Self {
            map,
            _m: PhantomData,
        }
    }
}

impl<M: SignalingModule> Default for PeerDataMap<M> {
    fn default() -> Self {
        Self {
            map: Default::default(),
            _m: PhantomData,
        }
    }
}

impl<M: SignalingModule> PeerDataMap<M> {
    /// Attach the same [`PeerJoinInfo`](SignalingModule::PeerJoinInfo) for all participants
    pub fn insert_for_all(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        info: M::PeerJoinInfo,
    ) -> Result<(), FatalError> {
        self.insert_for_matching(ctx, info, |_, _| true)
    }

    /// Attach [`PeerJoinInfo`](SignalingModule::PeerJoinInfo) to a single participant
    pub fn insert(
        &mut self,
        participant_id: ParticipantId,
        info: M::PeerJoinInfo,
    ) -> Result<(), FatalError> {
        let raw_value = SharedJson::from(
            to_value(&info)
                .with_context(|| {
                    format!(
                        "Failed to serialize PeerJoinInfo for module '{}'",
                        M::NAMESPACE
                    )
                })
                .map_err(FatalError)?,
        );

        self.map.insert(participant_id, raw_value);

        Ok(())
    }

    /// Attach [`PeerJoinInfo`](SignalingModule::PeerJoinInfo) for all participants that match the
    /// given filter
    pub fn insert_for_matching<F>(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        info: M::PeerJoinInfo,
        mut filter: F,
    ) -> Result<(), FatalError>
    where
        F: FnMut(ParticipantId, &ParticipantState) -> bool,
    {
        // Lazily serialize the PeerJoinInfo into a json string
        let mut raw_value: Option<SharedJson> = None;

        for (participant_id, state) in ctx.participants.connected().iter() {
            if !filter(*participant_id, state) {
                continue;
            }

            // Serialize the PeerJoinInfo if it hasn't already
            let raw_value = match &mut raw_value {
                Some(raw_value) => raw_value,
                None => raw_value.insert(SharedJson::from(
                    to_value(&info)
                        .with_context(|| {
                            format!(
                                "Failed to serialize PeerJoinInfo for module '{}'",
                                M::NAMESPACE
                            )
                        })
                        .map_err(FatalError)?,
                )),
            };

            self.map.insert(*participant_id, raw_value.clone());
        }

        Ok(())
    }
}

/// Data that a signaling module might require to initialize
#[derive(Clone, Debug)]
pub struct SignalingModuleInitData {
    /// The roomserver settings
    pub settings: Arc<Task>,
    /// The room parameters that are used to initialize the room
    pub room_parameters: Arc<RoomParameters>,
}

pub struct SignalingModuleFeatureDescription {
    pub feature_id: FeatureId,
    pub description: &'static str,
}

pub trait SignalingModuleDescription {
    const MODULE_ID: ModuleId;
    const DESCRIPTION: &'static str;
    const FEATURES: &[SignalingModuleFeatureDescription];
}
