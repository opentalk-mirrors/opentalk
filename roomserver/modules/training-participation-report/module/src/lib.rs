// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::Duration,
};

use anyhow::Context as _;
use chrono::TimeDelta;
use icu_locid::{LanguageIdentifier, langid};
use opentalk_roomserver_signaling::{
    localization,
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
    storage::assets::AssetUploaded,
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_training_participation_report::{
    TRAINING_PARTICIPATION_REPORT_MODULE_ID, TrainingParticipationReportCommand,
    TrainingParticipationReportEvent, TrainingParticipationReportParameterSet,
    TrainingParticipationReportState,
    event::{
        PresenceLoggingEndedReason, PresenceLoggingStartedReason, TrainingParticipationReportError,
    },
    settings::TrainingParticipationReportSettings,
    state::ParticipationLoggingState,
};
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    modules::ModuleId,
    time::Timestamp,
    training_participation_report::TimeRange,
    users::DisplayName,
};
use opentalk_types_signaling::ParticipantId;
use rand::RngExt as _;
use tokio::sync::oneshot::Sender;

use crate::{loopback::TrainingParticipationReportLoopback, template::ReportTemplateParameter};

mod loopback;
mod template;

const DEFAULT_INITIAL_AFTER: Duration = Duration::from_mins(10);
const DEFAULT_INITIAL_WITHIN: Duration = Duration::from_mins(20);

const DEFAULT_INTERVAL_AFTER: Duration = Duration::from_mins(60 + 45); // 1 hour 45 minutes
const DEFAULT_INTERVAL_WITHIN: Duration = Duration::from_mins(30);

const AVAILABLE_LANGUAGES: &[LanguageIdentifier] = &[langid!("en"), langid!("de")];

struct TrainingSession {
    initial_delay: TimeRange,
    interval: TimeRange,
    started_at: Timestamp,
    /// The known participants, includes all participants that were present at any time during
    /// the training session.
    participants: HashSet<ParticipantId>,
    checkpoints: Vec<Checkpoint>,
    tx_cancel: Sender<TrainingParticipationReportLoopback>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct Checkpoint {
    pub timestamp: Timestamp,
    pub presence: HashMap<ParticipantId, Timestamp>,
}

pub struct TrainingParticipationReportModule {
    title: EventTitle,
    description: EventDescription,
    autostart: Option<TrainingParticipationReportParameterSet>,
    sessions: HashMap<RoomKind, TrainingSession>,
    typst_package_path: PathBuf,
}

impl SignalingModuleDescription for TrainingParticipationReportModule {
    const MODULE_ID: ModuleId = TRAINING_PARTICIPATION_REPORT_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles training participation report functionality. Participants are asked to confirm their presence repeatedly at pre-configured time intervals. These confirmations are documented in the training participation report which is created automatically at the end of the meeting.";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = TrainingParticipationReportSettings;
}

impl SignalingModule for TrainingParticipationReportModule {
    const NAMESPACE: ModuleId = TRAINING_PARTICIPATION_REPORT_MODULE_ID;

    type Incoming = TrainingParticipationReportCommand;

    type Outgoing = TrainingParticipationReportEvent;

    type Internal = NoOp;

    type Loopback = TrainingParticipationReportLoopback;

    type JoinInfo = TrainingParticipationReportState;

    type PeerJoinInfo = ();

    type Error = TrainingParticipationReportError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let event = init_data.room_parameters.event.as_ref();
        let title = event
            .map(|e| e.title.clone())
            .unwrap_or_else(|| EventTitle::from_str_lossy(""));
        let description = event
            .map(|e| e.description.clone())
            .unwrap_or_else(|| EventDescription::from_str_lossy(""));
        let autostart = init_data
            .room_parameters
            .module_settings
            .get::<TrainingParticipationReportSettings>()
            .ok()
            .flatten()
            .and_then(|settings| settings.autostart);
        let typst_package_path = init_data.settings.reports.typst.packages_path.clone();

        Some(Self {
            sessions: HashMap::new(),
            title,
            autostart,
            description,
            typst_package_path,
        })
    }

