// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_report_generation::ReportGenerationError;
use opentalk_types_common::users::UserId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to generate the report")]
    ReportGeneration(#[from] ReportGenerationError),

    #[error("The legal vote protocol is missing the start entry")]
    MissingStartEntry,

    #[error("The legal vote protocol is missing the stop entry")]
    MissingStopEntry,

    #[error("Display name for user id {user_id} not found")]
    UserDisplayNameNotFound { user_id: UserId },
}
