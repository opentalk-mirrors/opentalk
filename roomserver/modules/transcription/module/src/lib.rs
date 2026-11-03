// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{Context as _, anyhow};
use fluent_langneg::NegotiationStrategy;
use icu_locid::{LanguageIdentifier, langid};
use opentalk_roomserver_room::storage::assets::AssetUploaded;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    client_parameters::ClientKind,
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_transcription::{
    TRANSCRIPTION_FEATURE_ID, TRANSCRIPTION_MODULE_ID,
    command::TranscriptionCommand,
    config::AutomaticTranscriptionConfig,
    event::{TranscriptionError, TranscriptionEvent},
    peer_state::TranscriptionPeerState,
    service::{command::TranscriptionServiceCommand, event::TranscriptionServiceEvent},
    settings::TranscriptionSettings,
    state::{AutomaticTranscriptionState, TranscriptionState},
    status::{TranscriptionModes, TranscriptionStatus},
};
use opentalk_transcription_web_api::v1::TranscriptionTarget;
use opentalk_types_common::{
    features::ModuleFeatureId, modules::ModuleId, time::TimeZone, transcription::RequestId,
};
use opentalk_types_signaling::ParticipantId;

use crate::{
    loopback::LoopBackEvent,
    state::{
        ActiveTranscriptionModes, ServiceAction, ServiceState, TranscriptionModeConfig,
        TranscriptionRoomState, automatic_transcription::FinishedAutomaticTranscription,
    },
};

const AVAILABLE_LANGUAGES: &[LanguageIdentifier] = &[langid!("en"), langid!("de")];

mod loopback;
mod state;
mod template;

pub struct TranscriptionModule {
    settings: TranscriptionSettings,
    typst_package_path: PathBuf,
    http_client: reqwest::Client,
    transcription_states: BTreeMap<RoomKind, TranscriptionRoomState>,
}

impl SignalingModuleDescription for TranscriptionModule {
    const MODULE_ID: ModuleId = TRANSCRIPTION_MODULE_ID;
    const DESCRIPTION: &'static str = "Live transcription for meetings";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[SignalingModuleFeatureDescription {
        feature_id: TRANSCRIPTION_FEATURE_ID,
        description: "Allows to create transcriptions for meetings",
    }];

    type Settings = TranscriptionSettings;
}

impl SignalingModule for TranscriptionModule {
    const NAMESPACE: ModuleId = TRANSCRIPTION_MODULE_ID;

    type Incoming = TranscriptionCommand;
    type Outgoing = TranscriptionEvent;
    type Internal = NoOp;
    type Loopback = LoopBackEvent;
    type JoinInfo = TranscriptionState;
    type PeerJoinInfo = TranscriptionPeerState;
    type Error = TranscriptionError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let tariff = &init_data.room_parameters.tariff;
        if tariff.disabled_features.contains(&ModuleFeatureId {
            module: TRANSCRIPTION_MODULE_ID,
            feature: TRANSCRIPTION_FEATURE_ID,
        }) {
            return None;
        }

        let settings = init_data
            .room_parameters
            .module_settings
            .get::<TranscriptionSettings>()
            .ok()??;
        let typst_package_path = init_data.settings.reports.typst.packages_path.clone();

