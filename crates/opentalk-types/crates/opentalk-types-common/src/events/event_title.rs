// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

pub const EVENT_TITLE_MAX_LENGTH: usize = 255;

/// The title of an event.
///
/// Can be parsed using [`std::str::FromStr`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
#[cfg_attr(
    feature = "diesel",
    derive(
        opentalk_diesel_newtype::DieselNewtype,
        diesel::expression::AsExpression,
        diesel::deserialize::FromSqlRow
    )
)]
#[cfg_attr(
    feature = "diesel",
    diesel(sql_type = diesel::sql_types::Text)
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export_to = "common/"))]
pub struct EventTitle(String);

impl EventTitle {
    /// Create a new [`EventTitle`] from a `&str`. If the input value is not
    /// suitable, it will be modified to become a valid [`EventTitle`], e.g. by
    /// stripping characters.
    pub fn from_str_lossy(s: &str) -> Self {
        Self(s.chars().take(EVENT_TITLE_MAX_LENGTH).collect())
    }

    /// Returns `true` if this `EventTitle` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Sanitize the `EventTitle` so that it can be used in a filename, replacing disallowed
    /// characters by `_`, and trimming to `max_length` (in character count, not bytes).
    pub fn sanitized_for_filename(&self, max_length: usize) -> String {
        fn is_allowed_char(c: char) -> bool {
            c.is_alphanumeric() || ['.', '_', '-', ' '].contains(&c)
        }

        fn to_valid_char(c: char) -> char {
            if is_allowed_char(c) { c } else { '_' }
        }

        self.0.chars().take(max_length).map(to_valid_char).collect()
    }

    /// Extracts a string slice containing the entire EventTitle.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    //! The `#[derive(utoipa::ToSchema)] implementation does not yet properly support
    //! exposing schema information of types wrapped by the NewType pattern, therefore
    //! a manual implementation is required for now.
    //! Issue: <https://github.com/juhaku/utoipa/issues/663>

    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::{EVENT_TITLE_MAX_LENGTH, EventTitle};
    use crate::utils::ExampleData as _;

    impl PartialSchema for EventTitle {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("The title of an event"))
                .max_length(Some(EVENT_TITLE_MAX_LENGTH))
                .examples([json!(EventTitle::example_data())])
                .into()
        }
    }

    impl ToSchema for EventTitle {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for EventTitle {
    fn example_data() -> Self {
        Self("Team Event".to_string())
    }
}

/// The error that is returned by [ModuleId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseEventTitleError {
    /// The input string was longer than the maximum length [EVENT_TITLE_MAX_LENGTH].
    #[snafu(display("Event title must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for EventTitle {
    type Err = ParseEventTitleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= EVENT_TITLE_MAX_LENGTH,
            TooLongSnafu {
                max_length: EVENT_TITLE_MAX_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{EventTitle, ParseEventTitleError};
    use crate::events::event_title::EVENT_TITLE_MAX_LENGTH;

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<EventTitle>().unwrap(),
            EventTitle("hello".to_string())
        );
        assert_eq!(
            "".parse::<EventTitle>().unwrap(),
            EventTitle("".to_string())
        );
        assert_eq!(
            "_".parse::<EventTitle>().unwrap(),
            EventTitle("_".to_string())
        );
        assert_eq!(
            "hello world".parse::<EventTitle>().unwrap(),
            EventTitle("hello world".to_string())
        );
        assert_eq!(
            "🚀".parse::<EventTitle>().unwrap(),
            EventTitle("🚀".to_string())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(longest.parse::<EventTitle>().unwrap(), EventTitle(longest));
    }

    #[test]
    fn parse_invalid() {
        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<EventTitle>(),
            Err(ParseEventTitleError::TooLong { max_length: 255 })
        ));
    }

    #[test]
    fn from_str_lossy() {
        assert_eq!(
            EventTitle::from_str_lossy("hello"),
            EventTitle("hello".to_string())
        );

        let too_long: String = "x".repeat(EVENT_TITLE_MAX_LENGTH + 1);
        let title = EventTitle::from_str_lossy(&too_long);
        assert_eq!(title.0.len(), EVENT_TITLE_MAX_LENGTH);
    }
}
