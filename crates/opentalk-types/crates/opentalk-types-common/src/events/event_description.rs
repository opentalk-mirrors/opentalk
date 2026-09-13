// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

pub const EVENT_DESCRIPTION_MAX_LENGTH: usize = 4096;

/// The description of an event.
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
pub struct EventDescription(String);

impl EventDescription {
    /// Create a new [`EventDescription`] from a `&str`. If the input value is not
    /// suitable, it will be modified to become a valid [`EventDescription`], e.g.
    /// by stripping characters.
    pub fn from_str_lossy(s: &str) -> Self {
        Self(s.chars().take(EVENT_DESCRIPTION_MAX_LENGTH).collect())
    }

    /// Returns `true` if this `EventDescription` has a length of zero, and `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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

    use super::{EVENT_DESCRIPTION_MAX_LENGTH, EventDescription};
    use crate::utils::ExampleData as _;

    impl PartialSchema for EventDescription {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("The description of an event"))
                .max_length(Some(EVENT_DESCRIPTION_MAX_LENGTH))
                .examples([json!(EventDescription::example_data())])
                .into()
        }
    }

    impl ToSchema for EventDescription {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl ExampleData for EventDescription {
    fn example_data() -> Self {
        Self("The Weekly Team Event".to_string())
    }
}

/// The error that is returned by [ModuleId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseEventDescriptionError {
    /// The input string was longer than the maximum length [EVENT_DESCRIPTION_MAX_LENGTH].
    #[snafu(display("Event description must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },
}

impl FromStr for EventDescription {
    type Err = ParseEventDescriptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() <= EVENT_DESCRIPTION_MAX_LENGTH,
            TooLongSnafu {
                max_length: EVENT_DESCRIPTION_MAX_LENGTH
            }
        );
        Ok(Self(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::{EventDescription, ParseEventDescriptionError};
    use crate::events::event_description::EVENT_DESCRIPTION_MAX_LENGTH;

    #[test]
    fn parse() {
        assert_eq!(
            "hello".parse::<EventDescription>().unwrap(),
            EventDescription("hello".to_string())
        );
        assert_eq!(
            "".parse::<EventDescription>().unwrap(),
            EventDescription("".to_string())
        );
        assert_eq!(
            "_".parse::<EventDescription>().unwrap(),
            EventDescription("_".to_string())
        );
        assert_eq!(
            "hello world".parse::<EventDescription>().unwrap(),
            EventDescription("hello world".to_string())
        );
        assert_eq!(
            "🚀".parse::<EventDescription>().unwrap(),
            EventDescription("🚀".to_string())
        );

        let longest: String = "x".repeat(4096);
        assert_eq!(
            longest.parse::<EventDescription>().unwrap(),
            EventDescription(longest)
        );
    }

    #[test]
    fn parse_invalid() {
        let too_long: String = "x".repeat(4097);
        assert!(matches!(
            too_long.parse::<EventDescription>(),
            Err(ParseEventDescriptionError::TooLong { max_length: 4096 })
        ));
    }

    #[test]
    fn from_str_lossy() {
        assert_eq!(
            EventDescription::from_str_lossy("hello"),
            EventDescription("hello".to_string())
        );

        let too_long: String = "x".repeat(EVENT_DESCRIPTION_MAX_LENGTH + 1);
        let title = EventDescription::from_str_lossy(&too_long);
        assert_eq!(title.0.len(), EVENT_DESCRIPTION_MAX_LENGTH);
    }
}
