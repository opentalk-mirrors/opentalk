// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_report_generation::ReportDateTime;
use opentalk_roomserver_types_legal_vote::{user_parameters::Duration, vote::LegalVoteId};
use opentalk_types_common::{time::TimeZone, users::DisplayName};
use serde::{Deserialize, Serialize};

use super::StopReason;
use crate::protocol::v1::FinalResults;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    pub pseudonymous: bool,

    pub creator: DisplayName,

    pub id: LegalVoteId,

    pub start_time: ReportDateTime,

    pub participant_count: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,

    pub enable_abstain: bool,

    pub auto_close: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<ReportDateTime>,

    pub stop_reason: StopReason,

    pub vote_count: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_results: Option<FinalResults>,

    pub report_timezone: TimeZone,
}
