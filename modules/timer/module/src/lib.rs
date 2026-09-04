// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeSet, HashMap},
    time::Duration,
};

use anyhow::Context;
use opentalk_roomserver_signaling::{
    module_context::{ChannelDroppedError, ModuleContext},
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, NoSettings, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    breakout::BreakoutRoom, connection_id::ConnectionId, room_kind::RoomKind,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_timer::{
    Kind, StopKind, TIMER_MODULE_ID, TimerCommand, TimerConfig, TimerError, command,
    event::{Stopped, TimerEvent},
    peer_state::TimerPeerState,
    state::TimerState,
};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;

use crate::timer::Timer;

mod timer;

#[derive(Debug)]
pub enum TimerLoopback {
    Stopped(Stopped),
    ChannelDropped,
}

impl From<ChannelDroppedError> for TimerLoopback {
    fn from(_: ChannelDroppedError) -> Self {
        Self::ChannelDropped
    }
}

#[derive(Debug)]
pub struct TimerModule {
    timers: HashMap<RoomKind, Option<Timer>>,
    ready_participants: HashMap<RoomKind, BTreeSet<ParticipantId>>,
}

struct ReadyCheckData {
    ready_status: bool,
    peer_data: PeerDataMap<TimerModule>,
    peer_events: PeerDataMap<TimerModule>,
}

impl SignalingModuleDescription for TimerModule {
    const MODULE_ID: ModuleId = TIMER_MODULE_ID;
    const DESCRIPTION: &'static str =
        "Handles timer functionality including the coffee-break timer.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for TimerModule {
    const NAMESPACE: ModuleId = TIMER_MODULE_ID;

    type Incoming = TimerCommand;

    type Outgoing = TimerEvent;

    type Internal = NoOp;

    type Loopback = TimerLoopback;

    type JoinInfo = TimerState;

    type PeerJoinInfo = TimerPeerState;

    type Error = TimerError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            timers: HashMap::from([(RoomKind::Main, None)]),
            ready_participants: HashMap::from([(RoomKind::Main, BTreeSet::new())]),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let timer = self
            .timers
            .get(&ctx.room)
            .with_context(|| format!("Room '{:?}' does not exist in timers", ctx.room))?;

        // Do not add JoinSuccess or PeerJoinInfo when there is no running timer
        let Some(timer) = timer else {
            return Ok(ModuleJoinData {
                join_success: None,
                peer_events: PeerDataMap::default(),
                peer_data: PeerDataMap::default(),
            });
        };

        if timer.config.ready_check_enabled {
            let ready_check_data = self.collect_ready_check_data(ctx, participant_id, ctx.room)?;

            Ok(ModuleJoinData {
                join_success: Some(TimerState {
                    config: timer.config.clone(),
                    ready_status: Some(ready_check_data.ready_status),
                }),
                peer_data: ready_check_data.peer_data,
                peer_events: ready_check_data.peer_events,
            })
        } else {
            Ok(ModuleJoinData {
                join_success: Some(TimerState {
                    config: timer.config.clone(),
                    ready_status: None,
                }),
                peer_data: PeerDataMap::default(),
                peer_events: PeerDataMap::default(),
            })
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

    fn on_breakout_start(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        _duration: Option<Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        for room in rooms {
            let room = RoomKind::Breakout(room.id);
            self.timers.insert(room, None);
            self.ready_participants.insert(room, BTreeSet::new());
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
        let timer = self
            .timers
            .get(&new_room)
            .with_context(|| format!("Room '{new_room:?}' does not exist in timers"))?;

        // Timer is disabled, send empty JoinInfo
        let Some(timer) = timer else {
            return Ok(ModuleSwitchData::<Self>::new());
        };

        let (ready_status, peer_data, peer_events) = if timer.config.ready_check_enabled {
            let ready_check_data = self.collect_ready_check_data(ctx, participant_id, new_room)?;
            (
                Some(ready_check_data.ready_status),
                ready_check_data.peer_data,
                ready_check_data.peer_events,
            )
        } else {
            (None, PeerDataMap::default(), PeerDataMap::default())
        };
        let timer_state = Some(TimerState {
            config: timer.config.clone(),
            ready_status,
        });

        let connections = ctx
            .participant_state(participant_id)
            .with_context(|| format!("Participant '{participant_id}' does not have state"))?
            .connections();
        let switch_success = connections.map(|con| (con, timer_state.clone())).collect();

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
        self.ready_participants
            .retain(|room, _| *room == RoomKind::Main);
        self.timers.retain(|room, _| *room == RoomKind::Main);
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
            TimerCommand::Start {
                kind,
                style,
                title,
                enable_ready_check,
            } => self.start_timer(ctx, sender, kind, style, title, enable_ready_check)?,
            TimerCommand::Stop { reason } => self.stop_timer(ctx, sender, reason)?,
            TimerCommand::UpdateReadyStatus { status } => {
                self.update_ready_status(ctx, sender, status)?;
            }
        }
        Ok(())
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.remove_timer(ctx.room)?;

        match event {
            TimerLoopback::Stopped(stopped) => {
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).ids(),
                    TimerEvent::Stopped(stopped),
                )?;
            }
            TimerLoopback::ChannelDropped => {
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).ids(),
                    TimerEvent::Error(TimerError::Internal),
                )?;
            }
        }
        Ok(())
    }
}

