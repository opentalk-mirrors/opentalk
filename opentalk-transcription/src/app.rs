// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{HashMap, hash_map::Entry},
    fs,
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use opentalk_client_signaling::{AuthConfig, OpenTalkClient, config::ClientConfig};
use opentalk_orchestrator_client::{
    TranscriptionEvent, TranscriptionResource, client::OrchestratorHandle,
};
use opentalk_transcription_web_api::v1::{TranscriptionBackend, TranscriptionTarget};
use opentalk_types_api_common::error::ApiError;
use tokio::sync::Mutex;
use whisper_rs::{WhisperContext, WhisperContextParameters};

use crate::{
    metrics,
    settings::Settings,
    tasks::ShutdownReceiver,
    transcription::{TranscriptionSession, TranscriptionSessionHandle},
};

/// The set of currently running transcription sessions, keyed by their target room.
pub(crate) type TranscriptionTasks =
    Arc<Mutex<HashMap<TranscriptionTarget, TranscriptionSessionHandle>>>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) settings: Arc<Settings>,
    /// The currently running transcription sessions.
    pub(crate) tasks: TranscriptionTasks,
    /// The shared whisper.cpp context that all sessions create their inference state from.
    pub(crate) whisper_context: Arc<WhisperContext>,
    /// The Prometheus metrics registry exposed on the `/metrics` endpoint.
    pub(crate) prometheus_state: Arc<metrics::PrometheusState>,
    /// The client used to communicate with the OpenTalk controller.
    pub(crate) opentalk_client: Arc<OpenTalkClient>,
    /// Handle to the orchestrator client, if the orchestrator is configured.
    pub(crate) orchestrator_handle: Option<OrchestratorHandle>,
    /// Receiver that is triggered when the service is shutting down.
    pub(crate) shutdown: ShutdownReceiver,
}

impl AppState {
    /// Creates the shared application state.
    ///
    /// This connects to the OpenTalk controller and loads the whisper model into a
    /// [`WhisperContext`], which is the expensive, long-lived resource shared by all sessions.
    ///
    /// # Errors
    ///
    /// Returns an error if the controller client cannot be created or if the configured model
    /// file does not exist or cannot be loaded.
    pub(crate) async fn new(
        settings: Arc<Settings>,
        transcription_tasks: TranscriptionTasks,
        orchestrator_handle: Option<OrchestratorHandle>,
        shutdown_signal: ShutdownReceiver,
    ) -> Result<Self> {
        let opentalk_client = OpenTalkClient::create(ClientConfig {
            auth: AuthConfig::ApiKey(settings.controller.api_key.clone()),
            controller: settings.controller.url.clone(),
        })
        .await?;

        if !fs::exists(&settings.transcription.model)
            .context("Failed to check if model path is valid")?
        {
            bail!(
                "Failed to load model '{}', file does not exist",
                settings.transcription.model
            )
        }

        let whisper_context = WhisperContext::new_with_params(
            &settings.transcription.model,
            WhisperContextParameters::default(),
        )
        .context("Failed to create whisper context")?;

        Ok(Self {
            settings,
            tasks: transcription_tasks,
            whisper_context: Arc::new(whisper_context),
            prometheus_state: Arc::new(metrics::PrometheusState::new()),
            opentalk_client: Arc::new(opentalk_client),
            orchestrator_handle,
            shutdown: shutdown_signal,
        })
    }
}

#[async_trait]
impl TranscriptionBackend for AppState {
    /// Starts a transcription session for the given target room.
    ///
    /// The session runs as a detached task that removes itself from [`AppState::tasks`] and
    /// notifies the orchestrator once it finishes. Returns a conflict error if a session is
    /// already running for the same target.
    async fn init(&self, target: TranscriptionTarget) -> Result<(), ApiError> {
        let context = Arc::new(self.clone());
        log::debug!("Start Transcription session {:?}", target.room_id);
        let (session, handle) = TranscriptionSession::create(context, target.clone())
            .await
            .map_err(|err| {
                log::error!("Failed to create transcription session: {err:?}");

                ApiError::internal()
            })?;

        let tasks = Arc::clone(&self.tasks);
        let orchestrator_handle = self.orchestrator_handle.clone();
        let transcription_target = target.clone();

        tokio::spawn(async move {
            match session.run().await {
                Ok(()) => {
                    log::debug!(
                        "Transcription session for room {} finished",
                        transcription_target.room_id
                    );
                }
                Err(e) => {
                    log::error!(
                        "Transcription session for room {} exited with error: {e:?}",
                        transcription_target.room_id
                    );
                }
            }

            if tasks.lock().await.remove(&transcription_target).is_none() {
                log::error!(
                    "Transcription session for room {:?} was not found in the task list",
                    transcription_target.room_id
                );
            }

            if let Some(orchestrator_handle) = orchestrator_handle
                && let Err(e) = orchestrator_handle
                    .send_event(TranscriptionEvent::RemoveTranscription(
                        TranscriptionResource {
                            room_id: transcription_target.room_id,
                            breakout_id: transcription_target.breakout_room,
                        },
                    ))
                    .await
            {
                log::error!(
                    "Failed to notify orchestrator about removed transcription session: {e:?}"
                );
            }
        });

        let mut tasks = self.tasks.lock().await;
        match tasks.entry(target) {
            Entry::Occupied(occupied_entry) => {
                log::error!(
                    "Received request to start transcription for room {:?} but a session is already running",
                    occupied_entry.key().room_id
                );

                handle.exit_task();

                return Err(ApiError::conflict()
                    .with_message("A transcription session is already running for this room"));
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(handle);
            }
        }

        Ok(())
    }
}
