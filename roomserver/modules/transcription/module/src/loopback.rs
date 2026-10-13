// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use icu_locid::{LanguageIdentifier, subtags::Language};
use opentalk_report_generation::GenerateOptions;
use opentalk_roomserver_room::storage::assets::{AssetMetaData, AssetUploaded, ModuleAssetStorage};
use opentalk_roomserver_types_transcription::settings::TranscriptionSettings;
use opentalk_transcription_web_api::v1::TranscriptionTarget;
use opentalk_types_common::{
    assets::{AssetFileKind, FileExtension, asset_file_kind},
    events::EventTitle,
    time::Timestamp,
};
use reqwest::Client;

use crate::{
    AVAILABLE_LANGUAGES, state::protocol::FinishedMeetingProtocol,
    template::ReportTemplateParameter,
};

const PROTOCOL_TEMPLATE: &str = include_str!("../templates/meeting_protocol.typ");
const FTL_EN: &str = include_str!("../templates/l10n/en.ftl");
const FTL_DE: &str = include_str!("../templates/l10n/de.ftl");

pub enum LoopBackEvent {
    TranscriptionRequestFailed,
    MeetingProtocolGenerated(AssetUploaded),
    MeetingProtocolGenerationFailed,
}

#[tracing::instrument(
    skip(transcription_settings, http_client, jwt_bearer_token),
    level = "debug"
)]
pub(super) async fn request_transcription(
    transcription_settings: TranscriptionSettings,
    body: TranscriptionTarget,
    http_client: Client,
    jwt_bearer_token: String,
) -> Option<LoopBackEvent> {
    let url = match transcription_settings.url.join("v1/init") {
        Ok(url) => url,
        Err(err) => {
            tracing::error!(
                "Failed to build transcription init url from base_url: {}, {err}",
                transcription_settings.url,
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
            tracing::error!("Failed to send init request to transcription service, {err:?}");
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
}

struct CreateReportError;

impl From<CreateReportError> for LoopBackEvent {
    fn from(_: CreateReportError) -> Self {
        Self::MeetingProtocolGenerationFailed
    }
}

pub(super) async fn create_protocol(
    storage: ModuleAssetStorage,
    title: Option<EventTitle>,
    report_language: LanguageIdentifier,
    transcription_language: Option<Language>,
    protocol: FinishedMeetingProtocol,
    typst_package_path: PathBuf,
) -> LoopBackEvent {
    let FinishedMeetingProtocol {
        started_at,
        ended_at,
        present_participant_count,
        moderators,
        pseudonymous,
        protocol,
    } = protocol;

    let template_parameter = ReportTemplateParameter {
        available_languages: AVAILABLE_LANGUAGES.to_vec(),
        report_language,
        transcription_language,
        title,
        started_at,
        ended_at,
        present_participant_count,
        moderators,
        pseudonymous,
        protocol,
    };

    create_protocol_inner(storage, template_parameter, &typst_package_path)
        .await
        .map(LoopBackEvent::MeetingProtocolGenerated)
        .unwrap_or_else(Into::into)
}

async fn create_protocol_inner(
    storage: ModuleAssetStorage,
    template_parameter: ReportTemplateParameter,
    typst_package_path: &Path,
) -> Result<AssetUploaded, CreateReportError> {
    const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("meeting_protocol");

    let mut generate_options = GenerateOptions::default();
    generate_options.packages_path = Some(typst_package_path);

    let serialized = serde_json::to_string(&template_parameter)
        .map_err(|err| {
            tracing::error!("Serializing parameters failed: {err}");
            CreateReportError
        })?
        .into_bytes()
        .into();
    let files = [
        ("data.json", (None, serialized)),
        ("l10n/de.ftl", (None, FTL_DE.as_bytes().into())),
        ("l10n/en.ftl", (None, FTL_EN.as_bytes().into())),
    ];
    let protocol = opentalk_report_generation::generate_pdf_report(
        PROTOCOL_TEMPLATE.to_owned(),
        BTreeMap::from_iter(files),
        &generate_options,
    )
    .map_err(|err| {
        tracing::error!("Failed to generate protocol: {err}");
        CreateReportError
    })?;

    storage
        .upload_asset_vec(
            protocol,
            AssetMetaData {
                kind: ASSET_FILE_KIND,
                timestamp: Timestamp::now(),
                extension: FileExtension::pdf(),
            },
        )
        .await
        .map_err(|err| {
            tracing::error!("Uploading protocol failed: {err:?}");
            CreateReportError
        })
}
