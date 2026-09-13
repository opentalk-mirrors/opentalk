// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! The id of a module, and some extra functionality around it.

use std::str::FromStr;

use snafu::{Snafu, ensure};

use crate::Identifier;

/// The minimum allowed length for a valid module id
pub const MODULE_ID_MIN_LENGTH: usize = 1;

/// The maximum allowed length for a valid module id
pub const MODULE_ID_MAX_LENGTH: usize = 255;

/// Regular expression of characters that are allowed inside a module id.
pub const MODULE_ID_SCHEMA_CHARS_REGEX: &str = "[_0-9a-z]";

/// The core module id
pub const CORE_MODULE_ID: ModuleId = ModuleId::__new_borrowed("core");

/// The default module id
pub const DEFAULT_MODULE_ID: ModuleId = CORE_MODULE_ID;

/// The id of a module.
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
#[cfg_attr(feature="diesel", diesel(sql_type = diesel::sql_types::Text))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
#[cfg_attr(
    feature = "typescript",
    derive(ts_rs::TS),
    ts(export_to = "common-identifiers/")
)]
pub struct ModuleId(Identifier);

impl Default for ModuleId {
    fn default() -> Self {
        DEFAULT_MODULE_ID
    }
}

impl ModuleId {
    /// Check whether this module id is the default module id identified by [`DEFAULT_MODULE_ID`].
    pub fn is_default(&self) -> bool {
        self == &DEFAULT_MODULE_ID
    }

    /// Get the `&str` reference to the module id
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    // Only for use in the `module_id` macro
    #[doc(hidden)]
    pub const fn __new_borrowed(value: &'static str) -> Self {
        Self(Identifier::new_borrowed(value))
    }

    /// Get an example instance of the [`ModuleId`].
    pub const fn example_data() -> Self {
        Self::__new_borrowed("mymodule")
    }
}

impl TryFrom<&'static str> for ModuleId {
    type Error = ParseModuleIdError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        ensure_is_valid(value)?;
        Ok(Self::__new_borrowed(value))
    }
}

