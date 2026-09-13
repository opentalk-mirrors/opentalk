// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::Context;
use icu_locid::LanguageIdentifier;
use opentalk_roomserver_signaling::{
    module_context::ChannelDroppedError,
    storage::{
        assets::{AssetMetaData, AssetUploaded, ModuleAssetStorage},
        module_resources::ModuleResourceStorage,
    },
};
use opentalk_roomserver_types::signaling::module_error::{FatalError, SignalingModuleError};
use opentalk_roomserver_types_legal_vote::{
    event::LegalVoteError, user_parameters::UserParameters, vote::LegalVoteId,
};
use opentalk_types_api_internal::module_resources::ModuleResourceOperation;
use opentalk_types_common::{
    assets::{AssetFileKind, FileExtension, asset_file_kind},
    time::{TimeZone, Timestamp},
    users::{DisplayName, UserId},
};
use opentalk_types_signaling::ParticipantId;

use crate::{
    protocol::{NewProtocol, v1::ProtocolEntry},
    report,
};

const PROTOCOL_TAG: &str = "protocol";

pub enum LegalVoteLoopback {
    ResourceCreated {
        id: LegalVoteId,
        initiator_user_id: UserId,
        initiator_participant_id: ParticipantId,
        parameters: UserParameters,
    },
    VoteTimedOut {
        id: LegalVoteId,
    },
    VoteEnded,
    VoteCancelled,
    CreatedPdf {
        msg_target: ParticipantId,
        legal_vote_id: LegalVoteId,
        asset: AssetUploaded,
    },
    ChannelDropped,
    Error(SignalingModuleError<LegalVoteError>),
}

impl From<ChannelDroppedError> for LegalVoteLoopback {
    fn from(_: ChannelDroppedError) -> Self {
        Self::ChannelDropped
    }
}

impl From<SignalingModuleError<LegalVoteError>> for LegalVoteLoopback {
    fn from(e: SignalingModuleError<LegalVoteError>) -> Self {
        Self::Error(e)
    }
}

pub(super) async fn create_resource(
    resource_storage: ModuleResourceStorage,
    initiator_user_id: UserId,
    initiator_participant_id: ParticipantId,
    parameters: UserParameters,
) -> LegalVoteLoopback {
    // TODO grant users access to the resource once supported in the controller
    create_resource_inner(
        resource_storage,
        initiator_user_id,
        initiator_participant_id,
        parameters,
    )
    .await
    .unwrap_or_else(Into::into)
}

async fn create_resource_inner(
    resource_storage: ModuleResourceStorage,
    initiator_user_id: UserId,
    initiator_participant_id: ParticipantId,
    parameters: UserParameters,
) -> Result<LegalVoteLoopback, SignalingModuleError<LegalVoteError>> {
    let data = serde_json::to_value(NewProtocol::new(Vec::new()))
        .context("Serializing `NewProtocol` failed")
        .map_err(FatalError)?;

    let resource = resource_storage
        .create(initiator_user_id, Some(PROTOCOL_TAG.into()), data)
        .await?;

    Ok(LegalVoteLoopback::ResourceCreated {
        id: resource.id.into(),
        initiator_user_id,
        initiator_participant_id,
        parameters,
    })
}

pub(super) async fn upload_results(
    storage: ModuleResourceStorage,
    legal_vote_id: LegalVoteId,
    protocol: Vec<ProtocolEntry>,
) -> Result<(), SignalingModuleError<LegalVoteError>> {
    let protocol = NewProtocol::new(protocol);
    let protocol = serde_json::to_value(protocol)
        .context("Failed to serialize protocol")
        .map_err(FatalError)?;
    let add_operation = ModuleResourceOperation::Add {
        path: "".into(),
        value: protocol,
    };

    storage
        .patch(legal_vote_id.into_inner(), vec![add_operation])
        .await
        .context("Failed to upload results")?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn generate_pdf(
    storage: ModuleAssetStorage,
    legal_vote_id: LegalVoteId,
    msg_target: ParticipantId,
    time_zone: TimeZone,
    timestamp: Timestamp,
    protocol: Vec<ProtocolEntry>,
    user_names: BTreeMap<UserId, DisplayName>,
    report_language: LanguageIdentifier,
    typst_package_path: PathBuf,
) -> LegalVoteLoopback {
    generate_pdf_inner(
        storage,
        legal_vote_id,
        msg_target,
        time_zone,
        timestamp,
        protocol,
        user_names,
        report_language,
        &typst_package_path,
    )
    .await
    .unwrap_or_else(Into::into)
}

#[allow(clippy::too_many_arguments)]
async fn generate_pdf_inner(
    storage: ModuleAssetStorage,
    legal_vote_id: LegalVoteId,
    msg_target: ParticipantId,
    time_zone: TimeZone,
    timestamp: Timestamp,
    protocol: Vec<ProtocolEntry>,
    user_names: BTreeMap<UserId, DisplayName>,
    report_language: LanguageIdentifier,
    typst_package_path: &Path,
) -> Result<LegalVoteLoopback, SignalingModuleError<LegalVoteError>> {
    const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("vote_protocol");

    let bytes = report::generate(
        user_names,
        protocol,
        time_zone.into(),
        report_language,
        typst_package_path,
    )
    .context("Failed to generate legal vote report")?;

    let metadata = AssetMetaData {
        kind: ASSET_FILE_KIND,
        timestamp,
        extension: FileExtension::pdf(),
    };
    let asset = storage
        .upload_asset_vec(bytes, metadata)
        .await
        .map_err(LegalVoteError::from)?;

    Ok(LegalVoteLoopback::CreatedPdf {
        msg_target,
        legal_vote_id,
        asset,
    })
}
