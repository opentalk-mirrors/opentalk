// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_report_generation::ReportDateTime;
use serde::Serialize;

use crate::report::data::StopReason;

#[derive(Debug, Serialize)]
pub struct StopInfo {
    pub time: Option<ReportDateTime>,
    pub reason: StopReason,
}
