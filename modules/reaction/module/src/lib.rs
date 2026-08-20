// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{HashMap, HashSet};

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
use opentalk_roomserver_types_reaction::{
    REACTION_MODULE_ID, Reaction, ReactionCommand, ReactionEvent, ReactionState,
    event::ReactionError, state::ReactionRestrictions,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;

pub struct ReactionModule {
    restrictions: HashMap<RoomKind, ReactionRestrictions>,
}

impl SignalingModuleDescription for ReactionModule {
    const MODULE_ID: ModuleId = REACTION_MODULE_ID;

    const DESCRIPTION: &'static str = "Handles emoji reactions";

    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for ReactionModule {
    const NAMESPACE: ModuleId = REACTION_MODULE_ID;

    type Incoming = ReactionCommand;

    type Outgoing = ReactionEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = ReactionState;

    type PeerJoinInfo = ();

    type Error = ReactionError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            restrictions: Default::default(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let restrictions = self
            .restrictions
            .get(&ctx.room)
            .cloned()
            .unwrap_or_default();
        Ok(ModuleJoinData {
            join_success: Some(ReactionState { restrictions }),
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
            ReactionCommand::React { reaction } => self.react(ctx, sender, reaction),
            ReactionCommand::EnableRestrictions {
                unrestricted_participants,
            } => self.enable_restrictions(ctx, sender, unrestricted_participants),
            ReactionCommand::DisableRestrictions => self.disable_restrictions(ctx, sender),
        }
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        let restrictions = self.restrictions.get(&new_room);
        let switch_success = ctx
            .participant_state(participant_id)
            .ok_or(FatalError(anyhow!(
                "Participant {participant_id:?} switched without a participant state"
            )))?
            .connections()
            .map(|id| {
                let state = ReactionState {
                    restrictions: restrictions.cloned().unwrap_or_default(),
                };

                (id, Some(state))
            })
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
        self.restrictions.retain(|room, _| *room == RoomKind::Main);

        Ok(())
    }
}

impl ReactionModule {
    fn react(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        reaction: Reaction,
    ) -> Result<(), SignalingModuleError<ReactionError>> {
        if let Some(ReactionRestrictions::Enabled {
            unrestricted_participants,
        }) = self.restrictions.get(&ctx.room)
            && !unrestricted_participants.contains(&sender)
        {
            return Err(ReactionError::Restricted.into());
        }

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ReactionEvent::Reacted {
                participant_id: sender,
                reaction,
            },
        )?;

        Ok(())
    }

    fn enable_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        unrestricted_participants: HashSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<ReactionError>> {
        if !ctx.is_moderator(sender) {
            return Err(ReactionError::InsufficientPermissions.into());
        }

        self.restrictions.insert(
            ctx.room,
            ReactionRestrictions::Enabled {
                unrestricted_participants: unrestricted_participants.clone(),
            },
        );

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ReactionEvent::RestrictionsEnabled {
                unrestricted_participants,
            },
        )?;

        Ok(())
    }

    fn disable_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<ReactionError>> {
        if !ctx.is_moderator(sender) {
            return Err(ReactionError::InsufficientPermissions.into());
        }

        self.restrictions.remove(&ctx.room);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            ReactionEvent::RestrictionsDisabled,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::ReactionModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&ReactionModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[reaction]");
    }
}