    #[tracing::instrument(skip(self, ctx, _connection_id, _is_first_connection), level = "debug")]
    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let join_success = self.join_room(ctx, ctx.room, participant_id)?;
        Ok(ModuleJoinData {
            join_success: Some(join_success),
            peer_data: PeerDataMap::default(),
            peer_events: PeerDataMap::default(),
        })
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        if ctx
            .participants
            .connected()
            .ids()
            .any(|id| id != participant_id)
        {
            // This is not the last participant, do nothing.
            return Ok(());
        }

        if ctx
            .participant_state(participant_id)
            .map(|state| state.connections().any(|id| id != connection_id))
            .unwrap_or(false)
        {
            // This is not the last connection of the participant, do nothing.
            return Ok(());
        }

        // End all training sessions when the last participant disconnected.
        let sessions: Vec<TrainingSession> =
            self.sessions.drain().map(|(_, session)| session).collect();
        for session in sessions {
            _ = self
                .stop_presence_logging(ctx, session)
                .inspect_err(|e| tracing::error!("Error while stopping presence logging: {e:?}"));
        }

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
            TrainingParticipationReportCommand::EnablePresenceLogging {
                initial_checkpoint_delay,
                checkpoint_interval,
            } => self.enable_presence_logging(
                ctx,
                sender,
                initial_checkpoint_delay,
                checkpoint_interval,
            ),
            TrainingParticipationReportCommand::DisablePresenceLogging => {
                self.disable_presence_logging(ctx, sender)
            }
            TrainingParticipationReportCommand::ConfirmPresence => {
                self.confirm_presence(ctx, sender)
            }
            TrainingParticipationReportCommand::TriggerPresenceCheck => {
                self.trigger_presence_check(ctx, sender)
            }
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            TrainingParticipationReportLoopback::CheckpointReached(timestamp) => {
                self.checkpoint_reached(ctx, timestamp)?;
            }
            // The checkpoint was canceled, nothing to do
            TrainingParticipationReportLoopback::CheckpointCanceled => {}
            TrainingParticipationReportLoopback::ReportUploaded(AssetUploaded {
                id,
                filename,
                quota,
                ..
            }) => {
                // Send the report to the room owner
                if let Some(participant_id) = Self::connected_owner(ctx) {
                    ctx.send_ws_message(
                        [participant_id],
                        TrainingParticipationReportEvent::PdfCreated {
                            filename,
                            asset_id: id,
                            quota,
                        },
                    )?;
                }
            }
            TrainingParticipationReportLoopback::ChannelDropped => {
                tracing::error!("The loopback channel was dropped");
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    TrainingParticipationReportEvent::Error(
                        TrainingParticipationReportError::Internal,
                    ),
                )?;
            }
            TrainingParticipationReportLoopback::Error(err) => return Err(err.into()),
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        // We do not stop presence logging when switching rooms because all participants switching
        // the breakout room is not as final as disconnecting. They might come back.
        let state = self.join_room(ctx, new_room, participant_id)?;
        let switch_success = ctx
            .participant_state(participant_id)
            .with_context(|| format!("Missing state for participant {participant_id}"))?
            .connections()
            .map(|connection_id| (connection_id, Some(state.clone())))
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
        let sessions: Vec<TrainingSession> = self
            .sessions
            .extract_if(|room, _| *room != RoomKind::Main)
            .map(|(_, session)| session)
            .collect();

        for session in sessions {
            _ = self
                .stop_presence_logging(ctx, session)
                .inspect_err(|e| tracing::error!("Error while stopping presence logging {e:?}"));
        }

        Ok(())
    }

    fn on_closing(&mut self, ctx: &mut ModuleContext<'_, Self>) -> Result<(), anyhow::Error> {
        let sessions: Vec<TrainingSession> =
            self.sessions.drain().map(|(_, session)| session).collect();

        for session in sessions {
            _ = self
                .stop_presence_logging(ctx, session)
                .inspect_err(|e| tracing::error!("Error while stopping presence logging: {e:?}"));
        }

        Ok(())
    }
}

