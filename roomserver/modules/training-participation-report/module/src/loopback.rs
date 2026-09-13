// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use opentalk_report_generation::GenerateOptions;
use opentalk_roomserver_signaling::{
    module_context::ChannelDroppedError,
    storage::assets::{AssetMetaData, AssetUploaded, ModuleAssetStorage},
};
use opentalk_roomserver_types_training_participation_report::event::TrainingParticipationReportError;
use opentalk_types_common::{
    assets::{AssetFileKind, FileExtension, asset_file_kind},
    time::Timestamp,
};

use crate::ReportTemplateParameter;

const REPORT_TEMPLATE: &str = include_str!("../templates/training_participation_report.typ");
const FTL_EN: &str = include_str!("../templates/l10n/en.ftl");
const FTL_DE: &str = include_str!("../templates/l10n/de.ftl");

pub enum TrainingParticipationReportLoopback {
    CheckpointReached(Timestamp),
    CheckpointCanceled,
    ReportUploaded(AssetUploaded),
    ChannelDropped,
    Error(TrainingParticipationReportError),
}

impl From<ChannelDroppedError> for TrainingParticipationReportLoopback {
    fn from(_: ChannelDroppedError) -> Self {
        Self::ChannelDropped
    }
}

impl From<TrainingParticipationReportError> for TrainingParticipationReportLoopback {
    fn from(err: TrainingParticipationReportError) -> Self {
        Self::Error(err)
    }
}

/// Create a training participation report and upload it to the asset storage.
#[tracing::instrument(skip(storage), level = "debug")]
pub(super) async fn create_report(
    storage: ModuleAssetStorage,
    template_parameter: ReportTemplateParameter,
    typst_package_path: PathBuf,
) -> TrainingParticipationReportLoopback {
    create_report_inner(storage, template_parameter, &typst_package_path)
        .await
        .unwrap_or_else(Into::into)
}

async fn create_report_inner(
    storage: ModuleAssetStorage,
    template_parameter: ReportTemplateParameter,
    typst_package_path: &Path,
) -> Result<TrainingParticipationReportLoopback, TrainingParticipationReportError> {
    const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("training_participation_report");

    let mut generate_options = GenerateOptions::default();
    generate_options.packages_path = Some(typst_package_path);

    let serialized = serde_json::to_string(&template_parameter)
        .inspect_err(|e| tracing::error!("Serializing parameters failed: {e}"))
        .map_err(|_| TrainingParticipationReportError::Generate)?
        .into_bytes()
        .into();
    let files = [
        ("data.json", (None, serialized)),
        ("l10n/de.ftl", (None, FTL_DE.as_bytes().into())),
        ("l10n/en.ftl", (None, FTL_EN.as_bytes().into())),
    ];
    let report = opentalk_report_generation::generate_pdf_report(
        REPORT_TEMPLATE.to_owned(),
        BTreeMap::from_iter(files),
        &generate_options,
    )
    .map_err(|_| TrainingParticipationReportError::Generate)?;

    let upload = storage
        .upload_asset_vec(
            report,
            AssetMetaData {
                kind: ASSET_FILE_KIND,
                timestamp: Timestamp::now(),
                extension: FileExtension::pdf(),
            },
        )
        .await
        .inspect_err(|e| tracing::error!("Uploading report failed: {e:?}"))
        .map_err(TrainingParticipationReportError::from)?;

    Ok(TrainingParticipationReportLoopback::ReportUploaded(upload))
}