        Some(Self {
            settings,
            typst_package_path,
            http_client: reqwest::Client::new(),
            transcription_states: BTreeMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        self.record_transcript_presence(ctx, participant_id)?;

        let PeerDataMaps {
            peer_events,
            peer_data,
        } = self.build_peer_data_maps(ctx, participant_id)?;

        Ok(ModuleJoinData {
            join_success: Some(self.build_module_state(ctx.room, participant_id)),
            peer_events,
            peer_data,
        })
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        if !is_transcription_service_for_current_room(ctx, participant_id)? {
            return Ok(());
        }

        if self
            .transcription_states
            .get(&ctx.room)
            .is_none_or(|state| state.service.id() != Some(participant_id))
        {
            return Ok(());
        }

        tracing::warn!("Transcription service disconnected unexpectedly");
        self.handle_service_error(ctx, TranscriptionError::ServiceDisconnected)
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        command: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match command {
            TranscriptionCommand::Start { language } => self.start_transcription_mode(
                ctx,
                sender,
                language,
                TranscriptionModeConfig::LiveTranscription,
            ),
            TranscriptionCommand::Stop => self.stop_live_transcription(ctx, sender),
            TranscriptionCommand::ChangeLanguage { language } => {
                self.change_language(ctx, sender, language)
            }
            TranscriptionCommand::TranscriptionServiceEvent { event } => {
                self.handle_transcription_service_event(ctx, event, sender)
            }
            TranscriptionCommand::StartAutomaticTranscription {
                language,
                timezone,
                pseudonymous,
            } => self.start_transcription_mode(
                ctx,
                sender,
                language,
                TranscriptionModeConfig::AutomaticTranscription {
                    config: AutomaticTranscriptionConfig { pseudonymous },
                    time_zone: timezone.unwrap_or_else(|| {
                        ctx.participant_state(sender)
                            .and_then(|state| state.kind.time_zone())
                            .unwrap_or_else(TimeZone::utc)
                    }),
                },
            ),
            TranscriptionCommand::PauseAutomaticTranscription => {
                self.pause_automatic_transcription(ctx, sender)
            }
            TranscriptionCommand::ResumeAutomaticTranscription => {
                self.resume_automatic_transcription(ctx, sender)
            }
            TranscriptionCommand::StopAutomaticTranscription => {
                self.stop_automatic_transcription(ctx, sender)
            }
            TranscriptionCommand::SetAutomaticTranscriptionConsent { consent } => {
                self.set_consent(ctx, sender, consent)
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
        self.record_transcript_presence(ctx, participant_id)?;

        let state = self.build_module_state(new_room, participant_id);
        let switch_success = ctx
            .participant_state(participant_id)
            .with_context(|| format!("Missing state for participant {participant_id}"))?
            .connections()
            .map(|con| (con, Some(state.clone())))
            .collect();

        let PeerDataMaps {
            peer_events,
            peer_data,
        } = self.build_peer_data_maps(ctx, participant_id)?;

        Ok(ModuleSwitchData {
            switch_success,
            peer_events,
            peer_data,
        })
    }

    fn on_breakout_closed(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let states: Vec<_> = self
            .transcription_states
            .extract_if(.., |room, _| *room != RoomKind::Main)
            .map(|(_, state)| state)
            .collect();

        let mut error = false;
        for mut state in states {
            if let Some(automatic_transcription) =
                state.modes.deactivate_automatic_transcription(ctx)
            {
                error |= self
                    .generate_transcript(ctx, state.language.as_deref(), automatic_transcription)
                    .is_err();
            }
        }

        if error {
            Err(TranscriptionError::GenerateTranscript.into())
        } else {
            Ok(())
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            LoopBackEvent::TranscriptionRequestFailed => {
                tracing::warn!("Failed to request transcription service");
                self.handle_service_error(ctx, TranscriptionError::ServiceRequestFailed)?;
            }
            LoopBackEvent::AutomaticTranscriptionGenerated(AssetUploaded {
                id: asset_id,
                filename,
                quota,
            }) => {
                tracing::debug!("Generated automatic transcription: {asset_id} ({filename})");
                ctx.send_ws_message(
                    ctx.participants.connected().ids(),
                    TranscriptionEvent::PdfCreated {
                        filename,
                        asset_id,
                        quota,
                    },
                )?;
            }
            LoopBackEvent::AutomaticTranscriptionGenerationFailed => {
                return Err(TranscriptionError::GenerateTranscript.into());
            }
        }

        Ok(())
    }

    fn on_moderation_role_change(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        if !ctx.is_moderator(participant_id) || !ctx.is_visible(participant_id) {
            // Moderator rights were revoked or the participant is hidden, nothing to do.
            return Ok(());
        }

        let Some(automatic_transcription) = self
            .transcription_states
            .get_mut(&ctx.room)
            .and_then(|state| state.modes.automatic_transcription_mut())
        else {
            // No automatic transcription ongoing, nothing to do.
            return Ok(());
        };

        automatic_transcription
            .seen_moderators
            .insert(participant_id);

        Ok(())
    }

    fn on_closing(&mut self, ctx: &mut ModuleContext<'_, Self>) -> Result<(), anyhow::Error> {
        let Some(mut state) = self.transcription_states.remove(&RoomKind::Main) else {
            // No transcription ongoing, nothing to do.
            return Ok(());
        };

        let Some(automatic_transcription) = state.modes.deactivate_automatic_transcription(ctx)
        else {
            // No automatic transcription ongoing, nothing to do.
            return Ok(());
        };

        self.generate_transcript(ctx, state.language.as_deref(), automatic_transcription)
            .map_err(|e| anyhow!("Failed to generate transcript: {e:?}"))
    }
}

impl TranscriptionModule {
    fn start_transcription_mode(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        language: Option<String>,
        config: TranscriptionModeConfig,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if !ctx.is_moderator(sender) {
            return Err(TranscriptionError::InsufficientPermissions.into());
        }

        if let Some(state) = self.transcription_states.get_mut(&ctx.room) {
            let was_already_active = state.modes.activate(ctx, config).is_unchanged();

            if was_already_active {
                self.send_modes_updated(ctx, [sender])?;
                return Ok(());
            }
        } else {
            // The new service starts as stopped, update_service will start it
            let state = TranscriptionRoomState {
                service: ServiceState::Stopped,
                language,
                modes: ActiveTranscriptionModes::start(ctx, config),
            };
            self.transcription_states.insert(ctx.room, state);
        };

        self.send_modes_updated(ctx, ctx.participants.in_room(ctx.room).connected().ids())?;
        self.update_service(ctx)
    }

    fn request_transcription(
        &mut self,
        ctx: &ModuleContext<'_, TranscriptionModule>,
        request_id: RequestId,
        language: Option<String>,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        let transcription_settings = self.settings.clone();

        let jwt_bearer_token = transcription_settings
            .api_key
            .generate_jwt()
            .context("Failed to generate JWT from transcription api_key")
            .map_err(FatalError)?;

        let http_client = self.http_client.clone();

        let body = TranscriptionTarget {
            room_id: ctx.room_id,
            request_id,
            breakout_room: match ctx.room {
                RoomKind::Main => None,
                RoomKind::Breakout(breakout_id) => Some(breakout_id.into()),
            },
            language,
        };

        ctx.spawn_optional(loopback::request_transcription(
            transcription_settings,
            body,
            http_client,
            jwt_bearer_token,
        ));

        Ok(())
    }

    fn pause_automatic_transcription(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if !ctx.is_moderator(sender) {
            return Err(TranscriptionError::InsufficientPermissions.into());
        }

        let Some(state) = self.transcription_states.get_mut(&ctx.room) else {
            return Err(TranscriptionError::NotActive.into());
        };

        let was_already_paused = state.modes.pause_automatic_transcription()?.is_unchanged();

        if was_already_paused {
            self.send_modes_updated(ctx, [sender])?;
            return Ok(());
        }

        self.send_modes_updated(ctx, ctx.participants.in_room(ctx.room).connected().ids())?;
        self.update_service(ctx)
    }

    fn resume_automatic_transcription(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if !ctx.is_moderator(sender) {
            return Err(TranscriptionError::InsufficientPermissions.into());
        }

        let Some(state) = self.transcription_states.get_mut(&ctx.room) else {
            return Err(TranscriptionError::NotActive.into());
        };

        let was_already_running = state.modes.resume_automatic_transcription()?.is_unchanged();

        if was_already_running {
            self.send_modes_updated(ctx, [sender])?;
            return Ok(());
        }

        self.send_modes_updated(ctx, ctx.participants.in_room(ctx.room).connected().ids())?;
        self.update_service(ctx)
    }

    fn stop_live_transcription(
        &mut self,
        ctx: &mut ModuleContext<'_, TranscriptionModule>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if !ctx.is_moderator(sender) {
            return Err(TranscriptionError::InsufficientPermissions.into());
        }

        let Some(state) = self.transcription_states.get_mut(&ctx.room) else {
            return Err(TranscriptionError::NotActive.into());
        };

        if !state.modes.live_transcription() {
            self.send_modes_updated(ctx, [sender])?;
            return Ok(());
        }

        state.modes.deactivate_live_transcription();
        self.send_modes_updated(ctx, ctx.participants.in_room(ctx.room).connected().ids())?;
        self.update_service(ctx)?;

        Ok(())
    }

    fn stop_automatic_transcription(
        &mut self,
        ctx: &mut ModuleContext<'_, TranscriptionModule>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if !ctx.is_moderator(sender) {
            return Err(TranscriptionError::InsufficientPermissions.into());
        }

        let Some(state) = self.transcription_states.get_mut(&ctx.room) else {
            return Err(TranscriptionError::NotActive.into());
        };

        let Some(automatic_transcription) = state.modes.deactivate_automatic_transcription(ctx)
        else {
            return Err(TranscriptionError::NotActive.into());
        };

        let language = state.language.clone();

        self.send_modes_updated(ctx, ctx.participants.in_room(ctx.room).connected().ids())?;
        self.update_service(ctx)?;

        self.generate_transcript(ctx, language.as_deref(), automatic_transcription)
    }

    fn generate_transcript(
        &self,
        ctx: &ModuleContext<'_, Self>,
        transcription_language: Option<&str>,
        automatic_transcription: FinishedAutomaticTranscription,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        let transcription_language =
            transcription_language.and_then(|language| language.parse::<LanguageIdentifier>().ok());
        let report_language =
            negotiate_languages(ctx, transcription_language.as_ref(), AVAILABLE_LANGUAGES)
                .ok_or(TranscriptionError::GenerateTranscript)?;

        let title = ctx
            .room_task_info
            .room
            .event
            .as_ref()
            .map(|event| event.title.clone());

        ctx.spawn(loopback::create_transcript(
            ctx.assets(),
            title,
            report_language,
            transcription_language.map(|id| id.language),
            automatic_transcription,
            self.typst_package_path.clone(),
        ));

        Ok(())
    }

    /// Cancel a transcription service that is possibly connected to the room, but has not yet sent
    /// the `Started` event.
    fn cancel_transcription_services(
        &self,
        ctx: &ModuleContext<'_, Self>,
    ) -> Result<(), FatalError> {
        let services = ctx
            .participants
            .in_room(ctx.room)
            .connected()
            .iter()
            .filter_map(|(id, state)| {
                if matches!(state.kind, ClientKind::Transcription { room } if room == ctx.room) {
                    Some(*id)
                } else {
                    None
                }
            });

        ctx.send_ws_message(
            services,
            TranscriptionEvent::ServiceCommand {
                command: TranscriptionServiceCommand::Stop,
            },
        )
    }

    /// Stop the transcription service but do **not** remove the transcription state.
    fn stop_transcription_service(&self, ctx: &ModuleContext<'_, Self>) -> Result<(), FatalError> {
        if let Some(participant_id) = self
            .transcription_states
            .get(&ctx.room)
            .and_then(|state| state.service.id())
        {
            ctx.send_ws_message(
                [participant_id],
                TranscriptionEvent::ServiceCommand {
                    command: TranscriptionServiceCommand::Stop,
                },
            )
        } else {
            Ok(())
        }
    }

    /// Returns the transcription state for the current room.
    ///
    /// Stops the transcription service and returns `None` when the room has no transcription state.
    fn state_or_stop_service(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        event: &str,
    ) -> Result<Option<&mut TranscriptionRoomState>, SignalingModuleError<TranscriptionError>> {
        if let Some(state) = self.transcription_states.get_mut(&ctx.room) {
            Ok(Some(state))
        } else {
            tracing::warn!(
                "Got TranscriptionServiceEvent::{event} for room {:?} without transcription state. \
                Stopping the service.",
                ctx.room,
            );

            ctx.send_ws_message(
                [participant_id],
                TranscriptionEvent::ServiceCommand {
                    command: TranscriptionServiceCommand::Stop,
                },
            )?;

            Ok(None)
        }
    }

    /// Handle events that were sent by the transcription service
    fn handle_transcription_service_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        service_event: TranscriptionServiceEvent,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if !is_transcription_service_for_current_room(ctx, participant_id)? {
            return Err(TranscriptionError::InsufficientPermissions.into());
        }

        tracing::debug!("Received transcription service event: {service_event:?}");

        match service_event {
            TranscriptionServiceEvent::Started {
                request_id,
                language,
            } => {
                let Some(state) = self.state_or_stop_service(ctx, participant_id, "Started")?
                else {
                    return Ok(());
                };

                if state.service != (ServiceState::Requested { request_id }) {
                    // The service belongs to an earlier request, e.g. the transcription was stopped
                    // and started again, or the automatic transcription was paused, before the
                    // service started.
                    ctx.send_ws_message(
                        [participant_id],
                        TranscriptionEvent::ServiceCommand {
                            command: TranscriptionServiceCommand::Stop,
                        },
                    )?;
                    return Ok(());
                }

                state.service = ServiceState::Running { participant_id };
                state.language = language;

                self.broadcast_status_updated(ctx)?;
            }
            TranscriptionServiceEvent::Updated {
                language: new_language,
            } => {
                let Some(state) = self.state_or_stop_service(ctx, participant_id, "Updated")?
                else {
                    return Ok(());
                };

                // Only accept updates from the currently running service
                if state.service.id().is_none_or(|id| id != participant_id) {
                    return Err(TranscriptionError::InsufficientPermissions.into());
                }

                state.language = new_language;

                self.broadcast_status_updated(ctx)?;
            }
            TranscriptionServiceEvent::Stopped => {
                let Some(state) = self.transcription_states.get_mut(&ctx.room) else {
                    // An untracked service stopped, the participants were already notified
                    return Ok(());
                };

                match state.service {
                    ServiceState::Running { participant_id: id } if id == participant_id => {
                        tracing::warn!("Transcription service stopped unexpectedly");
                        self.handle_service_error(ctx, TranscriptionError::ServiceDisconnected)?;
                    }
                    ServiceState::Stopping { participant_id: id } if id == participant_id => {
                        if state.modes.is_empty() {
                            self.transcription_states.remove(&ctx.room);
                        } else {
                            // This can be the case when the automatic transcription was paused
                            state.service = ServiceState::Stopped;
                        }
                        self.broadcast_status_updated(ctx)?;
                    }
                    _ => {} // an old service confirming it stopped, e.g. after resuming
                }
            }
            TranscriptionServiceEvent::Segment(transcription_segment) => {
                let Some(state) = self.state_or_stop_service(ctx, participant_id, "Segment")?
                else {
                    return Ok(());
                };

                // Only accept segments from the currently running service
                if state.service.id().is_none_or(|id| id != participant_id) {
                    return Err(TranscriptionError::InsufficientPermissions.into());
                }

                if let Some(automatic_transcription) = state.modes.automatic_transcription_mut()
                    && !automatic_transcription.paused()
                    && automatic_transcription
                        .consenting_participants
                        .contains(&transcription_segment.participant_id)
                {
                    automatic_transcription.push_segment(ctx, &transcription_segment);
                }

                if state.modes.live_transcription() {
                    ctx.send_ws_message(
                        ctx.participants.in_room(ctx.room).connected().ids(),
                        TranscriptionEvent::Segment(transcription_segment),
                    )?;
                }
            }
        };

        Ok(())
    }

