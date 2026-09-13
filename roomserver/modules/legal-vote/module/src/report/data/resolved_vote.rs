// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_report_generation::ReportDateTime;
use opentalk_roomserver_types_legal_vote::vote::VoteOption;
use opentalk_types_common::users::DisplayName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedVote {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<DisplayName>,

    pub token: String,

    pub option: VoteOption,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<ReportDateTime>,
}
