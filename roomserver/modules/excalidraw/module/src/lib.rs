// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! This module implements an excalidraw collaboration backend.
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
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_excalidraw::{
    EXCALIDRAW_MODULE_ID, ExcalidrawCommand, ExcalidrawError, ExcalidrawEvent,
    edit_restrictions::EditRestrictions,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

use crate::state::ExcalidrawState;

pub mod state;

pub struct ExcalidrawModule {
    state: HashMap<RoomKind, ExcalidrawState>,
}

impl SignalingModuleDescription for ExcalidrawModule {
    const MODULE_ID: ModuleId = EXCALIDRAW_MODULE_ID;
    const DESCRIPTION: &'static str =
        "Handles excalidraw whiteboard integration. Excalidraw is a collaborative drawing board.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for ExcalidrawModule {
    const NAMESPACE: ModuleId = EXCALIDRAW_MODULE_ID;

    type Incoming = ExcalidrawCommand;

    type Outgoing = ExcalidrawEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = ExcalidrawState;

    type PeerJoinInfo = ();

    type Error = ExcalidrawError;

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
            ExcalidrawCommand::Start {
                initial_scene,
                edit_restrictions,
            } => self.start(ctx, sender, initial_scene, edit_restrictions),
            ExcalidrawCommand::Stop => self.stop(ctx, sender),
            ExcalidrawCommand::BroadcastVolatile { data } => {
                self.broadcast_volatile(ctx, sender, data)
            }
            ExcalidrawCommand::Broadcast { data } => self.broadcast(ctx, sender, data),
            ExcalidrawCommand::Follow { participant_id } => {
                self.follow(ctx, sender, participant_id)
            }
            ExcalidrawCommand::Unfollow { participant_id } => {
                self.unfollow(ctx, sender, participant_id)
            }
            ExcalidrawCommand::StoreScene { scene } => self.store_scene(ctx, sender, scene),
            ExcalidrawCommand::EnableEditRestrictions {
                unrestricted_participants,
            } => self.enable_edit_restrictions(ctx, sender, unrestricted_participants),
            ExcalidrawCommand::DisableEditRestrictions => {
                self.disable_edit_restrictions(ctx, sender)
            }
        }
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

        let switch_success: BTreeMap<ConnectionId, Option<ExcalidrawState>> = ctx
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

impl ExcalidrawModule {
    #[tracing::instrument(skip(self, ctx, initial_scene), level = "debug")]
    fn start(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        initial_scene: serde_json::Value,
        edit_restrictions: EditRestrictions,
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        if !ctx.is_moderator(sender) {
            return Err(ExcalidrawError::InsufficientPermissions.into());
        }

        let Entry::Vacant(vacant_entry) = self.state.entry(ctx.room) else {
            return Err(ExcalidrawError::AlreadyStarted.into());
        };

        vacant_entry.insert(ExcalidrawState {
            scene: initial_scene.clone(),
            edit_restrictions: edit_restrictions.clone(),
        });

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ExcalidrawEvent::Started {
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
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        if !ctx.is_moderator(sender) {
            return Err(ExcalidrawError::InsufficientPermissions.into());
        }

        self.state.remove(&ctx.room);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ExcalidrawEvent::Stopped,
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
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        if !self.state.contains_key(&ctx.room) {
            return Err(ExcalidrawError::NotStarted.into());
        }

        // The volatile broadcast cannot modify the excalidraw state, so all participants are
        // allowed to use it.
        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ExcalidrawEvent::VolatileBroadcast { sender, data },
        )?;

        Ok(())
    }

    /// Broadcasts a non-volatile update to all participants in the room. This update can modify the
    /// excalidraw state and is restricted to participants that are allowed to edit excalidraw.
    ///
    /// Implements the `server-broadcast` commands from excalidraw-room.
    #[tracing::instrument(skip(self, ctx, data), level = "debug")]
    fn broadcast(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        data: serde_json::Value,
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        let Some(restrictions) = self
            .state
            .get(&ctx.room)
            .map(|state| &state.edit_restrictions)
        else {
            return Err(ExcalidrawError::NotStarted.into());
        };

        // The broadcast can modify the excalidraw state, so only participants that are allowed to
        // edit are allowed to use it.
        if !Self::is_allowed_to_edit(ctx, sender, restrictions) {
            return Err(ExcalidrawError::InsufficientPermissions.into());
        }

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ExcalidrawEvent::Broadcast { sender, data },
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx, scene), level = "debug")]
    fn store_scene(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        scene: serde_json::Value,
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        let Some(state) = self.state.get_mut(&ctx.room) else {
            return Err(ExcalidrawError::NotStarted.into());
        };

        if !Self::is_allowed_to_edit(ctx, sender, &state.edit_restrictions) {
            return Err(ExcalidrawError::InsufficientPermissions.into());
        }

        state.scene = scene.clone();

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ExcalidrawEvent::SceneStored { scene },
        )?;

        Ok(())
    }

    /// One participant starts following another participant, i.e. their excalidraw view will be
    /// synced to the followed participant's view.
    ///
    /// Implements the `user-follow` command with the `FOLLOW` action from excalidraw-room.
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn follow(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        if !self.state.contains_key(&ctx.room) {
            return Err(ExcalidrawError::NotStarted.into());
        }

        if !ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .contains(&target)
        {
            return Err(ExcalidrawError::UnknownParticipant.into());
        }

        // Notify the target participant that they are being followed
        ctx.send_ws_message(
            [target],
            ExcalidrawEvent::FollowerGained {
                participant_id: sender,
            },
        )?;

        // Notify the sender that following the target participant was successful
        ctx.send_ws_message(
            [sender],
            ExcalidrawEvent::Followed {
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
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        if !self.state.contains_key(&ctx.room) {
            return Err(ExcalidrawError::NotStarted.into());
        }

        if !ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .contains(&target)
        {
            return Err(ExcalidrawError::UnknownParticipant.into());
        }

        // We do not handle the case that the sender wasn't following the target participant in the
        // first place. In this case the target participant still receives an unfollow event, but
        // the client should be able to handle this. It is not worth tracking the following state
        // server side.
        // Notify the target participant that they are being unfollowed
        ctx.send_ws_message(
            [target],
            ExcalidrawEvent::FollowerLost {
                participant_id: sender,
            },
        )?;

        // Notify the sender that unfollowing the target participant was successful
        ctx.send_ws_message(
            [sender],
            ExcalidrawEvent::Unfollowed {
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
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        if !ctx.is_moderator(sender) {
            return Err(ExcalidrawError::InsufficientPermissions.into());
        }

        let Some(state) = self.state.get_mut(&ctx.room) else {
            return Err(ExcalidrawError::NotStarted.into());
        };

        state.edit_restrictions = EditRestrictions::Enabled {
            unrestricted_participants: unrestricted_participants.clone(),
        };

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ExcalidrawEvent::EditRestrictionsEnabled {
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
    ) -> Result<(), SignalingModuleError<ExcalidrawError>> {
        if !ctx.is_moderator(sender) {
            return Err(ExcalidrawError::InsufficientPermissions.into());
        }

        let Some(state) = self.state.get_mut(&ctx.room) else {
            return Err(ExcalidrawError::NotStarted.into());
        };

        state.edit_restrictions = EditRestrictions::Disabled;

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ExcalidrawEvent::EditRestrictionsDisabled,
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

    use crate::ExcalidrawModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&ExcalidrawModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[excalidraw]");
    }
}