impl TrainingParticipationReportModule {
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn enable_presence_logging(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        initial_delay: Option<TimeRange>,
        interval: Option<TimeRange>,
    ) -> Result<(), SignalingModuleError<TrainingParticipationReportError>> {
        if !ctx.is_room_owner(sender) {
            return Err(TrainingParticipationReportError::InsufficientPermissions.into());
        }

        if self.sessions.contains_key(&ctx.room) {
            return Err(TrainingParticipationReportError::PresenceLoggingAlreadyEnabled.into());
        }

        let initial_delay = initial_delay.unwrap_or(TimeRange::new_with_clamped_durations(
            DEFAULT_INITIAL_AFTER,
            DEFAULT_INITIAL_WITHIN,
        ));
        let interval = interval.unwrap_or(TimeRange::new_with_clamped_durations(
            DEFAULT_INTERVAL_AFTER,
            DEFAULT_INTERVAL_WITHIN,
        ));

        self.start_presence_logging(
            ctx,
            initial_delay,
            interval,
            PresenceLoggingStartedReason::StartedManually,
        )?;

        Ok(())
    }

    fn start_presence_logging(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        initial_delay: TimeRange,
        interval: TimeRange,
        reason: PresenceLoggingStartedReason,
    ) -> Result<(), FatalError> {
        let (first_checkpoint, tx_cancel) = Self::schedule_next_checkpoint(ctx, &initial_delay);

        self.sessions.insert(
            ctx.room,
            TrainingSession {
                initial_delay,
                interval,
                started_at: ctx.timestamp,
                participants: ctx
                    .participants
                    .in_room(ctx.room)
                    .connected()
                    .ids()
                    .collect(),
                checkpoints: Vec::new(),
                tx_cancel,
            },
        );

        if let Some(owner) = Self::connected_owner(ctx) {
            ctx.send_ws_message(
                [owner],
                TrainingParticipationReportEvent::PresenceLoggingStarted {
                    first_checkpoint: Some(first_checkpoint),
                    reason: Some(reason),
                },
            )?;

            ctx.send_ws_message(
                ctx.participants
                    .in_room(ctx.room)
                    .connected()
                    .ids()
                    .filter(|id| *id != owner),
                TrainingParticipationReportEvent::PresenceLoggingStarted {
                    first_checkpoint: None,
                    reason: None,
                },
            )?;
        } else {
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                TrainingParticipationReportEvent::PresenceLoggingStarted {
                    first_checkpoint: None,
                    reason: None,
                },
            )?;
        }

