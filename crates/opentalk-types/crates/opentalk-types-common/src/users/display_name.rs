// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use itertools::Itertools as _;
use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

/// The maximum allowed number of characters for a [`DisplayName`]
pub const DISPLAY_NAME_MAX_LENGTH: usize = 255;

/// The display name of a user.
///
/// Can be parsed using [`std::str::FromStr`].
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
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
pub struct DisplayName(String);

impl DisplayName {
    /// Returns `true` if this `DisplayName` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Create a new empty [`DisplayName`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [`DisplayName`] from a `&str`. If the input value is not
    /// suitable, it will be modified to become a valid [`DisplayName`], e.g. by
    /// stripping or replacing characters, or trimming whitespace.
    pub fn from_str_lossy(s: &str) -> Self {
        Self(
            s.split_whitespace()
                .join(" ")
                .chars()
                .take(DISPLAY_NAME_MAX_LENGTH)
                .collect::<String>()
                .trim()
                .to_string(),
        )
    }

    /// Get the `&str` reference to the display name string
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get a [`DisplayName`] containing the String `"Participant"`
    pub fn participant() -> Self {
        Self("Participant".to_string())
    }

    /// Get the length of the username (in characters)
    pub fn len(&self) -> usize {
        self.0.chars().count()
    }
}

#[cfg(feature = "utoipa")]
mod impl_to_schema {
    //! The `#[derive(utoipa::ToSchema)] implementation does not yet properly support
    //! exposing schema information of types wrapped by the NewType pattern, therefore
    //! a manual implementation is required for now.
    //! Issue: <https://github.com/juhaku/utoipa/issues/663>

    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::{DISPLAY_NAME_MAX_LENGTH, DisplayName};
    use crate::utils::ExampleData as _;

    impl PartialSchema for DisplayName {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("The display name of a user or participant"))
                .max_length(Some(DISPLAY_NAME_MAX_LENGTH))
                .examples([json!(DisplayName::example_data())])
                .into()
        }
    }

    impl ToSchema for DisplayName {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for DisplayName {
    fn example_data() -> Self {
        Self("Alice Adams".to_string())
    }
}

/// The error that is returned by [DisplayName::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseDisplayNameError {
    /// The input string was longer than the maximum length [DISPLAY_NAME_MAX_LENGTH].
    #[snafu(display("Display name must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for DisplayName {
    type Err = ParseDisplayNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= DISPLAY_NAME_MAX_LENGTH,
            TooLongSnafu {
                max_length: DISPLAY_NAME_MAX_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{DisplayName, ParseDisplayNameError};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<DisplayName>().unwrap(),
            DisplayName("hello".to_string())
        );
        assert_eq!(
            "".parse::<DisplayName>().unwrap(),
            DisplayName("".to_string())
        );
        assert_eq!(
            "_".parse::<DisplayName>().unwrap(),
            DisplayName("_".to_string())
        );
        assert_eq!(
            "hello world".parse::<DisplayName>().unwrap(),
            DisplayName("hello world".to_string())
        );
        assert_eq!(
            "🚀".parse::<DisplayName>().unwrap(),
            DisplayName("🚀".to_string())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(
            longest.parse::<DisplayName>().unwrap(),
            DisplayName(longest)
        );
    }

    #[test]
    fn parse_invalid() {
        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<DisplayName>(),
            Err(ParseDisplayNameError::TooLong { max_length: 255 })
        ));
    }

    #[test]
    fn from_str_lossy_leading_spaces() {
        assert_eq!(
            DisplayName::from_str_lossy("  First Last"),
            DisplayName("First Last".to_string()),
        );
    }

    #[test]
    fn from_str_lossy_trailing_spaces() {
        assert_eq!(
            DisplayName::from_str_lossy("First Last  "),
            DisplayName("First Last".to_string()),
        );
    }

    #[test]
    fn from_str_lossy_spaces_between() {
        assert_eq!(
            DisplayName::from_str_lossy("First  Last"),
            DisplayName("First Last".to_string()),
        );
    }
}
