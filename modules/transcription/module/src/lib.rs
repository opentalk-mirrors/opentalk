// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use anyhow::Context;
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    participant_state::ParticipantState,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
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
    event::{TranscriptionError, TranscriptionEvent},
    service::{command::TranscriptionServiceCommand, event::TranscriptionServiceEvent},
    settings::TranscriptionSettings,
    state::{TranscriptionState, TranscriptionStatus},
};
use opentalk_transcription_web_api::v1::TranscriptionTarget;
use opentalk_types_common::{features::ModuleFeatureId, modules::ModuleId};
use opentalk_types_signaling::ParticipantId;

pub struct TranscriptionModule {
    settings: TranscriptionSettings,
    http_client: reqwest::Client,
    transcription_states: BTreeMap<RoomKind, TranscriptionRoomState>,
}

/// Internal per-room transcription state
#[derive(Debug, Clone, PartialEq, Eq)]
enum TranscriptionRoomState {
    Requested { language: Option<String> },
    Running { language: Option<String> },
}

pub enum LoopBackEvent {
    TranscriptionRequestFailed,
}

impl SignalingModuleDescription for TranscriptionModule {
    const MODULE_ID: ModuleId = TRANSCRIPTION_MODULE_ID;
    const DESCRIPTION: &'static str = "Live transcription for meetings";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[SignalingModuleFeatureDescription {
        feature_id: TRANSCRIPTION_FEATURE_ID,
        description: "Allows to create transcriptions for meetings",
    }];
}

impl SignalingModule for TranscriptionModule {
    const NAMESPACE: ModuleId = TRANSCRIPTION_MODULE_ID;

    type Incoming = TranscriptionCommand;
    type Outgoing = TranscriptionEvent;
    type Internal = NoOp;
    type Loopback = LoopBackEvent;
    type JoinInfo = TranscriptionState;
    type PeerJoinInfo = ();
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

        Some(Self {
            settings,
            http_client: reqwest::Client::new(),
            transcription_states: BTreeMap::new(),
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        Ok(ModuleJoinData {
            join_success: Some(self.build_module_state(ctx.room)),
            ..Default::default()
        })
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        if is_transcription_service_for_current_room(ctx, participant_id)?
            && self.transcription_states.remove(&ctx.room).is_some()
        {
            // We had an active transcription but the transcription service disconnected
            // unexpectedly
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                TranscriptionError::ServiceDisconnected.into(),
            )?;

            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                TranscriptionEvent::StateUpdated {
                    status: TranscriptionStatus::Inactive,
                },
            )?;
        }

        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        command: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match command {
            TranscriptionCommand::Start { language } => {
                self.start_transcription(ctx, participant_id, language)
            }
            TranscriptionCommand::Stop => {
                if !ctx.is_moderator(participant_id) {
                    return Err(TranscriptionError::InsufficientPermissions.into());
                }

                self.stop_transcription(ctx)
            }
            TranscriptionCommand::ChangeLanguage { language } => {
                if !ctx.is_moderator(participant_id) {
                    return Err(TranscriptionError::InsufficientPermissions.into());
                }

                self.change_language(ctx, language)
            }
            TranscriptionCommand::TranscriptionServiceEvent { event } => {
                self.handle_transcription_service_event(ctx, event, participant_id)
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
        let state = self.build_module_state(new_room);

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

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            LoopBackEvent::TranscriptionRequestFailed => {
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    TranscriptionError::ServiceRequestFailed.into(),
                )?;

                self.stop_transcription(ctx)?;
            }
        }

        Ok(())
    }
}

impl TranscriptionModule {
    fn start_transcription(
        &mut self,
        ctx: &mut ModuleContext<'_, TranscriptionModule>,
        participant_id: ParticipantId,
        language: Option<String>,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if !ctx.is_moderator(participant_id) {
            return Err(TranscriptionError::InsufficientPermissions.into());
        }

        if self.transcription_states.contains_key(&ctx.room) {
            return Err(TranscriptionError::AlreadyActive.into());
        };

        self.request_transcription(ctx, language.clone())?;

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            TranscriptionEvent::StateUpdated {
                status: TranscriptionStatus::Requested { language },
            },
        )?;

