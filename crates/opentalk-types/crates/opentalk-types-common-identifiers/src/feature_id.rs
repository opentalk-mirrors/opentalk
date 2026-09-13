// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! The id of a feature, and some extra functionality around it.

use std::str::FromStr;

use snafu::{Snafu, ensure};

use crate::Identifier;

/// The minimum allowed length for a valid feature id
pub const FEATURE_ID_MIN_LENGTH: usize = 1;

/// The maximum allowed length for a valid feature id
pub const FEATURE_ID_MAX_LENGTH: usize = 255;

/// Regular expression of characters that are allowed inside a feature id.
pub const FEATURE_ID_SCHEMA_CHARS_REGEX: &str = "[_0-9a-z]";

/// The id of a feature.
///
/// Can be parsed using [`std::str::FromStr`].
/// May only contain alphanumeric ascii characters.
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
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export_to = "common-identifiers/")
)]
pub struct FeatureId(Identifier);

impl FeatureId {
    /// Get the `&str` reference to the module id
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    // Only for use in the `module_id` macro
    #[doc(hidden)]
    pub const fn __new_borrowed(value: &'static str) -> Self {
        Self(Identifier::new_borrowed(value))
    }

    /// Get an example instance of the [`FeatureId`].
    pub const fn example_data() -> Self {
        Self::__new_borrowed("myfeature")
    }
}

impl TryFrom<&'static str> for FeatureId {
    type Error = ParseFeatureIdError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        ensure_is_valid(value)?;
        Ok(Self::__new_borrowed(value))
    }
}

impl TryFrom<String> for FeatureId {
    type Error = ParseFeatureIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
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

    use super::{
        FEATURE_ID_MAX_LENGTH, FEATURE_ID_MIN_LENGTH, FEATURE_ID_SCHEMA_CHARS_REGEX, FeatureId,
    };

    impl PartialSchema for FeatureId {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("A feature identifier"))
                .min_length(Some(FEATURE_ID_MIN_LENGTH))
                .max_length(Some(FEATURE_ID_MAX_LENGTH))
                .pattern(Some(format!("^{FEATURE_ID_SCHEMA_CHARS_REGEX}*$")))
                .examples([json!(FeatureId::example_data())])
                .into()
        }
    }

    impl ToSchema for FeatureId {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

fn ensure_is_valid(s: &str) -> Result<(), ParseFeatureIdError> {
    ensure!(
        s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
        InvalidCharactersSnafu
    );
    ensure!(
        s.len() >= FEATURE_ID_MIN_LENGTH,
        TooShortSnafu {
            min_length: FEATURE_ID_MIN_LENGTH
        }
    );
    ensure!(
        s.len() <= FEATURE_ID_MAX_LENGTH,
        TooLongSnafu {
            max_length: FEATURE_ID_MAX_LENGTH
        }
    );
    Ok(())
}

/// The error that is returned by [FeatureId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseFeatureIdError {
    /// Invalid characters were found in the input data.
    #[snafu(display("Feature id may only contain lowercase letters, digits or \"_\""))]
    InvalidCharacters,

    /// The input string was shorter than the minimum length [FEATURE_ID_MIN_LENGTH].
    #[snafu(display("Feature id must have at least {min_length} characters"))]
    TooShort {
        /// The minimum allowed length.
        min_length: usize,
    },

    /// The input string was longer than the maximum length [FEATURE_ID_MAX_LENGTH].
    #[snafu(display("Feature id must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for FeatureId {
    type Err = ParseFeatureIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure_is_valid(s)?;
        Ok(Self(Identifier::new_owned(s.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use pretty_assertions::assert_eq;

    use super::{FeatureId, ParseFeatureIdError};
    use crate::identifier::Identifier;

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<FeatureId>().unwrap(),
            FeatureId("hello".into())
        );

        assert_eq!("_".parse::<FeatureId>().unwrap(), FeatureId("_".into()));
        assert_eq!(
            "hello_world".parse::<FeatureId>().unwrap(),
            FeatureId("hello_world".into())
        );

        assert_eq!("1".parse::<FeatureId>().unwrap(), FeatureId("1".into()));
        assert_eq!(
            "hello1".parse::<FeatureId>().unwrap(),
            FeatureId("hello1".into())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(
            longest.parse::<FeatureId>().unwrap(),
            FeatureId(longest.into())
        );
    }

    #[test]
    fn parse_invalid() {
        assert!(matches!(
            "".parse::<FeatureId>(),
            Err(ParseFeatureIdError::TooShort { min_length: 1 })
        ));

        assert!(matches!(
            " ".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "-".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello-world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello+world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello~world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello::world".parse::<FeatureId>(),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));

        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<FeatureId>(),
            Err(ParseFeatureIdError::TooLong { max_length: 255 })
        ));
    }

    #[test]
    fn try_from_static_str() {
        assert!(matches!(
            FeatureId::try_from(""),
            Err(ParseFeatureIdError::TooShort { min_length: 1 })
        ));
        assert!(matches!(
            FeatureId::try_from("hello+world"),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));
        assert_eq!(
            FeatureId::try_from("hello").expect("value must be parsable as FeatureId"),
            FeatureId::__new_borrowed("hello")
        );
    }

    #[test]
    fn try_from_string() {
        assert!(matches!(
            FeatureId::try_from("".to_string()),
            Err(ParseFeatureIdError::TooShort { min_length: 1 })
        ));
        assert!(matches!(
            FeatureId::try_from("hello+world".to_string()),
            Err(ParseFeatureIdError::InvalidCharacters)
        ));
        assert_eq!(
            FeatureId::try_from("hello".to_string()).expect("value must be parsable as FeatureId"),
            FeatureId::__new_borrowed("hello")
        );
    }

    #[test]
    fn partial_eq() {
        assert!(
            FeatureId::try_from("a").expect("value must be parsable as FeatureId")
                < FeatureId("z".to_string().into())
        );
        assert!(
            FeatureId::try_from("z").expect("value must be parsable as FeatureId")
                > FeatureId("a".to_string().into())
        );
    }

    #[test]
    fn hash_by_hash_set() {
        // we test availability of hashing indirectly usage of a HashSet.
        let expected: HashSet<FeatureId> = HashSet::from_iter(
            ["a", "b", "c"]
                .into_iter()
                .map(|s| FeatureId(Identifier::from(s))),
        );

        let b = HashSet::from_iter([
            FeatureId("a".into()),
            FeatureId("c".into()),
            FeatureId("a".to_string().into()),
            "b".parse().expect("value must be parsable as FeatureId"),
        ]);

        assert_eq!(b, expected);
    }

    #[test]
    fn ord_by_btree_set() {
        // we test availability PartialOrd indirectly usage of a BTreeSet.
        let expected = ["a", "b", "c"]
            .into_iter()
            .map(|s| FeatureId(Identifier::from(s)))
            .collect();

        let b = BTreeSet::from_iter([
            FeatureId("a".into()),
            FeatureId("c".into()),
            FeatureId("a".to_string().into()),
            "b".parse().expect("value must be parsable as FeatureId"),
        ]);

        assert_eq!(b, expected);
    }

    #[test]
    fn display() {
        assert_eq!("hello", FeatureId("hello".into()).to_string());

        let a = FeatureId("hello".into());
        let b = FeatureId("world".into());
        assert_eq!(format!("{a}, {b}"), "hello, world");
    }
}
