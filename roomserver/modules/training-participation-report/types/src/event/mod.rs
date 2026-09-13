// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `training_participation_report` namespace

mod error;
mod presence_logging_ended_reason;
mod presence_logging_started_reason;
mod training_participation_report_event;

pub use error::TrainingParticipationReportError;
pub use presence_logging_ended_reason::PresenceLoggingEndedReason;
pub use presence_logging_started_reason::PresenceLoggingStartedReason;
pub use training_participation_report_event::TrainingParticipationReportEvent;
