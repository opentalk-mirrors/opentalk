// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Utils for [`Event`]
use chrono::{DateTime, TimeZone};
use opentalk_inventory::{Event, EventDate};
use opentalk_types_api_common::error::ApiError;
use rrule::{RRule, RRuleSet, Unvalidated};
use snafu::{Report, ResultExt, Snafu};

/// Error that can be returned from rrule related functions
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
#[non_exhaustive]
pub enum EventRRuleSetError {
    /// The data representation of the event is inconsistent
    #[snafu(display("Inconsistent event data found: {message}"))]
    InconsistentEventData {
        /// Message describing the found inconsistency
        message: String,
    },

    /// An error occurred while working with RRules
    #[snafu(display("Error in RRule found: {source}"))]
    RRuleError {
        /// The source of the error
        source: rrule::RRuleError,
    },
}

impl From<EventRRuleSetError> for ApiError {
    fn from(value: EventRRuleSetError) -> Self {
        match value {
            EventRRuleSetError::InconsistentEventData { message: _ } => ApiError::internal(),
            EventRRuleSetError::RRuleError { source } => {
                log::error!("failed to parse rrule {}", Report::from_error(source));
                ApiError::internal()
            }
        }
    }
}

/// An extension trait for [`Event`].
pub trait EventExt {
    /// Get the [`RRuleSet`] for the [`Event`], will return `Ok(None)` for non
    /// recurring events.
    fn to_rruleset(&self) -> Result<Option<RRuleSet>, EventRRuleSetError>;

    /// Check if last occurence starts before a specific date.
    fn has_last_occurrence_before<T: TimeZone>(
        &self,
        date: DateTime<T>,
    ) -> Result<bool, EventRRuleSetError>;
}

impl EventExt for Event {
    fn to_rruleset(&self) -> Result<Option<RRuleSet>, EventRRuleSetError> {
        let Some(date) = self.date() else {
            return Ok(None);
        };

        date.to_rruleset()
    }

    fn has_last_occurrence_before<T: TimeZone>(
        &self,
        date: DateTime<T>,
    ) -> Result<bool, EventRRuleSetError> {
        let Some(rruleset) = self.to_rruleset()? else {
            return Ok(false);
        };

        if rruleset
            .get_rrule()
            .iter()
            .any(|rrule| rrule.get_count().is_some() || rrule.get_until().is_some())
        {
            match rruleset.into_iter().last() {
                Some(dt) if dt < date => return Ok(true),
                Some(_) | None => return Ok(false),
            }
        }

        Ok(false)
    }
}

/// An extension trait for [`EventDate`].
pub trait EventDateExt {
    /// Get the [`RRuleSet`] for the [`Event`], will return `Ok(None)` for non
    /// recurring events.
    fn to_rruleset(&self) -> Result<Option<RRuleSet>, EventRRuleSetError>;
}

impl EventDateExt for EventDate {
    fn to_rruleset(&self) -> Result<Option<RRuleSet>, EventRRuleSetError> {
        let Some(recurrence) = self.recurrence.as_ref() else {
            return Ok(None);
        };

        let rrule: RRule<Unvalidated> =
            recurrence.recurrence_pattern.parse().context(RRuleSnafu)?;

        // rrule uses chrono-tz 0.9 while we have 0.10 already.
        // as a workaround we convert through a string that we parse.
        // good enough for this use case, can be romved when chrono-tz
        // is updated in rrule.
        let starts_at_tz = self
            .starts_at_tz
            .to_string()
            .parse()
            .expect("timezone should be parseable");

        let starts_at_with_tz = self.starts_at.with_timezone(&rrule::Tz::Tz(starts_at_tz));

        let rruleset = rrule.build(starts_at_with_tz).context(RRuleSnafu)?;

        Ok(Some(rruleset))
    }
}
