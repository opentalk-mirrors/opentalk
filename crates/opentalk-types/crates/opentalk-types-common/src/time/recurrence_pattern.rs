// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use itertools::Itertools as _;
use snafu::{ResultExt as _, Snafu, ensure};

use crate::{
    time::{ParseRecurrenceRuleError, RecurrenceRule},
    utils::ExampleData,
};

/// The maximum number of recurrence rules inside a recurrence pattern
pub const RECURRENCE_PATTERN_MAX_LEN: usize = 4;

/// A recurrence pattern containing zero or more recurrence rules
#[derive(Default, Debug, Clone, PartialEq, Eq, derive_more::AsRef, derive_more::Into)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "Vec<RecurrenceRule>")
)]
pub struct RecurrencePattern(Vec<RecurrenceRule>);

impl RecurrencePattern {
    /// Returns `true` if the recurrence pattern contains no recurrence rules
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert to a multiline string
    ///
    /// If [`RecurrencePattern::is_empty`] is `true`, `None` is returned.
    /// If non-empty, one line per pattern is used.
    pub fn to_multiline_string(&self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self.0.iter().map(|p| p.to_string()).join("\n"))
        }
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{Ref, RefOr, Schema},
    };

    use super::{RECURRENCE_PATTERN_MAX_LEN, RecurrencePattern};
    use crate::{time::RecurrenceRule, utils::ExampleData as _};

    impl PartialSchema for RecurrencePattern {
        fn schema() -> RefOr<Schema> {
            Ref::from_schema_name(RecurrenceRule::name())
                .to_array_builder()
                .min_items(Some(1))
                .max_items(Some(RECURRENCE_PATTERN_MAX_LEN))
                .description(Some("A recurrence pattern containing recurrence rules"))
                .examples([json!(RecurrencePattern::example_data())])
                .into()
        }
    }

    impl ToSchema for RecurrencePattern {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

/// The error type returned when a conversion from a [`Vec<RecurrenceRule>`] to
/// a [`RecurrencePattern`] fails.
#[derive(Debug, Snafu)]
pub enum TryFromRecurrenceRulesError {
    /// Too many recurrence rules found
    #[snafu(display("Too many recurrence rules found. Max: {max}, found: {found}"))]
    TooManyRecurrenceRules {
        /// The number of found rules
        found: usize,

        /// The number of maximum allowed rules
        max: usize,
    },
}

impl TryFrom<Vec<RecurrenceRule>> for RecurrencePattern {
    type Error = TryFromRecurrenceRulesError;

    fn try_from(value: Vec<RecurrenceRule>) -> Result<Self, Self::Error> {
        ensure!(
            value.len() <= RECURRENCE_PATTERN_MAX_LEN,
            TooManyRecurrenceRulesSnafu {
                found: value.len(),
                max: RECURRENCE_PATTERN_MAX_LEN
            }
        );

        Ok(Self(value))
    }
}

/// The error type returned when parsing a [`RecurrencePattern`].
#[derive(Debug, Snafu)]
pub enum ParseRecurrencePatternError {
    /// Too many recurrence rules found
    #[snafu(display("Too many recurrence rules found. Max: {max}, found: {found}"))]
    TooManyRecurrenceRulesParsed {
        /// The number of found rules
        found: usize,

        /// The number of maximum allowed rules
        max: usize,
    },

    /// Recurrence rule parsing failed
    #[snafu(display("Recurring rule parsing failed at index {index}: {source}"))]
    RecurringRuleParsingFailed {
        /// Zero-based index of the line that failed to parse
        index: usize,

        /// Parsing error
        source: ParseRecurrenceRuleError,
    },
}

impl FromStr for RecurrencePattern {
    type Err = ParseRecurrencePatternError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<&str>>();

        ensure!(
            lines.len() <= RECURRENCE_PATTERN_MAX_LEN,
            TooManyRecurrenceRulesParsedSnafu {
                found: lines.len(),

                max: RECURRENCE_PATTERN_MAX_LEN
            }
        );

        let rules = lines
            .into_iter()
            .enumerate()
            .map(|(index, v)| {
                v.parse::<RecurrenceRule>()
                    .context(RecurringRuleParsingFailedSnafu { index })
            })
            .collect::<Result<Vec<RecurrenceRule>, ParseRecurrencePatternError>>()?;
        Ok(Self(rules))
    }
}

impl ExampleData for RecurrencePattern {
    fn example_data() -> Self {
        Self(vec![RecurrenceRule::example_data()])
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use serde_json::json;

    use super::RecurrencePattern;

    #[test]
    fn deserialize() {
        use crate::time::RecurrenceRule;

        let expected_pattern: RecurrenceRule = "FREQ=WEEKLY;INTERVAL=1;BYDAY=MO".parse().unwrap();

        let json = json!(["FREQ=WEEKLY;INTERVAL=1;BYDAY=MO"]);

        let deserialized: RecurrencePattern = serde_json::from_value(json).unwrap();

        assert_eq!(RecurrencePattern(vec![expected_pattern]), deserialized);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_with_invalid_recurrence_rule() {
        let invalid_recurrence_rule_string = "x".repeat(2000);

        let json = json!([invalid_recurrence_rule_string]);

        assert!(serde_json::from_value::<RecurrencePattern>(json).is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_with_invalid_number_of_rules() {
        use crate::{time::RecurrenceRule, utils::ExampleData as _};

        let valid_recurrence_rule = RecurrenceRule::example_data();

        let json = json!([
            valid_recurrence_rule.clone(),
            valid_recurrence_rule.clone(),
            valid_recurrence_rule.clone(),
            valid_recurrence_rule.clone(),
            valid_recurrence_rule.clone(),
        ]);

        assert!(serde_json::from_value::<RecurrencePattern>(json).is_err());
    }
}
