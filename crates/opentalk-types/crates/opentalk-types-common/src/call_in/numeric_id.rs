// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use derive_more::{AsRef, Display, Into};
use snafu::{Snafu, ensure};

use crate::{call_in::DIAL_IN_NUMERIC_ID_LENGTH, utils::ExampleData};

/// Base type for numeric dial-in identifieirs
///
/// For now, this type checks via its `FromStr` implementation that the value is valid,
/// e.g. when deserializing through [`serde`](https://docs.rs/serde).
///
/// When loading from the database through [`diesel`](https://docs.rs/diesel), no validation
/// is performed in the current implementation.
#[derive(AsRef, Display, Into, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "diesel",
    derive(
        opentalk_diesel_newtype::DieselNewtype,
        diesel::expression::AsExpression,
        diesel::deserialize::FromSqlRow
    )
)]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Text))]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(fmt), from_redis_value(FromStr))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export_to = "common/"))]
pub struct NumericId(String);

impl NumericId {
    /// Generate a new random `NumericId`
    #[cfg(feature = "rand")]
    pub fn generate() -> Self {
        use rand::{RngExt as _, distr::slice::Choose, rng};

        /// The set of numbers used to generate [`SipId`] & [`SipPassword`]
        const NUMERIC: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let numeric_dist = Choose::new(&NUMERIC).unwrap();

        Self(
            rng()
                .sample_iter(numeric_dist)
                .take(DIAL_IN_NUMERIC_ID_LENGTH)
                .collect(),
        )
    }
}

#[derive(Debug, Snafu)]
pub enum ParseNumericIdError {
    #[snafu(display("Invalid numeric id length. Found: {found}, required: exactly {required}"))]
    InvalidLength { found: usize, required: usize },

    #[snafu(display("Invalid character found. Only numeric values are allowed"))]
    InvalidCharacter,
}

impl FromStr for NumericId {
    type Err = ParseNumericIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure!(
            s.len() == DIAL_IN_NUMERIC_ID_LENGTH,
            InvalidLengthSnafu {
                found: s.len(),
                required: DIAL_IN_NUMERIC_ID_LENGTH
            }
        );

        ensure!(s.chars().all(|c| c.is_ascii_digit()), InvalidCharacterSnafu);

        Ok(Self(s.to_string()))
    }
}

impl ExampleData for NumericId {
    fn example_data() -> Self {
        Self("0000000000".to_string())
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

    use super::{DIAL_IN_NUMERIC_ID_LENGTH, NumericId};
    use crate::utils::ExampleData as _;

    impl PartialSchema for NumericId {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .description(Some("A string containing number characters"))
                .min_length(Some(DIAL_IN_NUMERIC_ID_LENGTH))
                .max_length(Some(DIAL_IN_NUMERIC_ID_LENGTH))
                .pattern(Some("[0-9]+"))
                .examples([json!(NumericId::example_data())])
                .into()
        }
    }

    impl ToSchema for NumericId {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DIAL_IN_NUMERIC_ID_LENGTH, NumericId, ParseNumericIdError};

    #[test]
    fn from_str() {
        assert!(matches!(
            "".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidLength {
                found: 0,
                required: DIAL_IN_NUMERIC_ID_LENGTH
            })
        ));

        assert!(matches!(
            "a".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidLength {
                found: 1,
                required: DIAL_IN_NUMERIC_ID_LENGTH
            })
        ));

        assert!(matches!(
            "123456789".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidLength {
                found: 9,
                required: DIAL_IN_NUMERIC_ID_LENGTH
            })
        ));

        assert!(matches!(
            "12345678900".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidLength {
                found: 11,
                required: DIAL_IN_NUMERIC_ID_LENGTH
            })
        ));

        assert!(matches!(
            "123456789a".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidCharacter)
        ));

        assert!(matches!(
            "abcdefghij".parse::<NumericId>(),
            Err(ParseNumericIdError::InvalidCharacter)
        ));

        assert!(matches!(
            "1234567890".parse::<NumericId>(),
            Ok(NumericId(s)) if s == "1234567890"
        ));

        assert!(matches!(
            "0000000000".parse::<NumericId>(),
            Ok(NumericId(s)) if s == "0000000000"
        ));

        assert!(matches!(
            "9999999999".parse::<NumericId>(),
            Ok(NumericId(s)) if s == "9999999999"
        ));
    }
}
