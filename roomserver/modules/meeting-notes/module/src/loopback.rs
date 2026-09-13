// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{collections::BTreeMap, sync::Arc};

use anyhow::{Context as _, anyhow};
use chrono::{Duration, Utc};
use futures::{StreamExt as _, TryStreamExt as _, stream};
use opentalk_etherpad_client::{EtherpadClient, EtherpadError};
use opentalk_roomserver_room::storage::assets::{AssetMetaData, AssetUploaded, ModuleAssetStorage};
use opentalk_roomserver_signaling::storage::assets::provider::AssetLoadError;
use opentalk_roomserver_types::{
    connection_id::ConnectionId,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_meeting_notes::MeetingNotesError;
use opentalk_types_common::{
    assets::{AssetFileKind, FileExtension, asset_file_kind},
    time::Timestamp,
    users::DisplayName,
};
use opentalk_types_signaling::ParticipantId;
use tracing::{Instrument, Span, instrument};

use crate::{CreateSession, InitState, PAD_NAME, SessionInfo, SessionUrl};

pub(super) const PARALLEL_UPDATES: usize = 25;

pub enum MeetingNotesLoopback {
    Initialized {
        group_id: String,
        results: BTreeMap<ConnectionId, Result<SessionUrl, GenerateUrlFailed>>,
    },
    WritersUpdated {
        results: BTreeMap<ConnectionId, Result<SessionUrl, GenerateUrlFailed>>,
    },
    PdfGenerated {
        asset: AssetUploaded,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct GenerateUrlFailed {
    pub participant_id: ParticipantId,
}

#[instrument(skip(client), level = "debug")]
pub(super) async fn initialize_etherpad(
    client: Arc<EtherpadClient>,
    mapped_id: String,
    participants: Vec<CreateSession>,
) -> Result<MeetingNotesLoopback, SignalingModuleError<MeetingNotesError>> {
    let group_id = client.create_group_for(mapped_id).await.map_err(|e| {
        tracing::error!("Failed to create group: {e}");
        MeetingNotesError::FailedInitialization
    })?;
    client
        .create_group_pad(&group_id, PAD_NAME, None)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create pad: {e}");
            MeetingNotesError::FailedInitialization
        })?;

    let sessions_expire = expires()?;
    let span = Span::current();

    let sessions = stream::iter(participants)
        .map(|create_session| {
            let client = Arc::clone(&client);
            let group_id = &group_id;
            let span = span.clone();
            async move {
                let id = create_session.connection_id;
                let CreateSession {
                    participant_id,
                    connection_id,
                    readonly,
                    display_name,
                    ..
                } = create_session;
                let result = generate_url_inner(
                    client,
                    participant_id,
                    connection_id,
                    readonly,
                    display_name,
                    group_id,
                    sessions_expire,
                )
                .instrument(span)
                .await
                .map_err(|e| into_generate_url_error(e, participant_id));
                (id, result)
            }
        })
        .buffer_unordered(PARALLEL_UPDATES)
        .collect()
        .await;

    Ok(MeetingNotesLoopback::Initialized {
        group_id,
        results: sessions,
    })
}

#[tracing::instrument(skip(client), level = "debug")]
pub(super) async fn generate_urls(
    client: Arc<EtherpadClient>,
    group_id: String,
    participants: Vec<CreateSession>,
) -> Result<MeetingNotesLoopback, SignalingModuleError<MeetingNotesError>> {
    let expires = expires()?;
    let span = Span::current();
    let result = stream::iter(participants)
        .then(|create_session| {
            let client = Arc::clone(&client);
            let group_id = group_id.clone();
            let span = span.clone();

            async move {
                let CreateSession {
                    participant_id,
                    connection_id,
                    readonly,
                    display_name,
                    existing_session_id,
                } = create_session;

                // Delete existing session if any
                if let Some(session_id) = existing_session_id {
                    // Returning a fatal error here terminates all streams and kills the module
                    delete_session(&client, session_id).await?;
                }

                Ok::<_, FatalError>(async move {
                    // Returning an error here sends into the loopback
                    let result = generate_url_inner(
                        client,
                        participant_id,
                        connection_id,
                        readonly,
                        display_name,
                        &group_id,
                        expires,
                    )
                    .await
                    .map_err(|e| into_generate_url_error(e, participant_id));
                    Ok((connection_id, result))
                })
            }
            .instrument(span)
        })
        .try_buffer_unordered(PARALLEL_UPDATES)
        .try_collect()
        .await?;

    Ok(MeetingNotesLoopback::WritersUpdated { results: result })
}

#[tracing::instrument(skip(client), level = "debug")]
pub(super) async fn generate_url(
    client: Arc<EtherpadClient>,
    participant_id: ParticipantId,
    connection_id: ConnectionId,
    readonly: bool,
    display_name: DisplayName,
    existing_session_id: Option<String>,
    group_id: String,
) -> Result<MeetingNotesLoopback, SignalingModuleError<MeetingNotesError>> {
    // Currently there is no proper session refresh in etherpad. Due to the difficulty of setting
    // new sessions on the client across domains, we set the expire duration to 14 days and hope
    // for the best.
    let duration = Duration::days(14);
    let expires = Utc::now()
        .checked_add_signed(duration)
        .ok_or(SignalingModuleError::Internal(anyhow!("DateTime overflow")))?
        .timestamp();

    if let Some(existing_session_id) = existing_session_id {
        delete_session(&client, existing_session_id).await?;
    }

    let result = generate_url_inner(
        client,
        participant_id,
        connection_id,
        readonly,
        display_name,
        &group_id,
        expires,
    )
    .await
    .map_err(|e| into_generate_url_error(e, participant_id));

    Ok(MeetingNotesLoopback::WritersUpdated {
        results: BTreeMap::from([(connection_id, result)]),
    })
}

fn into_generate_url_error(err: EtherpadError, participant_id: ParticipantId) -> GenerateUrlFailed {
    tracing::error!("Etherpad error: {err:?}");
    GenerateUrlFailed { participant_id }
}

async fn generate_url_inner(
    client: Arc<EtherpadClient>,
    participant_id: ParticipantId,
    connection_id: ConnectionId,
    readonly: bool,
    display_name: DisplayName,
    group_id: &str,
    expires: i64,
) -> Result<SessionUrl, EtherpadError> {
    // Create author if not exists
    let author_id = client
        .create_author_if_not_exits_for(&display_name.to_string(), &connection_id.to_string())
        .await?;

    // Create a new session
    let session_id = if readonly {
        client
            .create_read_session(group_id, &author_id, expires)
            .await?
    } else {
        client.create_session(group_id, &author_id, expires).await?
    };

    let url = client.auth_session_url(&session_id, PAD_NAME, Some(group_id))?;
    Ok(SessionUrl {
        session: SessionInfo {
            id: session_id,
            readonly,
        },
        participant_id,
        url,
    })
}

/// Deletes the specified `session_id` from etherpad.
///
/// Returns [`None`] when deletion of all sessions was successful, so that no loopback event is
/// produced. Returns a [`FatalError`] if deletion of any session failed. This is to ensure a
/// participant can't retain access to the etherpad after leaving the room.
#[tracing::instrument(skip(client), level = "debug")]
pub(crate) async fn delete_session(
    client: &EtherpadClient,
    session_id: String,
) -> Result<(), FatalError> {
    let Err(err) = client.delete_session(&session_id).await else {
        return Ok(());
    };

    let err = match err {
        EtherpadError::HttpRequest { source } => anyhow!("Etherpad unreachable: {source:?}"),
        EtherpadError::UrlParse { source } => anyhow!("Failed to parse URL: {source:?}"),
        // Etherpad returns an API error when the session does not exist, see https://github.com/ether/etherpad-lite/issues/5798.
        // We patch out this error in our container. So an API error here is unexpected.
        EtherpadError::ApiError { message } => anyhow!("Failed to delete session: {message}"),
        EtherpadError::EndpointError { endpoint, source } => {
            anyhow!("Etherpad endpoint {endpoint} returned an error: {source:?}")
        }
    };
    Err(FatalError(err))
}

/// Deletes the specified `session_ids` from etherpad.
///
/// Returns [`None`] when deletion of all sessions was successful, so that no loopback event is
/// produced. Returns a [`FatalError`] if deletion of any session failed. This is to ensure a
/// participant can't retain access to the etherpad after leaving the room.
#[tracing::instrument(skip(client), level = "debug")]
pub(crate) async fn delete_sessions(
    client: Arc<EtherpadClient>,
    session_ids: Vec<String>,
) -> Option<Result<MeetingNotesLoopback, SignalingModuleError<MeetingNotesError>>> {
    let span = Span::current();
    if let Err(e) = stream::iter(session_ids)
        .map(Ok::<_, SignalingModuleError<MeetingNotesError>>)
        .try_for_each_concurrent(PARALLEL_UPDATES, |session_id| {
            let client = Arc::clone(&client);
            let span = span.clone();
            async move {
                // Returning a fatal error here terminates all streams and kills the module
                delete_session(&client, session_id).instrument(span).await?;
                Ok(())
            }
        })
        .await
    {
        return Some(Err(e));
    }
    None
}

#[tracing::instrument(skip(client, rooms), level = "debug")]
pub(super) fn delete_pads(
    client: Arc<EtherpadClient>,
    rooms: impl Iterator<Item = InitState>,
) -> impl Future<Output = Vec<()>> {
    let group_ids = rooms.filter_map(|state| match state {
        InitState::Initialized { group_id, .. } => Some(group_id),
        InitState::Initializing => None,
    });
    let span = Span::current();
    stream::iter(group_ids)
        .map(move |group_id| {
            let client = Arc::clone(&client);
            let pad_id = crate::pad_name(&group_id);
            async move {
                match client.delete_pad(&pad_id).await {
                    Ok(()) => {
                        tracing::debug!("Successfully deleted etherpad pad '{pad_id}'");
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete etherpad pad '{pad_id}': {e}");
                    }
                }

                // Invalidate all sessions by deleting the group
                match client.delete_group(&group_id).await {
                    Ok(()) => {
                        tracing::debug!("Successfully deleted etherpad group '{group_id}'");
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete etherpad group '{group_id}': {e}");
                    }
                }
            }
            .instrument(span.clone())
        })
        .buffer_unordered(PARALLEL_UPDATES)
        .collect::<Vec<()>>()
}

#[tracing::instrument(skip(etherpad_client, storage_client, timestamp), level = "debug")]
pub(super) async fn generate_pdf(
    etherpad_client: Arc<EtherpadClient>,
    storage_client: ModuleAssetStorage,
    pad_id: String,
    session_id: String,
    timestamp: Timestamp,
) -> Result<MeetingNotesLoopback, SignalingModuleError<MeetingNotesError>> {
    const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("meetingnotes_pdf");

    let stream = etherpad_client
        .download_pdf(&session_id, &pad_id)
        .await
        // return an internal if the PDF can't be fetched from etherpad
        .context("Failed to create PDF")?
        .map_err(AssetLoadError::from);

    let metadata = AssetMetaData {
        kind: ASSET_FILE_KIND,
        timestamp,
        extension: FileExtension::pdf(),
    };

    let asset = storage_client
        .upload_asset(stream.boxed(), metadata)
        .await
        .map_err(MeetingNotesError::from)?;

    Ok(MeetingNotesLoopback::PdfGenerated { asset })
}

fn expires() -> Result<i64, SignalingModuleError<MeetingNotesError>> {
    let duration = Duration::minutes(5);
    let expires = Utc::now()
        .checked_add_signed(duration)
        .ok_or(SignalingModuleError::Internal(anyhow!("DateTime overflow")))?
        .timestamp();

    Ok(expires)
}
