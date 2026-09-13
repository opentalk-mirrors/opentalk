// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

/// The maximum allowed number of characters for a [`UserTitle`]
pub const USER_TITLE_MAX_LENGTH: usize = 255;

/// The title of a user.
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
pub struct UserTitle(String);

impl UserTitle {
    /// Returns `true` if this `UserTitle` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Create a new empty [`UserTitle`]
    pub fn new() -> Self {
        Self::default()
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

    use super::{USER_TITLE_MAX_LENGTH, UserTitle};
    use crate::utils::ExampleData as _;

    impl PartialSchema for UserTitle {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("The title of a user"))
                .max_length(Some(USER_TITLE_MAX_LENGTH))
                .examples([json!(UserTitle::example_data())])
                .into()
        }
    }

    impl ToSchema for UserTitle {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for UserTitle {
    fn example_data() -> Self {
        Self("M.Sc.".to_string())
    }
}

/// The error that is returned by [UserTitle::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseUserTitleError {
    /// The input string was longer than the maximum length [USER_TITLE_MAX_LENGTH].
    #[snafu(display("User title must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for UserTitle {
    type Err = ParseUserTitleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= USER_TITLE_MAX_LENGTH,
            TooLongSnafu {
                max_length: USER_TITLE_MAX_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{ParseUserTitleError, UserTitle};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<UserTitle>().unwrap(),
            UserTitle("hello".to_string())
        );
        assert_eq!("".parse::<UserTitle>().unwrap(), UserTitle("".to_string()));
        assert_eq!(
            "_".parse::<UserTitle>().unwrap(),
            UserTitle("_".to_string())
        );
        assert_eq!(
            "hello world".parse::<UserTitle>().unwrap(),
            UserTitle("hello world".to_string())
        );
        assert_eq!(
            "🚀".parse::<UserTitle>().unwrap(),
            UserTitle("🚀".to_string())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(longest.parse::<UserTitle>().unwrap(), UserTitle(longest));
    }

    #[test]
    fn parse_invalid() {
        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<UserTitle>(),
            Err(ParseUserTitleError::TooLong { max_length: 255 })
        ));
    }
}