        Ok(())
    }

    fn request_transcription(
        &mut self,
        ctx: &mut ModuleContext<'_, TranscriptionModule>,
        language: Option<String>,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        let transcription_config = self.settings.clone();

        let jwt_bearer_token = transcription_config
            .api_key
            .generate_jwt()
            .context("Failed to generate JWT from transcription api_key")
            .map_err(FatalError)?;

        let http_client = self.http_client.clone();

        let body = TranscriptionTarget {
            room_id: ctx.room_id,
            breakout_room: match ctx.room {
                RoomKind::Main => None,
                RoomKind::Breakout(breakout_id) => Some(breakout_id.into()),
            },
            language: language.clone(),
        };

        ctx.spawn_optional(async move {
            let url = match transcription_config.url.join("v1/init") {
                Ok(url) => url,
                Err(err) => {
                    tracing::error!(
                        "Failed to build transcription init url from base_url: {}, {err}",
                        transcription_config.url,
                    );

                    return Some(LoopBackEvent::TranscriptionRequestFailed);
                }
            };

            tracing::debug!("Sending transcription start request to {url} with body {body:#?}");

            let response = http_client
                .post(url)
                .bearer_auth(jwt_bearer_token)
                .json(&body)
                .send()
                .await;

            let response = match response {
                Ok(response) => response,
                Err(err) => {
                    tracing::error!(
                        "Failed to send init request to transcription service, {err:?}"
                    );
                    return Some(LoopBackEvent::TranscriptionRequestFailed);
                }
            };

            let response_status = response.status();
            let response_body = response
                .bytes()
                .await
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());

            if response_status.is_success() {
                tracing::debug!(
                    "Got transcription start response status={} body={:#?}",
                    response_status,
                    response_body,
                );

                None
            } else {
                tracing::error!(
                    "Got non-success response to transcription start request status={} body={:#?}",
                    response_status,
                    response_body,
                );
                Some(LoopBackEvent::TranscriptionRequestFailed)
            }
        });

        self.transcription_states
            .insert(ctx.room, TranscriptionRoomState::Requested { language });
        Ok(())
    }

    /// Send a stop command to the transcription service if the transcription state is requested or
    /// active
    ///
    /// Once the transcription service stops and disconnects, other participants will receive the
    /// `Inactive` state update
    fn stop_transcription(
        &mut self,
        ctx: &mut ModuleContext<'_, TranscriptionModule>,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        if let Some((transcription, _)) = find_transcription_service(ctx) {
            ctx.send_ws_message(
                [*transcription],
                TranscriptionEvent::ServiceCommand {
                    command: TranscriptionServiceCommand::Stop,
                },
            )?;
        } else {
            // The transcription service is not yet connected, but a stop command was issued
            //
            // Remove the transcription state so that if the transcription service connects later it
            // will notice the missing state and immediately disconnect without starting
            // the transcription
            if self.transcription_states.remove(&ctx.room).is_none() {
                return Err(TranscriptionError::NotActive.into());
            };

            // In this scenario, we don't need to wait until the transcription service disconnects
            // to update the state for other participants
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                TranscriptionEvent::StateUpdated {
                    status: TranscriptionStatus::Inactive,
                },
            )?;
        }

        Ok(())
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

        match service_event {
            TranscriptionServiceEvent::Started { language } => {
                self.transcription_states.insert(
                    ctx.room,
                    TranscriptionRoomState::Running {
                        language: language.clone(),
                    },
                );

                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    TranscriptionEvent::StateUpdated {
                        status: TranscriptionStatus::Running { language },
                    },
                )?;
            }
            TranscriptionServiceEvent::Updated {
                language: new_language,
            } => match self.transcription_states.get_mut(&ctx.room) {
                Some(TranscriptionRoomState::Requested { language }) => {
                    *language = new_language.clone();

                    ctx.send_ws_message(
                        ctx.participants.in_room(ctx.room).connected().ids(),
                        TranscriptionEvent::StateUpdated {
                            status: TranscriptionStatus::Requested {
                                language: new_language,
                            },
                        },
                    )?;
                }
                Some(TranscriptionRoomState::Running { language }) => {
                    *language = new_language.clone();

                    ctx.send_ws_message(
                        ctx.participants.in_room(ctx.room).connected().ids(),
                        TranscriptionEvent::StateUpdated {
                            status: TranscriptionStatus::Running {
                                language: new_language,
                            },
                        },
                    )?;
                }
                None => {
                    tracing::warn!(
                        "Got TranscriptionServiceEvent::Updated for room {:?} without transcription state",
                        ctx.room
                    );
                }
            },
            TranscriptionServiceEvent::Stopped => {
                self.transcription_states.remove(&ctx.room);

                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    TranscriptionEvent::StateUpdated {
                        status: TranscriptionStatus::Inactive,
                    },
                )?;
            }
            TranscriptionServiceEvent::Segment(transcription_segment) => {
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    TranscriptionEvent::Segment(transcription_segment),
                )?;
            }
        };

        Ok(())
    }

    fn change_language(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        language: Option<String>,
    ) -> Result<(), SignalingModuleError<TranscriptionError>> {
        let Some((transcription, _)) = find_transcription_service(ctx) else {
            return Err(TranscriptionError::NotActive.into());
        };

        ctx.send_ws_message(
            [*transcription],
            TranscriptionEvent::ServiceCommand {
                command: TranscriptionServiceCommand::ChangeLanguage { language },
            },
        )?;

        Ok(())
    }

    fn build_module_state(&mut self, room: RoomKind) -> TranscriptionState {
        let status = match self.transcription_states.get(&room).cloned() {
            Some(TranscriptionRoomState::Requested { language }) => {
                TranscriptionStatus::Requested { language }
            }
            Some(TranscriptionRoomState::Running { language }) => {
                TranscriptionStatus::Running { language }
            }
            None => TranscriptionStatus::Inactive,
        };

        TranscriptionState { status }
    }
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

/// Find the connected transcription service participant for the current room
fn find_transcription_service<'a>(
    ctx: &'a ModuleContext<'a, TranscriptionModule>,
) -> Option<(&'a ParticipantId, &'a ParticipantState)> {
    ctx.participants.in_room(ctx.room).iter().find(|(_, p)| {
        p.is_connected() && matches!(p.kind, ClientKind::Transcription { room } if room == ctx.room)
    })
}
