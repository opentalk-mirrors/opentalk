// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `meeting_report` namespace

mod error;
mod meeting_report_event;

pub use error::MeetingReportError;
pub use meeting_report_event::MeetingReportEvent;