impl TryFrom<String> for ModuleId {
    type Error = ParseModuleIdError;

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
        openapi::{ObjectBuilder, RefOr, Schema, Type, schema::AnyOf},
    };

    use super::{
        MODULE_ID_MAX_LENGTH, MODULE_ID_MIN_LENGTH, MODULE_ID_SCHEMA_CHARS_REGEX, ModuleId,
    };

    impl PartialSchema for ModuleId {
        fn schema() -> RefOr<Schema> {
            Schema::AnyOf(AnyOf {
                items: vec![
                    ObjectBuilder::new()
                        .schema_type(Type::String)
                        .description(Some("A module identifier"))
                        .min_length(Some(MODULE_ID_MIN_LENGTH))
                        .max_length(Some(MODULE_ID_MAX_LENGTH))
                        .pattern(Some(format!("^{MODULE_ID_SCHEMA_CHARS_REGEX}*$")))
                        .examples([json!(ModuleId::example_data())])
                        .into(),
                ],
                description: None,
                default: Some(json!(ModuleId::default())),
                example: Some(json!(Self::example_data())),
                discriminator: None,
                ..Default::default()
            })
            .into()
        }
    }

    impl ToSchema for ModuleId {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

fn ensure_is_valid(s: &str) -> Result<(), ParseModuleIdError> {
    ensure!(
        s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
        InvalidCharactersSnafu
    );
    ensure!(
        s.len() >= MODULE_ID_MIN_LENGTH,
        TooShortSnafu {
            min_length: MODULE_ID_MIN_LENGTH
        }
    );
    ensure!(
        s.len() <= MODULE_ID_MAX_LENGTH,
        TooLongSnafu {
            max_length: MODULE_ID_MAX_LENGTH
        }
    );
    Ok(())
}

/// The error that is returned by [ModuleId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseModuleIdError {
    /// Invalid characters were found in the input data.
    #[snafu(display("Module id may only contain lowercase letters, digits or \"_\""))]
    InvalidCharacters,

    /// The input string was shorter than the minimum length [MODULE_ID_MIN_LENGTH].
    #[snafu(display("Module id must have at least {min_length} characters"))]
    TooShort {
        /// The minimum allowed length.
        min_length: usize,
    },

    /// The input string was longer than the maximum length [MODULE_ID_MAX_LENGTH].
    #[snafu(display("Module id must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for ModuleId {
    type Err = ParseModuleIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure_is_valid(s)?;
        Ok(Self(Identifier::new_owned(s.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use pretty_assertions::assert_eq;

    use super::{ModuleId, ParseModuleIdError};
    use crate::{identifier::Identifier, module_id::DEFAULT_MODULE_ID};

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<ModuleId>().unwrap(),
            ModuleId("hello".into())
        );
        assert_eq!("_".parse::<ModuleId>().unwrap(), ModuleId("_".into()));
        assert_eq!(
            "hello_world".parse::<ModuleId>().unwrap(),
            ModuleId("hello_world".into())
        );

        assert_eq!("1".parse::<ModuleId>().unwrap(), ModuleId("1".into()));
        assert_eq!(
            "hello1".parse::<ModuleId>().unwrap(),
            ModuleId("hello1".into())
        );

        let longest: String = "x".repeat(255);
        assert_eq!(
            longest.parse::<ModuleId>().unwrap(),
            ModuleId(longest.into())
        );
    }

    #[test]
    fn parse_invalid() {
        assert!(matches!(
            "".parse::<ModuleId>(),
            Err(ParseModuleIdError::TooShort { min_length: 1 })
        ));

        assert!(matches!(
            " ".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "-".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello-world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello+world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello~world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        assert!(matches!(
            "hello::world".parse::<ModuleId>(),
            Err(ParseModuleIdError::InvalidCharacters)
        ));

        let too_long: String = "x".repeat(256);
        assert!(matches!(
            too_long.parse::<ModuleId>(),
            Err(ParseModuleIdError::TooLong { max_length: 255 })
        ));
    }

    #[test]
    fn try_from_static_str() {
        assert!(matches!(
            ModuleId::try_from(""),
            Err(ParseModuleIdError::TooShort { min_length: 1 })
        ));
        assert!(matches!(
            ModuleId::try_from("hello+world"),
            Err(ParseModuleIdError::InvalidCharacters)
        ));
        assert_eq!(
            ModuleId::try_from("hello").expect("value must be parsable as ModuleId"),
            ModuleId::__new_borrowed("hello")
        );
    }

    #[test]
    fn try_from_string() {
        assert!(matches!(
            ModuleId::try_from("".to_string()),
            Err(ParseModuleIdError::TooShort { min_length: 1 })
        ));
        assert!(matches!(
            ModuleId::try_from("hello+world".to_string()),
            Err(ParseModuleIdError::InvalidCharacters)
        ));
        assert_eq!(
            ModuleId::try_from("hello".to_string()).expect("value must be parsable as ModuleId"),
            ModuleId::__new_borrowed("hello")
        );
    }

    #[test]
    fn partial_eq() {
        assert!(
            ModuleId::try_from("a").expect("value must be parsable as ModuleId")
                < ModuleId("z".to_string().into())
        );
        assert!(
            ModuleId::try_from("z").expect("value must be parsable as ModuleId")
                > ModuleId("a".to_string().into())
        );
    }

    #[test]
    fn hash_by_hash_set() {
        // we test availability of hashing indirectly usage of a HashSet.
        let expected: HashSet<ModuleId> = HashSet::from_iter(
            ["a", "b", "c"]
                .into_iter()
                .map(|s| ModuleId(Identifier::from(s))),
        );

        let b = HashSet::from_iter([
            ModuleId("a".into()),
            ModuleId("c".into()),
            ModuleId("a".to_string().into()),
            "b".parse().expect("value must be parsable as ModuleId"),
        ]);

        assert_eq!(b, expected);
    }

    #[test]
    fn ord_by_btree_set() {
        // we test availability PartialOrd indirectly usage of a BTreeSet.
        let expected = ["a", "b", "c"]
            .into_iter()
            .map(|s| ModuleId(Identifier::from(s)))
            .collect();

        let b = BTreeSet::from_iter([
            ModuleId("a".into()),
            ModuleId("c".into()),
            ModuleId("a".to_string().into()),
            "b".parse().expect("value must be parsable as ModuleId"),
        ]);

        assert_eq!(b, expected);
    }

    #[test]
    fn display() {
        assert_eq!("hello", ModuleId("hello".into()).to_string());

        let a = ModuleId("hello".into());
        let b = ModuleId("world".into());
        assert_eq!(format!("{a}, {b}"), "hello, world");
    }

    #[test]
    fn is_default() {
        assert!(DEFAULT_MODULE_ID.is_default());
        assert!(ModuleId::default().is_default());
        assert!(ModuleId("core".into()).is_default());
        assert!(!ModuleId("eroc".into()).is_default());
    }
}
