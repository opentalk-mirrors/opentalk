// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling state for the `training_participation_report` namespace

use opentalk_types_common::training_participation_report::TrainingParticipationReportParameterSet;
use serde::{Deserialize, Serialize};

/// The state of the `training_participation_report` module
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrainingParticipationReportState {
    /// Current state of the participation logging procedure
    pub state: ParticipationLoggingState,

    /// The default parameters of the room. Only communicated to the room owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<TrainingParticipationReportParameterSet>,
}

impl opentalk_types_signaling::SignalingModuleFrontendData for TrainingParticipationReportState {
    const NAMESPACE: Option<opentalk_types_common::modules::ModuleId> =
        Some(crate::TRAINING_PARTICIPATION_REPORT_MODULE_ID);
}

/// The state of the participation logging procedure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipationLoggingState {
    /// No participation logging is active, nothing to do for a client.
    Disabled,

    /// Participation logging is enabled, either waiting for the initial timeout
    /// or the participant already confirmed the last checkpoint. A client
    /// should notify the participant about this state.
    Enabled,

    /// Participation logging is enabled, a checkpoint has already been passed
    /// and the newly joined participant can immediately confirm their presence.
    WaitingForConfirmation,
}