    /// Handle a transcription service that is gone or could not be started.
    ///
    /// Recovers by removing the current transcription state to return to a clean slate. The
    /// automatic transcription is generated, so that no data is lost.
    fn handle_service_error(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        error: TranscriptionError,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        let Some(state) = self.transcription_states.get_mut(&ctx.room) else {
            // No state present, nothing to do.
            return Ok(());
        };

        let automatic_transcription = state.modes.deactivate_automatic_transcription(ctx);

        let state = self.transcription_states.remove(&ctx.room);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            error.into(),
        )?;
        self.send_modes_updated(ctx, ctx.participants.in_room(ctx.room).connected().ids())?;
        self.broadcast_status_updated(ctx)?;

        if let Some(automatic_transcription) = automatic_transcription {
            let transcription_language = state.and_then(|state| state.language);
            self.generate_transcript(
                ctx,
                transcription_language.as_deref(),
                automatic_transcription,
            )?;
        }

        Ok(())
    }

    fn change_language(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        language: Option<String>,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if !ctx.is_moderator(sender) {
            return Err(TranscriptionError::InsufficientPermissions.into());
        }

        let Some(state) = self.transcription_states.get_mut(&ctx.room) else {
            return Err(TranscriptionError::NotActive.into());
        };

        if let ServiceState::Running { participant_id } = state.service {
            // The language is updated once the service confirms the change
            ctx.send_ws_message(
                [participant_id],
                TranscriptionEvent::ServiceCommand {
                    command: TranscriptionServiceCommand::ChangeLanguage { language },
                },
            )?;

            Ok(())
        } else {
            Err(TranscriptionError::NotActive.into())
        }
    }

    fn set_consent(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        sender: ParticipantId,
        consent: bool,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        let Some(automatic_transcription) = self
            .transcription_states
            .get_mut(&ctx.room)
            .and_then(|state| state.modes.automatic_transcription_mut())
        else {
            return Err(TranscriptionError::NotActive.into());
        };

        let updated = if consent {
            automatic_transcription
                .consenting_participants
                .insert(sender)
        } else {
            automatic_transcription
                .consenting_participants
                .remove(&sender)
        };

        if updated {
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                TranscriptionEvent::AutomaticTranscriptionConsentUpdated {
                    participant: sender,
                    consents: consent,
                },
            )?;
        } else {
            // When the participants consent was already in the correct state, only notify the
            // sender
            ctx.send_ws_message(
                [sender],
                TranscriptionEvent::AutomaticTranscriptionConsentUpdated {
                    participant: sender,
                    consents: consent,
                },
            )?;
        };

        Ok(())
    }

    fn update_service(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        let action = self
            .transcription_states
            .get_mut(&ctx.room)
            .map_or(ServiceAction::None, TranscriptionRoomState::reconcile);

        tracing::debug!("Updating transcription service: {action:?}");

        match action {
            ServiceAction::Start {
                request_id,
                language,
            } => {
                self.request_transcription(ctx, request_id, language)?;
                self.broadcast_status_updated(ctx)?;
            }
            ServiceAction::Stop => self.stop_transcription_service(ctx)?,
            ServiceAction::Cancel => {
                // A service might be connected, but not have sent the `Started` event yet, so tell
                // it to stop.
                // Notify the participants immediately instead of waiting for the service's
                // `Stopped` reply. `reconcile` already  moved the service to `Stopped`, and the
                // service might not even be connected. The reply is ignored, because it cannot be
                // told apart from the `Stopped` of an old service after resuming.
                self.cancel_transcription_services(ctx)?;
                self.broadcast_status_updated(ctx)?;
            }
            ServiceAction::None => {}
        }

        // Remove the transcription state if there are no active modes left and there is no
        // stopping service to wait for
        if self
            .transcription_states
            .get(&ctx.room)
            .is_some_and(|state| state.modes.is_empty() && state.service == ServiceState::Stopped)
        {
            self.transcription_states.remove(&ctx.room);
        }

        Ok(())
    }

    /// Notify all participants in the room about the status of the transcription service.
    fn broadcast_status_updated(&self, ctx: &ModuleContext<'_, Self>) -> Result<(), FatalError> {
        let status = self.transcription_states.get(&ctx.room).map_or(
            TranscriptionStatus::Inactive,
            TranscriptionRoomState::status,
        );

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            TranscriptionEvent::StateUpdated { status },
        )
    }

    /// Notify the given participants about the active transcription modes.
    fn send_modes_updated(
        &self,
        ctx: &ModuleContext<'_, Self>,
        participant_ids: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<(), FatalError> {
        let modes = self
            .transcription_states
            .get(&ctx.room)
            .map(|state| TranscriptionModes::from(&state.modes))
            .unwrap_or_default();

        ctx.send_ws_message(participant_ids, TranscriptionEvent::ModesUpdated { modes })
    }

    fn build_module_state(
        &mut self,
        room: RoomKind,
        participant_id: ParticipantId,
    ) -> TranscriptionState {
        let Some(state) = self.transcription_states.get(&room) else {
            return TranscriptionState {
                status: TranscriptionStatus::Inactive,
                live_transcription: false,
                automatic_transcription: None,
            };
        };

        TranscriptionState {
            status: state.status(),
            live_transcription: state.modes.live_transcription(),
            automatic_transcription: state.modes.automatic_transcription().map(
                |automatic_transcription| AutomaticTranscriptionState {
                    config: automatic_transcription.config(),
                    consents: automatic_transcription
                        .consenting_participants
                        .contains(&participant_id),
                    paused: automatic_transcription.paused(),
                },
            ),
        }
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn record_transcript_presence(
        &mut self,
        ctx: &ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        let Some(automatic_transcription) = self
            .transcription_states
            .get_mut(&ctx.room)
            .and_then(|state| state.modes.automatic_transcription_mut())
        else {
            // Automatic transcription is not active, nothing to do.
            return Ok(());
        };

        if !ctx
            .participant_state(participant_id)
            .context("Participant has no state")?
            .is_visible()
        {
            // Participant should be visible in the transcript.
            return Ok(());
        }

        automatic_transcription
            .seen_participants
            .insert(participant_id);

        if ctx.is_moderator(participant_id) {
            automatic_transcription
                .seen_moderators
                .insert(participant_id);
        }

        Ok(())
    }

    fn build_peer_data_maps(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<PeerDataMaps, SignalingModuleError<TranscriptionError>> {
        let mut peer_data = PeerDataMap::default();
        let mut peer_events = PeerDataMap::default();

        if let Some(automatic_transcription) = self
            .transcription_states
            .get(&ctx.room)
            .and_then(|state| state.modes.automatic_transcription())
        {
            peer_data.insert_for_matching(
                ctx,
                TranscriptionPeerState {
                    consents_automatic_transcription: true,
                },
                |id, _| {
                    automatic_transcription
                        .consenting_participants
                        .contains(&id)
                },
            )?;
            peer_data.insert_for_matching(
                ctx,
                TranscriptionPeerState {
                    consents_automatic_transcription: false,
                },
                |id, _| {
                    !automatic_transcription
                        .consenting_participants
                        .contains(&id)
                },
            )?;

            peer_events.insert_for_all(
                ctx,
                TranscriptionPeerState {
                    consents_automatic_transcription: automatic_transcription
                        .consenting_participants
                        .contains(&participant_id),
                },
            )?;
        }

        Ok(PeerDataMaps {
            peer_data,
            peer_events,
        })
    }
}

struct PeerDataMaps {
    peer_events: PeerDataMap<TranscriptionModule>,
    peer_data: PeerDataMap<TranscriptionModule>,
}

/// Check if the given participant is the transcription service for the current room
fn is_transcription_service_for_current_room(
    ctx: &mut ModuleContext<'_, TranscriptionModule>,
    participant_id: ParticipantId,
) -> Result<bool, SignalingModuleError<TranscriptionError>> {
    let participant_state = ctx
        .participant_state(participant_id)
        .context("Failed to get state for disconnected participant")?;

    Ok(participant_state.kind == ClientKind::Transcription { room: ctx.room })
}

fn negotiate_languages<M: SignalingModule>(
    ctx: &ModuleContext<'_, M>,
    transcription_language: Option<&LanguageIdentifier>,
    available_languages: &[LanguageIdentifier],
) -> Option<LanguageIdentifier> {
    let user_language = &ctx.room_task_info.room.preferred_language;
    let system_default = &ctx.room_task_info.room.fallback_language;

    let mut languages: Vec<&LanguageIdentifier> = Vec::new();
    if let Some(language) = transcription_language {
        languages.push(language);
    }
    languages.extend_from_slice(&[user_language, system_default]);

    fluent_langneg::negotiate_languages(
        &languages,
        available_languages,
        None,
        NegotiationStrategy::Lookup,
    )
    .into_iter()
    .next()
    .cloned()
    .or_else(|| {
        tracing::warn!(
            "Could not find a valid report language. \
            System default: {system_default}, available: {available_languages:?}."
        );
        available_languages.first().cloned()
    })
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::TranscriptionModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&TranscriptionModule::example_config()).unwrap();

        assert_snapshot!(produced, @r#"
        [transcription]
        url = "http://localhost:11413/"

        [transcription.api_key]
        id = "transcription"
        secret = "secret"
        "#);
    }
}
