// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, BTreeSet, HashMap};

use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, NoSettings, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    breakout::BreakoutRoom, connection_id::ConnectionId, room_kind::RoomKind,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_raise_hands::{
    RAISE_HANDS_MODULE_ID,
    command::RaiseHandsCommand,
    event::{RaiseHandsError, RaiseHandsEvent},
    state::{RaisedHandPeerState, RaisedHandState},
};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;

pub struct RaiseHandsModule {
    /// The raised hands state for each room. If a room is not present in this map,
    /// raised hands are not enabled for that room.
    raised_hands: HashMap<RoomKind, HashMap<ParticipantId, Timestamp>>,
}

impl SignalingModuleDescription for RaiseHandsModule {
    const MODULE_ID: ModuleId = RAISE_HANDS_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles the raising hand functionality";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for RaiseHandsModule {
    const NAMESPACE: ModuleId = RAISE_HANDS_MODULE_ID;

    type Incoming = RaiseHandsCommand;

    type Outgoing = RaiseHandsEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = RaisedHandState;

    type PeerJoinInfo = RaisedHandPeerState;

    type Error = RaiseHandsError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            raised_hands: HashMap::from_iter([(RoomKind::Main, HashMap::new())]),
        })
    }

    #[allow(unused_variables)]
    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let mut raised_hands = self.raised_hands_state(ctx.room);

        let join_success = RaisedHandState {
            raise_hands_enabled: raised_hands.is_some(),
            state: raised_hands
                .as_mut()
                .and_then(|m| m.remove(&participant_id)),
        };

        let mut peer_data = PeerDataMap::default();
        for (participant_id, peer_state) in raised_hands.unwrap_or_default() {
            peer_data.insert(participant_id, peer_state)?;
        }

        let mut peer_events = PeerDataMap::default();
        if let Some(state) = &join_success.state {
            peer_events.insert_for_all(ctx, state.clone())?;
        }

        Ok(ModuleJoinData {
            join_success: Some(join_success),
            peer_data,
            peer_events,
        })
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let was_last_connection = ctx
            .participant_state(participant_id)
            .is_none_or(|state| state.connections.is_empty());
        if was_last_connection && let Some(raised_hands) = self.raised_hands.get_mut(&ctx.room) {
            raised_hands.remove(&participant_id);
        }

        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            RaiseHandsCommand::EnableRaiseHands => self.enable_raise_hands(ctx, sender),
            RaiseHandsCommand::DisableRaiseHands => self.disable_raise_hands(ctx, sender),
            RaiseHandsCommand::RaiseHand => self.raise_hand(ctx, sender),
            RaiseHandsCommand::LowerHand => self.lower_hand(ctx, sender),
            RaiseHandsCommand::ResetRaisedHands { target } => {
                self.reset_raised_hands(ctx, sender, target)
            }
        }
    }

    fn on_breakout_start(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        _duration: Option<std::time::Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        // Enable raised hands for breakout rooms when enabled in the main room.
        if self.raised_hands.contains_key(&RoomKind::Main) {
            self.raised_hands.extend(
                rooms
                    .iter()
                    .map(|room| (RoomKind::Breakout(room.id), HashMap::new())),
            );
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
        // Reset raised hand of the participant when switching rooms
        self.raised_hands
            .entry(old_room)
            .and_modify(|raised_hands| {
                raised_hands.remove(&participant_id);
            });

        let mut raised_hands = self.raised_hands_state(new_room);
        let own_state = RaisedHandState {
            raise_hands_enabled: raised_hands.is_some(),
            state: raised_hands
                .as_mut()
                .and_then(|m| m.remove(&participant_id)),
        };
        let switch_success = ctx
            .participants
            .all_unfiltered
            .get(&participant_id)
            .ok_or(RaiseHandsError::UnknownParticipant)?
            .connections()
            .map(|con_id| (con_id, Some(own_state.clone())))
            .collect();

        let mut peer_data = PeerDataMap::default();
        for (participant_id, peer_state) in raised_hands.unwrap_or_default() {
            peer_data.insert(participant_id, peer_state)?;
        }

        let mut peer_events = PeerDataMap::default();
        if let Some(state) = own_state.state {
            peer_events.insert_for_all(ctx, state)?;
        }

        Ok(ModuleSwitchData {
            switch_success,
            peer_data,
            peer_events,
        })
    }

    fn on_breakout_closed(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.raised_hands.retain(|room, _| *room == RoomKind::Main);
        Ok(())
    }
}

impl RaiseHandsModule {
    fn enable_raise_hands(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        if !ctx.is_moderator(sender) {
            return Err(RaiseHandsError::InsufficientPermissions.into());
        }

        self.raised_hands.entry(ctx.room).or_default();

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            RaiseHandsEvent::RaiseHandsEnabled { issued_by: sender },
        )?;

        Ok(())
    }

    fn disable_raise_hands(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        if !ctx.is_moderator(sender) {
            return Err(RaiseHandsError::InsufficientPermissions.into());
        }

        self.raised_hands.remove(&ctx.room);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            RaiseHandsEvent::RaiseHandsDisabled { issued_by: sender },
        )?;

        Ok(())
    }

    fn raise_hand(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        let raised_hands = self
            .raised_hands
            .get_mut(&ctx.room)
            .ok_or(RaiseHandsError::RaiseHandsDisabled)?;

        raised_hands
            .entry(participant_id)
            .and_modify(|timestamp| *timestamp = ctx.timestamp)
            .or_insert(ctx.timestamp);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            RaiseHandsEvent::HandRaised {
                participant: participant_id,
            },
        )?;

        Ok(())
    }

    fn lower_hand(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        let raised_hands = self
            .raised_hands
            .get_mut(&ctx.room)
            .ok_or(RaiseHandsError::RaiseHandsDisabled)?;

        raised_hands.remove(&participant_id);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            RaiseHandsEvent::HandLowered {
                participant: participant_id,
            },
        )?;

        Ok(())
    }

    fn reset_raised_hands(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        targets: Option<BTreeSet<ParticipantId>>,
    ) -> Result<(), SignalingModuleError<RaiseHandsError>> {
        if !ctx.is_moderator(sender) {
            return Err(RaiseHandsError::InsufficientPermissions.into());
        }

        let raised_hands = self
            .raised_hands
            .get_mut(&ctx.room)
            .ok_or(RaiseHandsError::RaiseHandsDisabled)?;

        let mut participants: BTreeSet<ParticipantId>;
        if let Some(targets) = targets {
            participants = BTreeSet::new();
            for participant_id in targets {
                if raised_hands.remove(&participant_id).is_some() {
                    participants.insert(participant_id);
                }
            }
        } else {
            participants = raised_hands.keys().copied().collect();
            raised_hands.clear();
        }

        if !participants.is_empty() {
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                RaiseHandsEvent::RaisedHandResetByModerator {
                    issued_by: sender,
                    participants,
                },
            )?;
        }

        Ok(())
    }

    fn raised_hands_state(
        &self,
        room: RoomKind,
    ) -> Option<BTreeMap<ParticipantId, RaisedHandPeerState>> {
        self.raised_hands.get(&room).map(|raised_hands| {
            raised_hands
                .iter()
                .map(|(&participant_id, &raised_at)| {
                    (participant_id, RaisedHandPeerState { raised_at })
                })
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::RaiseHandsModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&RaiseHandsModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[raise_hands]");
    }
}
