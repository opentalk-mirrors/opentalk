// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use chrono_tz::Tz;
use icu_locid::{LanguageIdentifier, langid};
use opentalk_report_generation::{ReportDateTime, ToReportDateTime as _};
use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    time::{TimeZone, Timestamp},
    users::DisplayName,
};
use opentalk_types_signaling::ParticipantId;
use serde::Serialize;

use crate::Checkpoint;

const AVAILABLE_LANGUAGES: &[LanguageIdentifier] = &[langid!("en"), langid!("de")];

/// Struct containing all parameters required for rendering the report from the template.
#[derive(Debug, Serialize)]
pub(crate) struct ReportTemplateParameter {
    pub available_languages: Vec<LanguageIdentifier>,
    pub title: EventTitle,
    pub description: EventDescription,
    pub start: ReportDateTime,
    pub end: ReportDateTime,
    pub report_timezone: TimeZone,
    pub report_language: LanguageIdentifier,
    pub participants: HashMap<ParticipantId, Option<DisplayName>>,
    pub checkpoints: Vec<ReportCheckpoint>,
}

impl ReportTemplateParameter {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        title: EventTitle,
        description: EventDescription,
        report_time_zone: TimeZone,
        report_language: LanguageIdentifier,
        start: Timestamp,
        end: Timestamp,
        participants: HashMap<ParticipantId, Option<DisplayName>>,
        checkpoints: Vec<Checkpoint>,
    ) -> Self {
        Self {
            available_languages: AVAILABLE_LANGUAGES.to_vec(),
            title,
            description,
            start: start.to_report_date_time(&report_time_zone),
            end: end.to_report_date_time(&report_time_zone),
            report_timezone: report_time_zone,
            report_language,
            participants,
            checkpoints: checkpoints
                .iter()
                .map(|c| ReportCheckpoint::from_checkpoint(c, &report_time_zone))
                .collect(),
        }
    }
}

/// Representation of a [`Checkpoint`] suitable for report generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ReportCheckpoint {
    pub timestamp: ReportDateTime,
    pub presence: HashMap<ParticipantId, ReportDateTime>,
}

impl ReportCheckpoint {
    pub fn from_checkpoint(
        Checkpoint {
            timestamp,
            presence,
        }: &Checkpoint,
        report_tz: &Tz,
    ) -> Self {
        Self {
            timestamp: timestamp.to_report_date_time(report_tz),
            presence: presence
                .iter()
                .map(|(participant, timestamp)| {
                    (*participant, timestamp.to_report_date_time(report_tz))
                })
                .collect(),
        }
    }
}
