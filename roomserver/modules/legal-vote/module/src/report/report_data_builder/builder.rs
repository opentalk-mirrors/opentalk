// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeMap;

use chrono_tz::Tz;
use icu_locid::LanguageIdentifier;
use opentalk_report_generation::{ReportDateTime, ToReportDateTime as _};
use opentalk_roomserver_types_legal_vote::{
    cancel::CancelReason, issue::Issue, parameters::Parameters, token::Token, vote::VoteOption,
};
use opentalk_types_common::users::{DisplayName, UserId};

use super::{StopInfo, VoteData};
use crate::{
    protocol::v1::{FinalResults, ProtocolEntry, StopKind, UserInfo, VoteEvent},
    report::{
        Error,
        data::{Event, ReportData, ResolvedCancel, ResolvedVote, StopReason, TimedEvent},
        report_data_builder::start::Start,
    },
};

pub struct Builder {
    user_names: BTreeMap<UserId, DisplayName>,
    data: VoteData,
}

impl Builder {
    pub(crate) fn new(user_names: BTreeMap<UserId, DisplayName>) -> Self {
        Self {
            user_names,
            data: VoteData::default(),
        }
    }

    pub(crate) fn build_report_data(
        mut self,
        protocol: Vec<ProtocolEntry>,
        timezone: Tz,
        report_language: LanguageIdentifier,
    ) -> Result<ReportData, Error> {
        for ProtocolEntry { timestamp, event } in protocol {
            let time = timestamp.to_report_date_time(&timezone);
            self.handle_event(event, time)?;
        }

        self.data
            .finalize(&self.user_names, timezone, report_language)
    }

    fn handle_event(
        &mut self,
        event: VoteEvent,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        match event {
            VoteEvent::Start { issuer, parameters } => self.handle_start(issuer, *parameters),
            VoteEvent::Vote {
                user_info,
                token,
                option,
            } => self.handle_vote(user_info, token, option, time)?,
            VoteEvent::Stop(stop_kind) => self.handle_stop(stop_kind, time)?,
            VoteEvent::FinalResults(final_results) => self.handle_final_results(final_results),
            VoteEvent::Issue { user_info, issue } => self.handle_issue(user_info, issue, time)?,
            VoteEvent::UserLeft { user_info } => self.handle_user_left(user_info, time)?,
            VoteEvent::UserJoined { user_info } => self.handle_user_joined(user_info, time)?,
            VoteEvent::Cancel { issuer, reason } => self.handle_cancel(issuer, reason, time)?,
        }

        Ok(())
    }

    fn handle_start(&mut self, issuer: UserId, parameters: Parameters) {
        self.data.start = Some(Start { issuer, parameters });
    }

    fn handle_vote(
        &mut self,
        user_info: Option<UserInfo>,
        token: Token,
        option: VoteOption,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let name = match user_info {
            Some(info) => Some(
                self.user_names
                    .get(&info.issuer)
                    .ok_or(Error::UserDisplayNameNotFound {
                        user_id: info.issuer,
                    })?
                    .clone(),
            ),
            None => None,
        };

        self.data.votes.push(ResolvedVote {
            name,
            token: token.to_string(),
            option,
            time,
        });

        Ok(())
    }

    fn handle_stop(
        &mut self,
        stop_kind: StopKind,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let stop_kind = match stop_kind {
            StopKind::ByUser(user_id) => StopReason::ByUser {
                user: self.get_user_name(user_id)?,
            },
            StopKind::Auto => StopReason::Auto,
            StopKind::Expired => StopReason::Expired,
        };

        self.data.stop_info = Some(StopInfo {
            time,
            reason: stop_kind,
        });

        Ok(())
    }

    fn handle_final_results(&mut self, final_results: FinalResults) {
        self.data.final_results = Some(final_results);
    }

    fn handle_issue(
        &mut self,
        user_info: Option<UserInfo>,
        issue: Issue,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let name = match user_info {
            Some(info) => Some(self.get_user_name(info.issuer)?),
            None => None,
        };

        self.data.events.push(TimedEvent {
            time,
            event: Event::Issue { name, issue },
        });

        Ok(())
    }

    fn handle_user_left(
        &mut self,
        user_info: Option<UserInfo>,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let name = match user_info {
            Some(info) => Some(self.get_user_name(info.issuer)?),
            None => None,
        };

        self.data.events.push(TimedEvent {
            time,
            event: Event::UserLeft { name },
        });

        Ok(())
    }

    fn handle_user_joined(
        &mut self,
        user_info: Option<UserInfo>,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        let name = match user_info {
            Some(info) => Some(self.get_user_name(info.issuer)?),
            None => None,
        };

        self.data.events.push(TimedEvent {
            time,
            event: Event::UserJoined { name },
        });

        Ok(())
    }

    fn handle_cancel(
        &mut self,
        issuer: UserId,
        reason: CancelReason,
        time: Option<ReportDateTime>,
    ) -> Result<(), Error> {
        self.data.stop_info = Some(StopInfo {
            time,
            reason: StopReason::Canceled(ResolvedCancel {
                user: self.get_user_name(issuer)?,
                reason,
            }),
        });

        Ok(())
    }

    fn get_user_name(&self, user_id: UserId) -> Result<DisplayName, Error> {
        Ok(self
            .user_names
            .get(&user_id)
            .ok_or(Error::UserDisplayNameNotFound { user_id })?
            .clone())
    }
}
