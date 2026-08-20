// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # Auto-Moderation Module
//!
//! ## Functionality
//!
//! On room startup the automod is disabled.
//!
//! Selecting the options for the automod is managed by the frontend and this module does not
//! provide templates or anything else
//!
//! Following `selection_strategies` are defined:
//!
//! - `None`: No automatic reselection happens after the current speaker yields. The next one must
//!   always be selected by the moderator. The moderator may choose a participant directly or let
//!   the roomserver choose one randomly. For that the roomserver holds a `allow_list` which is a
//!   set of participants which are able to be randomly selected. Furthermore the roomserver will
//!   hold a list of start/stop speaker events. That list can optionally be used to avoid double
//!   selections when randomly choosing a participant.
//!
//! - `Playlist`: The playlist-strategy requires a playlist of participants. This list will be
//!   stored ordered inside the roomserver. Whenever a speaker yields the roomserver will
//!   automatically choose the next participant in the list to be the next speaker.
//!
//!   A moderator may choose to skip over a speaker. That can be done by selecting the next one or
//!   let the roomserver choose someone random from the playlist.
//!   The playlist can, while the automod is active, be edited.
//!
//! - `Random`: This strategy behaves like `None` but will always choose the next speaker randomly
//!   from the `allow_list` as soon as the current speaker yields.
//!
//! - `Nomination`: This strategy behaves like `None` but requires the current speaker to nominate
//!   the next participant to be speaker. The nominated participant MUST be inside the `allow_list`
//!   and if double selection is not enabled the roomserver will check if the nominated participant
//!   already was a speaker.
//!
//! ### Lifecycle
//!
//! As soon as a moderator starts the automod, the automod-module will set the config in memory and
//! then send a start event to all participants.
//!
//! The selection of the first speaker must be done by the frontend then, depending of the
//! `selection_strategy`, will the automod continue running until finished or stopped.
//!
//! Once the active speaker yields or their time runs out, the automod module is responsible to
//! select the next speaker (if the `selection_strategy` requires it). This behavior MUST only
//! be executed after ensuring that this participant is in fact still the speaker.
//!
//! If the participant leaves while being speaker, the automod-module must execute the same
//! behavior as if the participants simply yielded without selecting the next one (which would be
//! required for the `nominate` `selection_strategy`. A moderator has to intervene in this
//! situation).
//!
//! Moderators will always be able to execute a re-selection of the current speaker regardless of
//! the `selection_strategy`.

use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    time::Duration,
};

