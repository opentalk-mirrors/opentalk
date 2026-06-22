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
use tokio::{sync::Mutex, task::JoinHandle};
use whisper_rs::{WhisperContext, WhisperContextParameters};

use crate::{
    metrics, settings::Settings, tasks::ShutdownReceiver, transcription::TranscriptionSession,
};

/// The set of currently running transcription sessions, keyed by their target room.
pub(crate) type TranscriptionTasks =
    Arc<Mutex<HashMap<TranscriptionTarget, JoinHandle<Result<()>>>>>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) tasks: TranscriptionTasks,
    pub(crate) whisper_context: Arc<WhisperContext>,
    pub(crate) prometheus_state: Arc<metrics::PrometheusState>,
    pub(crate) opentalk_client: Arc<OpenTalkClient>,
    pub(crate) orchestrator_handle: Option<OrchestratorHandle>,
    pub(crate) shutdown: ShutdownReceiver,
}

impl AppState {
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
    async fn init(&self, target: TranscriptionTarget) -> Result<(), ApiError> {
        let context = Arc::new(self.clone());
        log::debug!("Start Transcription session {:?}", target.room_id);
        let session = TranscriptionSession::create(context, target.clone())
            .await
            .map_err(|err| {
                log::error!("Failed to create transcription session: {err:?}");

                ApiError::internal()
            })?;

        let tasks = Arc::clone(&self.tasks);
        let orchestrator_handle = self.orchestrator_handle.clone();
        let transcription_target = target.clone();

        let task = tokio::spawn(async move {
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

            if let Some(orchestrator_handle) = orchestrator_handle {
                if let Err(e) = orchestrator_handle
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
            }

            Ok(())
        });

        let mut tasks = self.tasks.lock().await;

        match tasks.entry(target) {
            Entry::Occupied(occupied_entry) => {
                log::error!(
                    "Received request to start transcription for room {:?} but a session is already running",
                    occupied_entry.key().room_id
                );
                task.abort();

                return Err(ApiError::conflict()
                    .with_message("A transcription session is already running for this room"));
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(task);
            }
        }

        Ok(())
    }
}
