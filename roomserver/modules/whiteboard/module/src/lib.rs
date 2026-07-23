// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! This module implements an excalidraw based whiteboard backend.
//! It is based on [excalidraw-room](https://github.com/excalidraw/excalidraw-room).

use std::collections::{BTreeMap, HashMap, HashSet, hash_map::Entry};

use anyhow::anyhow;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, NoSettings, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    breakout::BreakoutRoom,
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_whiteboard::{
    WHITEBOARD_MODULE_ID, WhiteboardCommand, WhiteboardError, WhiteboardEvent,
    edit_restrictions::EditRestrictions,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

use crate::state::WhiteboardState;

pub mod state;

pub struct WhiteboardModule {
    state: HashMap<RoomKind, WhiteboardState>,
}

impl SignalingModuleDescription for WhiteboardModule {
    const MODULE_ID: ModuleId = WHITEBOARD_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles excalidraw based whiteboard integration. Excalidraw is a collaborative drawing board.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for WhiteboardModule {
    const NAMESPACE: ModuleId = WHITEBOARD_MODULE_ID;

    type Incoming = WhiteboardCommand;

    type Outgoing = WhiteboardEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = WhiteboardState;

    type PeerJoinInfo = ();

    type Error = WhiteboardError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            state: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let join_success = self.state.get(&ctx.room).cloned();
        Ok(ModuleJoinData {
            join_success,
            peer_events: PeerDataMap::default(),
            peer_data: PeerDataMap::default(),
        })
    }

    fn on_participant_disconnected(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
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
            WhiteboardCommand::Start {
                initial_scene,
                edit_restrictions,
            } => self.start(ctx, sender, initial_scene, edit_restrictions),
            WhiteboardCommand::Stop => self.stop(ctx, sender),
            WhiteboardCommand::BroadcastVolatile { data } => {
                self.broadcast_volatile(ctx, sender, data)
            }
            WhiteboardCommand::Broadcast { data } => self.broadcast(ctx, sender, data),
            WhiteboardCommand::Follow { participant_id } => {
                self.follow(ctx, sender, participant_id)
            }
            WhiteboardCommand::Unfollow { participant_id } => {
                self.unfollow(ctx, sender, participant_id)
            }
            WhiteboardCommand::StoreScene { scene } => self.store_scene(ctx, sender, scene),
            WhiteboardCommand::EnableEditRestrictions {
                unrestricted_participants,
            } => self.enable_edit_restrictions(ctx, sender, unrestricted_participants),
            WhiteboardCommand::DisableEditRestrictions => {
                self.disable_edit_restrictions(ctx, sender)
            }
        }
    }

    fn on_breakout_start(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        _duration: Option<std::time::Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        // Autostart the whiteboard with a fresh board in the breakout rooms when it is already
        // running in the main room.
        if self.state.contains_key(&RoomKind::Main) {
            self.state.extend(rooms.iter().map(|room| {
                (
                    RoomKind::Breakout(room.id),
                    WhiteboardState {
                        // Initializing whiteboard with empty scene; required by the frontend to be
                        // set correctly.
                        scene: serde_json::json!({"elements": []}),
                        edit_restrictions: EditRestrictions::default(),
                    },
                )
            }));
        }
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
            return Ok(ModuleSwitchData::default());
        };

        let switch_success: BTreeMap<ConnectionId, Option<WhiteboardState>> = ctx
            .participant_state(participant_id)
            .ok_or(FatalError(anyhow!(
                "Participant {participant_id:?} switched without participant state"
            )))?
            .connections()
            .map(|id| (id, Some(state.clone())))
            .collect();

        Ok(ModuleSwitchData {
            switch_success,
            peer_events: PeerDataMap::default(),
            peer_data: PeerDataMap::default(),
        })
    }

    fn on_breakout_closed(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.state.retain(|room, _| *room == RoomKind::Main);

        Ok(())
    }
}

impl WhiteboardModule {
    #[tracing::instrument(skip(self, ctx, initial_scene), level = "debug")]
    fn start(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        initial_scene: serde_json::Value,
        edit_restrictions: EditRestrictions,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !ctx.is_moderator(sender) {
            return Err(WhiteboardError::InsufficientPermissions.into());
        }

        let Entry::Vacant(vacant_entry) = self.state.entry(ctx.room) else {
            return Err(WhiteboardError::AlreadyStarted.into());
        };

        vacant_entry.insert(WhiteboardState {
            scene: initial_scene.clone(),
            edit_restrictions: edit_restrictions.clone(),
        });

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            WhiteboardEvent::Started {
                initial_scene,
                edit_restrictions,
            },
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn stop(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !ctx.is_moderator(sender) {
            return Err(WhiteboardError::InsufficientPermissions.into());
        }

        self.state.remove(&ctx.room);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            WhiteboardEvent::Stopped,
        )?;

        Ok(())
    }
    /// Broadcasts a volatile update to all participants in the room
    ///
    /// Implements the `server-volatile-broadcast` commands from excalidraw-room.
    #[tracing::instrument(skip(self, ctx, data), level = "debug")]
    fn broadcast_volatile(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        data: serde_json::Value,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !self.state.contains_key(&ctx.room) {
            return Err(WhiteboardError::NotStarted.into());
        }

        // The volatile broadcast cannot modify the whiteboard state, so all participants are
        // allowed to use it.
        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            WhiteboardEvent::VolatileBroadcast { sender, data },
        )?;

