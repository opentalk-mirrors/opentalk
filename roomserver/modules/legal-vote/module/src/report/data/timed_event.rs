// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_report_generation::ReportDateTime;
use serde::{Deserialize, Serialize};

use super::Event;

/// An event that was caused by a user. For example when a participant joined, left or reported an
/// issue
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimedEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<ReportDateTime>,

    #[serde(flatten)]
    pub event: Event,
}
