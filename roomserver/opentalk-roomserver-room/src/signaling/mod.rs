// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! All [`SignalingModule`]s need to be accessed from the same collection by the
//! [`RoomTask`](super::task::RoomTask). They can not be stored directly in a single collection,
//! because they use associated types and therefore are generic. To store them in a single
//! collection despite that, we make use of dynamic dispatch and polymorphism using
//! the [`ModuleHandle`] trait.
//!
//! The [`ModuleDispatcher`] acts as an intermediate between the generic JSON messages and the
//! [`SignalingModule`]s, turning received messages into concrete types that are specific to the
//! [`SignalingModule`]. This works because the [`ModuleDispatcher`] is generic over the
//! [`SignalingModule`] and can defer its specific types.
//!
//! Due to the generic nature of the [`ModuleDispatcher`] they cannot be stored in a single
//! collection either, at least not when their generic type differs. This is where the
//! [`ModuleHandle`] is used as an abstraction to remove any generic bounds. We achieve this with
//! dynamic dispatch by storing them as a `Box<dyn ModuleDispatcher>`.
use std::{any::Any, collections::BTreeMap, fmt::Display, time::Duration};

use anyhow::Context;
use dyn_module_context::DynModuleContext;
pub use module_initializer::{DescriptionPrinter, ModuleInitializer};
use opentalk_roomserver_signaling::{
    event_origin::EventOrigin,
    module_context::ModuleContext,
    signaling_module::{CreateReplica, SignalingModule},
};
use opentalk_roomserver_types::{
    breakout::{BREAKOUT_MODULE_ID, BreakoutRoom},
    connection_id::ConnectionId,
    core::CORE_MODULE_ID,
    error::SignalingError,
    room_kind::RoomKind,
    shared_json::SharedJson,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::{ModuleData, ParticipantId};
use serde_json::value::RawValue;
use tracing::{Span, field::Empty};

pub mod dyn_module_context;
pub(crate) mod module_initializer;

pub const CORE_MODULES: [ModuleId; 2] = [CORE_MODULE_ID, BREAKOUT_MODULE_ID];

/// Abstracted handle to a [`SignalingModule`]
pub trait ModuleHandle: Send + Sync {
    /// Invokes an event in the associated [`SignalingModule`]
    fn on_event(
        &mut self,
        // TODO: make this owned
        ctx: &mut DynModuleContext<'_>,
        event: DynEvent,
    ) -> Result<(), FatalError>;

    fn on_broadcast_event(
        &mut self,
        ctx: &mut DynModuleContext<'_>,
        event: &mut DynBroadcastEvent<'_>,
    ) -> Result<(), FatalError>;

    fn on_closing(&mut self, ctx: &mut DynModuleContext<'_>);
}

pub enum DynEvent {
    WebsocketMessage {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        command: Box<RawValue>,
    },
    LobbyWebsocketMessage {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        command: Box<RawValue>,
    },
    LoopbackEvent(Box<dyn Any + Send + 'static>),
    InternalCommand {
        sender: ModuleId,
        command: Box<dyn Any + Send + 'static>,
    },
}

pub enum DynBroadcastEvent<'evt> {
    Connected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        /// The module data for the participant in the new room.
        own_data: &'evt mut ModuleData,
        /// The module data about the joining participant, sent to the participants already in the
        /// room.
        peer_events: &'evt mut BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
        /// The module data about other participants in the room, for and send to the joining
        /// participant.
        peer_data: &'evt mut BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
    },

    Disconnected {
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    },

    Closing,

    BreakoutStart {
        rooms: &'evt Vec<BreakoutRoom>,
        duration: Option<Duration>,
    },

    /// Breakout rooms are about to be closed. All participants will be moved to the main room
    /// automatically.
    BreakoutClosing,

    /// Breakout rooms have been closed and all participant are moved back to the main room.
    BreakoutClosed,

    /// A participant switches between main and/or breakout rooms
    SwitchRoom {
        participant_id: ParticipantId,
        old_room: RoomKind,
        new_room: RoomKind,
        /// The module data for the participant in the new room. Each connection needs to have
        /// their own module data
        own_data: &'evt mut BTreeMap<ConnectionId, ModuleData>,
        /// The module data about the switching participant, sent to the participants already in
        /// the room.
        peer_events: &'evt mut BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
        /// The module data about other participants in the room, for and send to the switching
        /// participant.
        peer_data: &'evt mut BTreeMap<ParticipantId, BTreeMap<ModuleId, SharedJson>>,
    },

    /// The moderator role was granted or revoked for a participant.
    ModerationRoleChanged {
        participant_id: ParticipantId,
    },
}

impl Display for DynBroadcastEvent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DynBroadcastEvent::Connected { .. } => write!(f, "Connected"),
            DynBroadcastEvent::Disconnected { .. } => write!(f, "Disconnected"),
            DynBroadcastEvent::BreakoutStart { .. } => write!(f, "BreakoutStart"),
            DynBroadcastEvent::BreakoutClosing => write!(f, "BreakoutClosing"),
            DynBroadcastEvent::BreakoutClosed => write!(f, "BreakoutClosed"),
            DynBroadcastEvent::Closing => write!(f, "Closing"),
            DynBroadcastEvent::SwitchRoom { .. } => write!(f, "SwitchRoom"),
            DynBroadcastEvent::ModerationRoleChanged { .. } => write!(f, "ModerationRoleChanged"),
        }
    }
}