        Ok(())
    }

    /// Broadcasts a non-volatile update to all participants in the room. This update can modify the
    /// whiteboard state and is restricted to participants that are allowed to edit whiteboard.
    ///
    /// Implements the `server-broadcast` commands from excalidraw-room.
    #[tracing::instrument(skip(self, ctx, data), level = "debug")]
    fn broadcast(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        data: serde_json::Value,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        let Some(restrictions) = self
            .state
            .get(&ctx.room)
            .map(|state| &state.edit_restrictions)
        else {
            return Err(WhiteboardError::NotStarted.into());
        };

        // The broadcast can modify the whiteboard state, so only participants that are allowed to
        // edit are allowed to use it.
        if !Self::is_allowed_to_edit(ctx, sender, restrictions) {
            return Err(WhiteboardError::InsufficientPermissions.into());
        }

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            WhiteboardEvent::Broadcast { sender, data },
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx, scene), level = "debug")]
    fn store_scene(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        scene: serde_json::Value,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        let Some(state) = self.state.get_mut(&ctx.room) else {
            return Err(WhiteboardError::NotStarted.into());
        };

        if !Self::is_allowed_to_edit(ctx, sender, &state.edit_restrictions) {
            return Err(WhiteboardError::InsufficientPermissions.into());
        }

        state.scene = scene.clone();

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            WhiteboardEvent::SceneStored { scene },
        )?;

        Ok(())
    }

    /// One participant starts following another participant, i.e. their whiteboard view will be
    /// synced to the followed participant's view.
    ///
    /// Implements the `user-follow` command with the `FOLLOW` action from excalidraw-room.
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn follow(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !self.state.contains_key(&ctx.room) {
            return Err(WhiteboardError::NotStarted.into());
        }

        if !ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .contains(&target)
        {
            return Err(WhiteboardError::UnknownParticipant.into());
        }

        // Notify the target participant that they are being followed
        ctx.send_ws_message(
            [target],
            WhiteboardEvent::FollowerGained {
                participant_id: sender,
            },
        )?;

        // Notify the sender that following the target participant was successful
        ctx.send_ws_message(
            [sender],
            WhiteboardEvent::Followed {
                participant_id: target,
            },
        )?;

        Ok(())
    }

    /// One participant stops following another participant.
    ///
    /// Implements the `user-follow` command with the `UNFOLLOW` action from excalidraw-room.
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn unfollow(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !self.state.contains_key(&ctx.room) {
            return Err(WhiteboardError::NotStarted.into());
        }

        if !ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .contains(&target)
        {
            return Err(WhiteboardError::UnknownParticipant.into());
        }

        // We do not handle the case that the sender wasn't following the target participant in the
        // first place. In this case the target participant still receives an unfollow event, but
        // the client should be able to handle this. It is not worth tracking the following state
        // server side.
        // Notify the target participant that they are being unfollowed
        ctx.send_ws_message(
            [target],
            WhiteboardEvent::FollowerLost {
                participant_id: sender,
            },
        )?;

        // Notify the sender that unfollowing the target participant was successful
        ctx.send_ws_message(
            [sender],
            WhiteboardEvent::Unfollowed {
                participant_id: target,
            },
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn enable_edit_restrictions(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        sender: ParticipantId,
        unrestricted_participants: HashSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !ctx.is_moderator(sender) {
            return Err(WhiteboardError::InsufficientPermissions.into());
        }

        let Some(state) = self.state.get_mut(&ctx.room) else {
            return Err(WhiteboardError::NotStarted.into());
        };

        state.edit_restrictions = EditRestrictions::Enabled {
            unrestricted_participants: unrestricted_participants.clone(),
        };

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            WhiteboardEvent::EditRestrictionsEnabled {
                unrestricted_participants,
            },
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn disable_edit_restrictions(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<WhiteboardError>> {
        if !ctx.is_moderator(sender) {
            return Err(WhiteboardError::InsufficientPermissions.into());
        }

        let Some(state) = self.state.get_mut(&ctx.room) else {
            return Err(WhiteboardError::NotStarted.into());
        };

        state.edit_restrictions = EditRestrictions::Disabled;

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            WhiteboardEvent::EditRestrictionsDisabled,
        )?;

        Ok(())
    }

    fn is_allowed_to_edit(
        ctx: &ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        restrictions: &EditRestrictions,
    ) -> bool {
        ctx.is_moderator(participant_id)
            || restrictions == &EditRestrictions::Disabled
            || matches!(
                restrictions,
                EditRestrictions::Enabled {
                    unrestricted_participants
                } if unrestricted_participants.contains(&participant_id)
            )
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

        assert_snapshot!(produced, @"[whiteboard]");
    }
}