        Ok(())
    }

    #[must_use]
    fn schedule_next_checkpoint(
        ctx: &ModuleContext<'_, Self>,
        time_range: &TimeRange,
    ) -> (Timestamp, Sender<TrainingParticipationReportLoopback>) {
        let seconds_to_wait = Self::random_waiting_duration_seconds(time_range);
        let next_checkpoint =
            ctx.timestamp + TimeDelta::from_std(seconds_to_wait).unwrap_or(TimeDelta::MAX);

        let tx = ctx.loopback_after(seconds_to_wait, move || {
            TrainingParticipationReportLoopback::CheckpointReached(next_checkpoint)
        });

        (next_checkpoint, tx)
    }

    fn random_waiting_duration_seconds(range: &TimeRange) -> Duration {
        let offset = if range.within() == Duration::ZERO {
            Duration::ZERO
        } else {
            rand::rng().random_range(Duration::ZERO..range.within())
        };
        range.after().saturating_add(offset)
    }

    fn disable_presence_logging(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<TrainingParticipationReportError>> {
        if !ctx.is_room_owner(sender) {
            return Err(TrainingParticipationReportError::InsufficientPermissions.into());
        }

        let Some(session) = self.sessions.remove(&ctx.room) else {
            return Err(TrainingParticipationReportError::PresenceLoggingNotEnabled.into());
        };

        self.stop_presence_logging(ctx, session).or_else(|e| {
            ctx.send_ws_message([sender], TrainingParticipationReportEvent::Error(e))
        })?;

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            TrainingParticipationReportEvent::PresenceLoggingEnded {
                reason: PresenceLoggingEndedReason::StoppedManually,
            },
        )?;

        Ok(())
    }

    fn stop_presence_logging(
        &self,
        ctx: &ModuleContext<'_, Self>,
        session: TrainingSession,
    ) -> Result<(), TrainingParticipationReportError> {
        let TrainingSession {
            started_at,
            participants,
            checkpoints,
            tx_cancel,
            ..
        } = session;

        if tx_cancel
            .send(TrainingParticipationReportLoopback::CheckpointCanceled)
            .is_err()
        {
            tracing::debug!("Cancel receiver has been dropped");
        }

        if !checkpoints.is_empty() {
            self.generate_report(ctx, started_at, participants, checkpoints)?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn checkpoint_reached(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError<TrainingParticipationReportError>> {
        let Some(session) = self.sessions.get_mut(&ctx.room) else {
            tracing::warn!("Training session ended without the checkpoint being canceled");
            return Ok(());
        };

        // The timer of the previous checkpoint has already fired, so the old cancel sender can be
        // dropped without notifying anybody.
        Self::create_checkpoint(ctx, session, timestamp)?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn trigger_presence_check(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<TrainingParticipationReportError>> {
        if !ctx.is_room_owner(sender) {
            return Err(TrainingParticipationReportError::InsufficientPermissions.into());
        }

        let Some(session) = self.sessions.get_mut(&ctx.room) else {
            return Err(TrainingParticipationReportError::PresenceLoggingNotEnabled.into());
        };

        let previous_tx_cancel = Self::create_checkpoint(ctx, session, ctx.timestamp)?;

        // Cancel the pending checkpoint timer. Dropping the sender without sending would result
        // in a `ChannelDropped` loopback event.
        if previous_tx_cancel
            .send(TrainingParticipationReportLoopback::CheckpointCanceled)
            .is_err()
        {
            tracing::debug!("Cancel receiver has been dropped");
        }

        Ok(())
    }

    /// Create a new checkpoint at `timestamp`, schedule the next one according to the session
    /// interval and request all participants in the room to confirm their presence.
    ///
    /// Returns the cancel sender of the previously scheduled checkpoint.
    fn create_checkpoint(
        ctx: &ModuleContext<'_, Self>,
        session: &mut TrainingSession,
        timestamp: Timestamp,
    ) -> Result<Sender<TrainingParticipationReportLoopback>, FatalError> {
        session.checkpoints.push(Checkpoint {
            timestamp,
            presence: HashMap::new(),
        });

        let (_, tx) = Self::schedule_next_checkpoint(ctx, &session.interval);
        let previous_tx_cancel = std::mem::replace(&mut session.tx_cancel, tx);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            TrainingParticipationReportEvent::PresenceConfirmationRequested,
        )?;

        Ok(previous_tx_cancel)
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn confirm_presence(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<TrainingParticipationReportError>> {
        let Some(TrainingSession { checkpoints, .. }) = self.sessions.get_mut(&ctx.room) else {
            return Err(TrainingParticipationReportError::PresenceLoggingNotEnabled.into());
        };

        if let Some(checkpoint) = checkpoints.last_mut() {
            checkpoint.presence.insert(sender, ctx.timestamp);
        } else {
            tracing::warn!(
                "Checkpoint does not exist, creating a new one. Timestamp might be inaccurate."
            );
            checkpoints.push(Checkpoint {
                timestamp: ctx.timestamp,
                presence: HashMap::from_iter([(sender, ctx.timestamp)]),
            });
        };

        ctx.send_ws_message(
            [sender],
            TrainingParticipationReportEvent::PresenceConfirmationLogged,
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn generate_report(
        &self,
        ctx: &ModuleContext<'_, Self>,
        started_at: Timestamp,
        participants: HashSet<ParticipantId>,
        checkpoints: Vec<Checkpoint>,
    ) -> Result<(), TrainingParticipationReportError> {
        let participant_names: HashMap<ParticipantId, Option<DisplayName>> = participants
            .into_iter()
            .map(|id| {
                (
                    id,
                    ctx.participant_state(id)
                        .map(|state| state.display_name.clone()),
                )
            })
            .collect();
        let report_language = localization::negotiate_languages(ctx, AVAILABLE_LANGUAGES)
            .ok_or(TrainingParticipationReportError::Generate)?;
        let template_parameter = ReportTemplateParameter::new(
            self.title.clone(),
            self.description.clone(),
            ctx.room_task_info.room.created_by.timezone,
            report_language,
            started_at,
            ctx.timestamp,
            participant_names,
            checkpoints,
        );
        let typst_package_path = self.typst_package_path.clone();
        ctx.spawn(loopback::create_report(
            ctx.assets(),
            template_parameter,
            typst_package_path,
        ));

        Ok(())
    }

    /// Find the connected participant in the current (breakout) room that is the room owner, if
    /// any.
    fn connected_owner(ctx: &ModuleContext<'_, Self>) -> Option<ParticipantId> {
        let owner_user_id = ctx.room_task_info.owner();
        ctx.participants
            .in_room(ctx.room)
            .connected()
            .iter()
            .find(|(_, state)| state.kind.user_id().is_some_and(|id| id == owner_user_id))
            .map(|(id, _)| *id)
    }

    /// Handle a participant joining a (breakout) room, returning the state they should receive.
    fn join_room(
        &mut self,
        ctx: &mut ModuleContext<'_, TrainingParticipationReportModule>,
        room: RoomKind,
        participant_id: ParticipantId,
    ) -> Result<
        TrainingParticipationReportState,
        SignalingModuleError<TrainingParticipationReportError>,
    > {
        // If a training session is already active
        if let Some(TrainingSession {
            initial_delay,
            interval,
            participants,
            checkpoints,
            ..
        }) = self.sessions.get_mut(&room)
        {
            // If a training session is active, add the new participant to the list of known
            // participants.
            participants.insert(participant_id);

            // If there already is a checkpoint, the new participant can immediately confirm
            // their presence.
            let state = if checkpoints.is_empty() {
                ParticipationLoggingState::Enabled
            } else {
                ParticipationLoggingState::WaitingForConfirmation
            };
            return Ok(TrainingParticipationReportState {
                state,
                parameters: ctx.is_room_owner(participant_id).then(|| {
                    TrainingParticipationReportParameterSet {
                        initial_checkpoint_delay: initial_delay.clone(),
                        checkpoint_interval: interval.clone(),
                    }
                }),
            });
        }

        // If a training session is planned
        if let Some(TrainingParticipationReportParameterSet {
            initial_checkpoint_delay,
            checkpoint_interval,
        }) = self.autostart.clone()
        {
            // If a training session is planned, but not active, start it now.
            // This is the case when the first participant joins the room.
            self.start_presence_logging(
                ctx,
                initial_checkpoint_delay.clone(),
                checkpoint_interval.clone(),
                PresenceLoggingStartedReason::Autostart,
            )?;

            return Ok(TrainingParticipationReportState {
                state: ParticipationLoggingState::Enabled,
                parameters: ctx.is_room_owner(participant_id).then_some(
                    TrainingParticipationReportParameterSet {
                        initial_checkpoint_delay,
                        checkpoint_interval,
                    },
                ),
            });
        }

        // A training session is neither active nor planned
        Ok(TrainingParticipationReportState {
            state: ParticipationLoggingState::Disabled,
            parameters: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::TrainingParticipationReportModule;

    #[test]
    fn serialize_example_config() {
        let produced =
            toml::to_string_pretty(&TrainingParticipationReportModule::example_config()).unwrap();

        assert_snapshot!(produced, @"
        [training_participation_report.autostart.checkpoint_interval]
        after = 300
        within = 400

        [training_participation_report.autostart.initial_checkpoint_delay]
        after = 100
        within = 200
        ");
    }
}