/// Resolves generic JSON messages into concrete types for the associated [`SignalingModule`]
///
/// Implements the [`ModuleHandle`] trait.
pub struct ModuleDispatcher<M: SignalingModule + Send + 'static> {
    module: M,
}

impl<M> ModuleDispatcher<M>
where
    M: SignalingModule,
{
    /// Dispatches dynamic events to the correct modules and resolves their type
    fn handle_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        event: DynEvent,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        match event {
            DynEvent::WebsocketMessage {
                participant_id: sender,
                connection_id,
                command,
            } => self.handle_ws_event(ctx, sender, connection_id, &command),
            DynEvent::LobbyWebsocketMessage {
                participant_id: sender,
                connection_id,
                command,
            } => self.handle_lobby_ws_event(ctx, sender, connection_id, &command),
            DynEvent::LoopbackEvent(result) => self.handle_loopback_event(ctx, result),
            DynEvent::InternalCommand { sender, command } => {
                self.handle_internal_command(ctx, &sender, command)
            }
        }
    }

    #[tracing::instrument(skip_all, fields(opentalk.module = %M::NAMESPACE, opentalk.event_type=Empty))]
    fn handle_broadcast_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        event: &mut DynBroadcastEvent<'_>,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        let span = Span::current();
        match event {
            DynBroadcastEvent::Connected {
                participant_id,
                connection_id,
                own_data,
                peer_events,
                peer_data,
            } => {
                span.record("opentalk.event_type", "Connected");
                let is_first_connection = ctx
                    .participants
                    .all_unfiltered
                    .get(participant_id)
                    .map(|s| s.connections.len() == 1)
                    .context("new participant not in participant set")?;

                let join_info = self.module.on_participant_joined(
                    ctx,
                    *participant_id,
                    *connection_id,
                    is_first_connection,
                )?;

                if let Some(success_info) = join_info.join_success {
                    own_data
                        .insert(&success_info)
                        .with_context(|| {
                            format!("failed to serialize JoinInfo for module '{}'", M::NAMESPACE)
                        })
                        .map_err(FatalError)?;
                }

                for (participant_id, peer_join_info) in join_info.peer_events.map {
                    peer_events
                        .entry(participant_id)
                        .or_default()
                        .insert(M::NAMESPACE, peer_join_info);
                }

                for (participant_id, data) in join_info.peer_data.map {
                    peer_data
                        .entry(participant_id)
                        .or_default()
                        .insert(M::NAMESPACE, data);
                }
            }
            DynBroadcastEvent::Disconnected {
                participant_id,
                connection_id,
            } => {
                span.record("opentalk.event_type", "Disconnected");
                self.module
                    .on_participant_disconnected(ctx, *participant_id, *connection_id)?;
            }
            DynBroadcastEvent::BreakoutStart { rooms, duration } => {
                span.record("opentalk.event_type", "BreakoutStart");
                self.module.on_breakout_start(ctx, rooms, *duration)?;
            }

            DynBroadcastEvent::SwitchRoom {
                participant_id,
                old_room,
                new_room,
                own_data,
                peer_events,
                peer_data,
            } => {
                span.record("opentalk.event_type", "SwitchRoom");
                let module_data =
                    self.module
                        .on_breakout_switch(ctx, *participant_id, *old_room, *new_room)?;

                // record data sent to other peers
                for (other, peer_join_info) in module_data.peer_events.map {
                    peer_events
                        .entry(other)
                        .or_default()
                        .insert(M::NAMESPACE, peer_join_info);
                }

                // record data about the current participant and all their connections
                for (conn_id, join_info) in module_data.switch_success {
                    if let Some(join_info) = join_info {
                        own_data
                            .entry(conn_id)
                            .or_default()
                            .insert(&join_info)
                            .with_context(|| {
                                format!(
                                    "failed to serialize JoinInfo for module '{}'",
                                    M::NAMESPACE
                                )
                            })
                            .map_err(FatalError)?;
                    }
                }

                // record data about other participants for the current participant
                for (other, data) in module_data.peer_data.map {
                    peer_data
                        .entry(other)
                        .or_default()
                        .insert(M::NAMESPACE, data);
                }
            }

            DynBroadcastEvent::BreakoutClosing => {
                span.record("opentalk.event_type", "BreakoutClosing");
                self.module.on_breakout_closing(ctx)?;
            }

            DynBroadcastEvent::BreakoutClosed => {
                span.record("opentalk.event_type", "BreakoutClosed");
                self.module.on_breakout_closed(ctx)?;
            }
            DynBroadcastEvent::Closing => {
                span.record("opentalk.event_type", "Closing");
                self.module.on_closing(ctx)?;
            }
            DynBroadcastEvent::ModerationRoleChanged { participant_id } => {
                span.record("opentalk.event_type", "ModerationRoleChanged");
                self.module
                    .on_moderation_role_change(ctx, *participant_id)?;
            }
        }
        Ok(())
    }

    /// Resolves a generic JSON message that was received by [`ModuleHandle::on_event`] to the
    /// concrete [`SignalingModule::Incoming`] type.
    #[tracing::instrument(skip_all, fields(opentalk.module = %M::NAMESPACE))]
    fn handle_ws_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        command: &RawValue,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        let payload: <M as SignalingModule>::Incoming = match serde_json::from_str(command.get()) {
            Ok(payload) => payload,
            Err(err) => {
                tracing::debug!(
                    "failed to deserialize websocket message for namespace {}: {}",
                    M::NAMESPACE,
                    err
                );

                ctx.handle_error(SignalingError::InvalidJson {
                    message: "failed to deserialize websocket message".into(),
                });

                return Ok(());
            }
        };

        if let Some(replication_event) = payload.replicate() {
            ctx.send_replica(sender, connection_id, replication_event)?;
        }

        self.module
            .on_websocket_message(ctx, sender, connection_id, payload)?;

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(opentalk.command.sender = %sender, opentalk.module = %M::NAMESPACE))]
    fn handle_lobby_ws_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        sender: ParticipantId,
        connection_id: ConnectionId,
        command: &RawValue,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        let payload: <M as SignalingModule>::Incoming = match serde_json::from_str(command.get()) {
            Ok(payload) => payload,
            Err(err) => {
                tracing::debug!(
                    "failed to deserialize websocket message for namespace {}: {}",
                    M::NAMESPACE,
                    err
                );

                ctx.handle_error(SignalingError::InvalidJson {
                    message: "failed to deserialize websocket message".into(),
                });

                return Ok(());
            }
        };

        if let Some(replication_event) = payload.replicate() {
            ctx.send_replica(sender, connection_id, replication_event)?;
        }

        self.module
            .on_websocket_message_lobby(ctx, sender, connection_id, payload)?;

        Ok(())
    }

    /// Resolves a dynamic loopback message that was received by [`ModuleHandle::on_event`] to the
    /// concrete [`SignalingModule::Loopback`] type.
    #[tracing::instrument(skip_all, fields(opentalk.module = %M::NAMESPACE))]
    fn handle_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        value: Box<dyn Any + Send + 'static>,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        let event = value.downcast().ok().with_context(|| {
            format!(
                "Failed to downcast loopback type for module in namespace {}",
                M::NAMESPACE
            )
        })?;

        self.module.on_loopback_event(ctx, *event)?;
        Ok(())
    }

    /// Resolves a dynamic internal command that was received by [`ModuleHandle::on_event`] to the
    /// concrete [`SignalingModule::IncomingInternal`] type.
    #[tracing::instrument(skip(self, ctx, command), fields(opentalk.module = %M::NAMESPACE))]
    fn handle_internal_command(
        &mut self,
        ctx: &mut ModuleContext<'_, M>,
        sender: &ModuleId,
        command: Box<dyn Any + Send + 'static>,
    ) -> Result<(), SignalingModuleError<M::Error>> {
        let command = command.downcast().ok().with_context(|| {
            format!(
                "Failed to downcast internal command type send from module '{sender}' to '{}'",
                M::NAMESPACE
            )
        })?;

        self.module.on_internal_command(ctx, *command)?;

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(opentalk.module = %M::NAMESPACE))]
    fn handle_error(
        ctx: &mut ModuleContext<'_, M>,
        err: SignalingModuleError<M::Error>,
    ) -> Result<(), FatalError> {
        match err {
            SignalingModuleError::Fatal(err) => return Err(err),
            SignalingModuleError::Internal(err) => {
                tracing::error!(
                    "module '{}' returned an internal error: {err:#?}",
                    M::NAMESPACE
                );
                ctx.handle_error(SignalingError::Internal);
            }
            SignalingModuleError::Module(err) => {
                let msg = err.into();

                match ctx.event_origin {
                    EventOrigin::Participant(participant_origin) => {
                        ctx.send_ws_message([participant_origin.id], msg)?;
                    }
                    EventOrigin::Internal => {
                        tracing::warn!(
                            "module '{}' returned a websocket error message but the event origin is internal: {msg:?} ",
                            M::NAMESPACE
                        );
                    }
                }
            }
            SignalingModuleError::NotSupported => ctx.handle_error(SignalingError::NotSupported),
        }
        Ok(())
    }
}

