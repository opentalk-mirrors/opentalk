// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use chrono_tz::Tz;
use icu_locid::LanguageIdentifier;
use opentalk_report_generation::ToReportDateTime as _;
use opentalk_types_common::users::{DisplayName, UserId};

use super::StopInfo;
use crate::{
    protocol::v1::FinalResults,
    report::{
        AVAILABLE_LANGUAGES, Error,
        data::{ReportData, ResolvedVote, Summary, TimedEvent},
        report_data_builder::start::Start,
    },
};

#[derive(Debug, Default)]
pub struct VoteData {
    pub start: Option<Start>,
    pub stop_info: Option<StopInfo>,
    pub final_results: Option<FinalResults>,
    pub votes: Vec<ResolvedVote>,
    pub events: Vec<TimedEvent>,
}

impl VoteData {
    pub(super) fn finalize(
        self,
        user_names: &BTreeMap<UserId, DisplayName>,
        timezone: Tz,
        report_language: LanguageIdentifier,
    ) -> Result<ReportData, Error> {
        let VoteData {
            start,
            stop_info,
            final_results,
            votes,
            events,
        } = self;

        let Some(start) = start else {
            return Err(Error::MissingStartEntry);
        };

        let Some(stop_info) = stop_info else {
            return Err(Error::MissingStopEntry);
        };

        let summary = Summary {
            title: start.parameters.inner.name.to_string(),
            subtitle: start
                .parameters
                .inner
                .subtitle
                .map(|subtitle| subtitle.to_string()),
            topic: start.parameters.inner.topic.map(|topic| topic.to_string()),
            pseudonymous: start.parameters.inner.pseudonymous,
            creator: user_names
                .get(&start.issuer)
                .ok_or(Error::UserDisplayNameNotFound {
                    user_id: start.issuer,
                })?
                .clone(),
            id: start.parameters.legal_vote_id,
            start_time: start.parameters.start_time.to_report_date_time(&timezone),
            participant_count: start.parameters.max_votes,
            duration: start.parameters.inner.duration,
            enable_abstain: start.parameters.inner.enable_abstain,
            auto_close: start.parameters.inner.auto_close,
            end_time: stop_info.time,
            stop_reason: stop_info.reason,
            vote_count: votes.len() as u32,
            final_results,
            report_timezone: timezone.into(),
        };

        Ok(ReportData {
            available_languages: AVAILABLE_LANGUAGES.to_vec(),
            summary,
            votes,
            events,
            report_language,
        })
    }
}