impl TimerModule {
    fn collect_ready_check_data(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        room: RoomKind,
    ) -> Result<ReadyCheckData, SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        let others = ctx
            .participants
            .in_room(room)
            .connected()
            .ids()
            .filter(|&other| other != participant_id);
        let ready_participants = self.ready_participants.get(&room).with_context(|| {
            format!("Room '{room:?}' does not exist in participant ready state")
        })?;

        // Joining participants might already be ready when they were in the room before.
        let ready_status = ready_participants.contains(&participant_id);

        // Collect ready state of all other participants for the switched participant.
        let mut peer_data = PeerDataMap::default();
        for p in others {
            peer_data.insert(
                p,
                TimerPeerState {
                    ready_status: ready_participants.contains(&p),
                },
            )?;
        }

        // Append ready state when the running timer has ready check enabled.
        let mut peer_events = PeerDataMap::default();
        peer_events.insert_for_all(ctx, TimerPeerState { ready_status })?;

        Ok(ReadyCheckData {
            ready_status,
            peer_data,
            peer_events,
        })
    }

    fn start_timer(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        kind: command::Kind,
        style: Option<String>,
        title: Option<String>,
        ready_check_enabled: bool,
    ) -> Result<(), SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        if !ctx.is_moderator(sender) {
            return Err(TimerError::InsufficientPermissions.into());
        }

        let timer = self
            .timers
            .get_mut(&ctx.room)
            .with_context(|| format!("Room '{:?}' does not exist in timers", ctx.room))?;
        if timer.is_some() {
            return Err(TimerError::TimerAlreadyRunning.into());
        }

        let started_at = ctx.timestamp;
        let mut tx_cancel = None;
        let kind = match kind {
            command::Kind::Stopwatch => Kind::Stopwatch,
            command::Kind::Countdown { duration } => {
                let ends_at = started_at
                    .checked_add_signed(chrono::Duration::seconds(duration.into()))
                    .ok_or(TimerError::InvalidDuration)?;

                let tx = ctx.loopback_after(Duration::from_secs(duration as u64), || {
                    TimerLoopback::Stopped(Stopped {
                        kind: StopKind::Expired,
                        reason: None,
                    })
                });
                tx_cancel = Some(tx);

                Kind::Countdown {
                    ends_at: Timestamp::from(ends_at),
                }
            }
        };

        *timer = Some(Timer {
            config: TimerConfig {
                started_at,
                kind,
                style: style.clone(),
                title: title.clone(),
                ready_check_enabled,
            },
            tx_cancel,
        });

        ctx.send_ws_message(
            ctx.participants.filter().room(ctx.room).ids(),
            TimerEvent::Started {
                config: TimerConfig {
                    started_at,
                    kind,
                    style,
                    title,
                    ready_check_enabled,
                },
            },
        )?;
        Ok(())
    }

    fn stop_timer(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        reason: Option<String>,
    ) -> Result<(), SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        if !ctx.is_moderator(sender) {
            return Err(TimerError::InsufficientPermissions.into());
        }

        if let Some(mut timer) = self.remove_timer(ctx.room)? {
            let stopped = Stopped {
                kind: StopKind::ByModerator(sender),
                reason,
            };
            if let Some(tx) = timer.tx_cancel.take() {
                if tx.send(TimerLoopback::Stopped(stopped)).is_err() {
                    tracing::debug!("Timer cancel sender has been dropped");
                }
            } else {
                // If there is no cancel sender, this means the timer does not use a
                // loopback task (e.g. stopwatch). In this case we can simply notify
                // the participants that the timer was cancelled.
                ctx.send_ws_message(
                    ctx.participants.filter().room(ctx.room).ids(),
                    TimerEvent::Stopped(stopped),
                )?;
            }
        }

        Ok(())
    }

    /// Removes the timer and the associated ready state
    fn remove_timer(
        &mut self,
        room: RoomKind,
    ) -> Result<Option<Timer>, SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        let timer = self
            .timers
            .get_mut(&room)
            .with_context(|| format!("Room '{room:?}' does not exist in timers"))?;
        let ready_states = self
            .ready_participants
            .get_mut(&room)
            .with_context(|| format!("Room '{room:?}' does not exist in ready states",))?;
        if let Some(timer) = timer.take() {
            ready_states.clear();
            return Ok(Some(timer));
        }
        Ok(None)
    }

    fn update_ready_status(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        ready: bool,
    ) -> Result<(), SignalingModuleError<<TimerModule as SignalingModule>::Error>> {
        let timer = self
            .timers
            .get_mut(&ctx.room)
            .with_context(|| format!("Room '{:?}' does not exist in timers", ctx.room))?;

        let Some(timer) = timer else {
            return Err(TimerError::TimerNotRunning)?;
        };

        if !timer.config.ready_check_enabled {
            Err(TimerError::ReadyCheckNotEnabled)?;
        }

        let ready_participants = self
            .ready_participants
            .get_mut(&ctx.room)
            .with_context(|| {
                format!(
                    "Room '{:?}' does not exist in participant ready state",
                    ctx.room
                )
            })?;

        let changed = if ready {
            ready_participants.insert(sender)
        } else {
            ready_participants.remove(&sender)
        };
        if changed {
            ctx.send_ws_message(
                ctx.participants.filter().room(ctx.room).ids(),
                TimerEvent::UpdatedReadyStatus {
                    participant_id: sender,
                    status: ready,
                },
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::TimerModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&TimerModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[timer]");
    }
}