use anyhow::Context as _;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_signaling::{
    module_context::{ChannelDroppedError, ModuleContext},
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, NoSettings, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_automod::{
    AUTOMOD_MODULE_ID,
    command::{AutomodCommand, Select},
    config::{FrontendConfig, Parameter, SelectionStrategy},
    event::{AutomodError, AutomodEvent, StoppedReason},
    state::AutomodState,
};
use opentalk_roomserver_types_livekit::{LiveKitInternal, ParticipantsMuted};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::ParticipantId;
use tokio::sync::oneshot;

use crate::{
    session::Session,
    speaker_selection::{SpeakerSelectionOutput, SpeakerUpdate},
};

pub(crate) mod history_entry;
mod session;
mod speaker_selection;

/// Indicates that the time limit for a speaker has been reached
pub struct SpeakerTimeLimitReached {
    /// The participant that has reached the time limit
    pub speaker: Option<ParticipantId>,
}

pub struct AutomodModule {
    /// The currently active automod sessions, indexed by room
    sessions: HashMap<RoomKind, Session>,
}

pub enum AutomodLoopback {
    SpeakerTimeLimitReached { speaker: Option<ParticipantId> },
    ParticipantsMuted(ParticipantsMuted),
    ChannelDropped,
}

impl From<ChannelDroppedError> for AutomodLoopback {
    fn from(_: ChannelDroppedError) -> Self {
        AutomodLoopback::ChannelDropped
    }
}

impl SignalingModuleDescription for AutomodModule {
    const MODULE_ID: ModuleId = AUTOMOD_MODULE_ID;
    const DESCRIPTION: &'static str =
        "Handles auto-moderation functionality such as the talking stick";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for AutomodModule {
    const NAMESPACE: ModuleId = AUTOMOD_MODULE_ID;

    type Incoming = AutomodCommand;

    type Outgoing = AutomodEvent;

    type Internal = NoOp;

    type Loopback = AutomodLoopback;

    type JoinInfo = AutomodState;

    type PeerJoinInfo = ();

    type Error = AutomodError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            sessions: HashMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let state = self.join_room(ctx, ctx.room, participant_id)?;

        Ok(ModuleJoinData {
            join_success: state,
            ..Default::default()
        })
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        self.remove_participant(ctx, ctx.room, participant_id)
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            AutomodCommand::Start {
                parameter,
                allow_list,
                playlist,
            } => self.start(ctx, sender, parameter, allow_list, playlist),
            AutomodCommand::Edit {
                allow_list,
                playlist,
            } => self.edit(ctx, sender, allow_list, playlist),
            AutomodCommand::Stop => self.stop(ctx, sender),
            AutomodCommand::Select(select) => self.select(ctx, sender, select),
            AutomodCommand::Yield { next } => self.yield_next(ctx, sender, next),
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            AutomodLoopback::SpeakerTimeLimitReached { speaker } => {
                self.on_speaker_time_limit_reached(ctx, speaker)?;
            }
            AutomodLoopback::ParticipantsMuted(ParticipantsMuted { participants, .. }) => {
                tracing::debug!(
                    "Following participants were muted by the {} module: {participants:?}",
                    Self::NAMESPACE
                );
            }
            AutomodLoopback::ChannelDropped => {
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    AutomodEvent::Error(AutomodError::Internal),
                )?;
            }
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
        // Remove the participant from the remaining list of the old room
        self.remove_participant(ctx, old_room, participant_id)?;

        let Some(state) = self.join_room(ctx, new_room, participant_id)? else {
            // Automod not active in the new room, return empty `ModuleSwitchData`
            return Ok(ModuleSwitchData::<Self>::default());
        };

        let switch_success = ctx
            .participant_state(participant_id)
            .with_context(|| format!("Missing state for participant {participant_id}"))?
            .connections()
            .map(|con| (con, Some(state.clone())))
            .collect();

        Ok(ModuleSwitchData {
            switch_success,
            ..Default::default()
        })
    }

    fn on_breakout_closed(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        // Remove the sessions for the breakout rooms
        self.sessions.retain(|room, _| *room == RoomKind::Main);
        Ok(())
    }
}

impl AutomodModule {
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn start(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        parameter: Parameter,
        allow_list: Option<Vec<ParticipantId>>,
        playlist: Option<Vec<ParticipantId>>,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        if !ctx.is_moderator(sender) {
            return Err(AutomodError::InsufficientPermissions.into());
        }

        let remaining = Self::resolve_valid_speaker_list(
            ctx,
            parameter.selection_strategy,
            allow_list,
            playlist,
        )
        .ok_or(SignalingModuleError::Module(AutomodError::InvalidSelection))?;

        match self.sessions.entry(ctx.room) {
            Occupied(..) => return Err(AutomodError::SessionAlreadyRunning.into()),
            Vacant(vacant) => {
                tracing::debug!("Starting automod session in room: {:?}", ctx.room);
                vacant.insert(Session::new(sender, parameter.clone(), remaining.clone()))
            }
        };

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            AutomodEvent::Started(
                FrontendConfig {
                    parameter,
                    history: Vec::new(),
                    remaining,
                    issued_by: sender,
                }
                .into_public(),
            ),
        )?;

        let (tx, rx) = oneshot::channel();
        ctx.send_internal_command::<LiveKitModule>(LiveKitInternal::Mute {
            sender: None,
            participants: ctx.participants.in_room(ctx.room).ids().collect(),
            return_channel: tx,
        });