impl<M> ModuleHandle for ModuleDispatcher<M>
where
    M: SignalingModule,
{
    #[tracing::instrument(skip_all, level = "debug")]
    fn on_event(
        &mut self,
        ctx: &mut DynModuleContext<'_>,
        event: DynEvent,
    ) -> Result<(), FatalError> {
        let mut module_context = ctx.reborrow().into_typed_context();

        if let Err(err) = self.handle_event(&mut module_context, event) {
            Self::handle_error(&mut module_context, err)?;
        }

        Ok(())
    }

    fn on_broadcast_event(
        &mut self,
        ctx: &mut DynModuleContext<'_>,
        event: &mut DynBroadcastEvent<'_>,
    ) -> Result<(), FatalError> {
        let mut module_context: ModuleContext<'_, M> = ctx.reborrow().into_typed_context();

        if let Err(err) = self.handle_broadcast_event(&mut module_context, event) {
            return Self::handle_error(&mut module_context, err);
        }

        Ok(())
    }

    fn on_closing(&mut self, ctx: &mut DynModuleContext<'_>) {
        let mut module_context: ModuleContext<'_, M> = ctx.reborrow().into_typed_context();

        if let Err(err) = self.module.on_closing(&mut module_context) {
            tracing::error!(
                "Module '{}' returned an error while closing: {err:#?}",
                M::NAMESPACE,
            );
        }
    }
}
