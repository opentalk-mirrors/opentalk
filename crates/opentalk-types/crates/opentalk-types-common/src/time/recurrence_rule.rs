// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

/// The maximum allowed number of characters for a recurrence rule
pub const RECURRENCE_RULE_MAX_LEN: usize = 1024;

/// A recurrence rule according to the
/// [`RFC5545`](https://www.rfc-editor.org/rfc/rfc5545) specification.
///
/// Note: currently the rrule patterns are not enforced, the only enforced
/// requirement is a maximum length of [`RECURRENCE_RULE_MAX_LEN`] characters.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
pub struct RecurrenceRule(String);

/// An error which can be returned when parsing an a recurrence rule
#[derive(Debug, Snafu)]
pub enum ParseRecurrenceRuleError {
    /// The recurrence rule string is too long
    #[snafu(display(
        "Recurrence rule string is too long. Max length: {max_len}, found length: {found_len}"
    ))]
    RecurrenceRuleTooLong {
        /// The length of the string that was found
        found_len: usize,

        /// The maximum allowed length of the string
        max_len: usize,
    },
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::{RECURRENCE_RULE_MAX_LEN, RecurrenceRule};
    use crate::utils::ExampleData as _;

    impl PartialSchema for RecurrenceRule {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .max_length(Some(RECURRENCE_RULE_MAX_LEN))
                .description(Some("A recurrence rule according to RFC5545"))
                .examples([json!(RecurrenceRule::example_data())])
                .into()
        }
    }

    impl ToSchema for RecurrenceRule {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl FromStr for RecurrenceRule {
    type Err = ParseRecurrenceRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= 1024,
            RecurrenceRuleTooLongSnafu {
                found_len: s.len(),
                max_len: RECURRENCE_RULE_MAX_LEN
            }
        );
        Ok(Self(s.to_string()))
    }
}

impl ExampleData for RecurrenceRule {
    fn example_data() -> Self {
        Self("FREQ=WEEKLY;INTERVAL=1;BYDAY=MO".to_string())
    }
}