        ctx.recv_loopback(rx, AutomodLoopback::ParticipantsMuted);

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn edit(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        allow_list: Option<Vec<ParticipantId>>,
        playlist: Option<Vec<ParticipantId>>,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        if !ctx.is_moderator(sender) {
            return Err(AutomodError::InsufficientPermissions.into());
        }

        // Only edit if automod is active in the current room
        let Some(session) = self.sessions.get_mut(&ctx.room) else {
            return Err(AutomodError::SessionNotRunning.into());
        };

        let remaining = Self::resolve_valid_speaker_list(
            ctx,
            session.parameter.selection_strategy,
            allow_list,
            playlist,
        )
        .ok_or(SignalingModuleError::Module(AutomodError::InvalidEdit))?;
        session.remaining.clone_from(&remaining);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            AutomodEvent::RemainingUpdated { remaining },
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn stop(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        if !ctx.is_moderator(sender) {
            return Err(AutomodError::InsufficientPermissions.into());
        }

        self.stop_session(ctx, StoppedReason::StoppedByModerator { issued_by: sender })?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn stop_session(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        reason: StoppedReason,
    ) -> Result<(), FatalError> {
        self.sessions.remove(&ctx.room);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            AutomodEvent::Stopped(reason),
        )
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn select(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        select: Select,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        if !ctx.is_moderator(sender) {
            return Err(AutomodError::InsufficientPermissions.into());
        }

        match select {
            Select::None => self.select_none(ctx),
            Select::Random => self.select_random(ctx),
            Select::Next => self.select_next(ctx),
            Select::Specific {
                participant,
                keep_in_remaining,
            } => self.select_specific(ctx, participant, keep_in_remaining),
        }
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn select_none(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        let session = self
            .sessions
            .get_mut(&ctx.room)
            .ok_or(AutomodError::SessionNotRunning)?;
        let previous_speaker = session.speaker;

        let update = speaker_selection::select_unchecked(session, None);
        let time_limit = session.parameter.time_limit;
        self.handle_speaker_update(
            ctx,
            SpeakerSelectionOutput::ContinueWith { update },
            time_limit,
            previous_speaker,
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn select_random(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        let session = self
            .sessions
            .get_mut(&ctx.room)
            .ok_or(AutomodError::SessionNotRunning)?;
        let previous_speaker = session.speaker;

        let update = speaker_selection::select_random(session, &mut rand::rng());
        let time_limit = session.parameter.time_limit;
        self.handle_speaker_update(ctx, update, time_limit, previous_speaker)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn select_next(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        let session = self
            .sessions
            .get_mut(&ctx.room)
            .ok_or(AutomodError::SessionNotRunning)?;

        let valid = match session.parameter.selection_strategy {
            SelectionStrategy::None | SelectionStrategy::Nomination => false,
            SelectionStrategy::Playlist | SelectionStrategy::Random => true,
        };

        if !valid {
            return Err(AutomodError::InvalidSelection.into());
        }

        let previous_speaker = session.speaker;

        let update = speaker_selection::select_next(session, None, &mut rand::rng())?;
        let time_limit = session.parameter.time_limit;
        self.handle_speaker_update(ctx, update, time_limit, previous_speaker)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn select_specific(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant: ParticipantId,
        keep_in_remaining: bool,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        let session = self
            .sessions
            .get_mut(&ctx.room)
            .ok_or(AutomodError::SessionNotRunning)?;

        if !ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .contains(&participant)
        {
            return Err(AutomodError::InvalidSelection.into());
        }

        let previous_speaker = session.speaker;
        let output =
            speaker_selection::select_specific(session, Some(participant), keep_in_remaining)?;

        let time_limit = session.parameter.time_limit;
        self.handle_speaker_update(ctx, output, time_limit, previous_speaker)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn handle_speaker_update(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        output: SpeakerSelectionOutput,
        time_limit: Option<Duration>,
        previous_speaker: Option<ParticipantId>,
    ) -> Result<(), FatalError> {
        // Mute all participants when the speaker is changed
        let (tx, rx) = oneshot::channel();
        ctx.send_internal_command::<LiveKitModule>(LiveKitInternal::Mute {
            sender: None,
            participants: ctx
                .participants
                .in_room(ctx.room)
                .connected()
                .ids()
                .collect(),
            return_channel: tx,
        });
        ctx.recv_loopback(rx, AutomodLoopback::ParticipantsMuted);

        let update = match output {
            SpeakerSelectionOutput::ContinueWith { update } => update,
            SpeakerSelectionOutput::End => {
                return self.stop_session(ctx, StoppedReason::SessionFinished);
            }
        };

        if let Some(SpeakerUpdate {
            speaker,
            history,
            remaining,
        }) = update
        {
            if let Some(time_limit) = time_limit {
                ctx.loopback_after(time_limit, move || {
                    AutomodLoopback::SpeakerTimeLimitReached { speaker }
                });
            }

            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                AutomodEvent::SpeakerUpdated {
                    speaker,
                    history,
                    remaining,
                },
            )?;
        } else {
            let session = self.sessions.get(&ctx.room).with_context(|| {
                format!(
                    "Trying to handle speaker update in room '{:?}' without a running automod session",
                    ctx.room
                )
            }).map_err(FatalError)?;

            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                AutomodEvent::SpeakerUpdated {
                    speaker: session.speaker,
                    history: Some(session.participant_history().collect()),
                    remaining: Some(session.remaining.clone()),
                },
            )?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn yield_next(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        next: Option<ParticipantId>,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        let session = self
            .sessions
            .get_mut(&ctx.room)
            .ok_or(AutomodError::SessionNotRunning)?;

        if session.speaker != Some(sender) {
            return Err(AutomodError::InsufficientPermissions.into());
        }

        let valid = match session.parameter.selection_strategy {
            SelectionStrategy::None => false,
            SelectionStrategy::Playlist | SelectionStrategy::Random => next.is_none(),
            SelectionStrategy::Nomination => next.is_some(),
        };

        if !valid {
            return Err(AutomodError::InvalidSelection.into());
        }

        let previous_speaker = session.speaker;

        let output = speaker_selection::select_next(session, next, &mut rand::rng())?;
        let time_limit = session.parameter.time_limit;
        self.handle_speaker_update(ctx, output, time_limit, previous_speaker)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn join_room(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        room: RoomKind,
        participant_id: ParticipantId,
    ) -> Result<Option<AutomodState>, FatalError> {
        let session = self.sessions.get_mut(&room);
        let Some(session) = session else {
            // Automod not active, return empty JoinInfo
            return Ok(None);
        };

        let history: Vec<ParticipantId> = session.participant_history().collect();

        if session.parameter.auto_append_on_join && !history.contains(&participant_id) {
            // Append the joining participant to the history
            session.remaining.push(participant_id);
            ctx.send_ws_message(
                ctx.participants
                    .in_room(room)
                    .connected()
                    .iter()
                    .filter_map(
                        |(&id, _)| {
                            if id == participant_id { None } else { Some(id) }
                        },
                    ),
                AutomodEvent::RemainingUpdated {
                    remaining: session.remaining.clone(),
                },
            )?;
        }

        Ok(Some(AutomodState {
            config: FrontendConfig {
                parameter: session.parameter.clone(),
                history,
                remaining: session.remaining.clone(),
                issued_by: session.issued_by,
            }
            .into_public(),
            speaker: session.speaker,
        }))
    }

    /// Removes the participant from the automod session in `room`.
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn remove_participant(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        room: RoomKind,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        let session = self.sessions.get_mut(&room);
        let Some(session) = session else {
            // Automod not active, nothing to do
            return Ok(());
        };

        let index = session
            .remaining
            .iter()
            .position(|id| *id == participant_id);
        if let Some(index) = index {
            session.remaining.remove(index);
            ctx.send_ws_message(
                ctx.participants.in_room(room).connected().ids(),
                AutomodEvent::RemainingUpdated {
                    remaining: session.remaining.clone(),
                },
            )?;
        }

        if session.speaker == Some(participant_id) {
            self.select_next(ctx)?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn on_speaker_time_limit_reached(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        speaker: Option<ParticipantId>,
    ) -> Result<(), SignalingModuleError<AutomodError>> {
        let Some(session) = self.sessions.get(&ctx.room) else {
            // The session has ended in the meantime
            return Ok(());
        };
        if session.speaker != speaker {
            // The speaker has changed in the meantime
            return Ok(());
        }

        match session.parameter.selection_strategy {
            // Selection strategies `None` and `Nomination` do not have a concept of
            // a "next" speaker. Select no speaker.
            SelectionStrategy::None | SelectionStrategy::Nomination => self.select_none(ctx)?,
            _ => self.select_next(ctx)?,
        }

        Ok(())
    }

    /// Returns the list corresponding to the given `selection_strategy` or [`None`]
    /// if no list matches the criteria.
    fn resolve_valid_speaker_list(
        ctx: &mut ModuleContext<'_, Self>,
        selection_strategy: SelectionStrategy,
        allow_list: Option<Vec<ParticipantId>>,
        playlist: Option<Vec<ParticipantId>>,
    ) -> Option<Vec<ParticipantId>> {
        let list = if selection_strategy.uses_allow_list() {
            allow_list
        } else {
            playlist
        };

        let list = match list {
            Some(list) if list.is_empty() => return None,
            Some(list) => list,
            None => return None,
        };

        let connected_participants: Vec<ParticipantId> = ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .ids()
            .collect();
        if !list
            .iter()
            .all(|participant| connected_participants.contains(participant))
        {
            return None;
        }

        Some(list)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::AutomodModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&AutomodModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[automod]");
    }
}
