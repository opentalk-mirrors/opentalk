// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{fmt::Display, str::FromStr};

use snafu::{ResultExt, Snafu, ensure};

use crate::{
    features::{FeatureId, NAMESPACE_SEPARATOR, ParseFeatureIdError},
    modules::{ModuleId, ParseModuleIdError},
    utils::ExampleData,
};

/// Identifier of a feature inside a module
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "diesel",
    derive(diesel::expression::AsExpression, diesel::deserialize::FromSqlRow)
)]
#[cfg_attr(
    feature = "diesel",
    diesel(sql_type = diesel::sql_types::Text),
)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export_to = "common/"))]
pub struct ModuleFeatureId {
    /// The id of the module
    pub module: ModuleId,

    /// The id of the feature
    pub feature: FeatureId,
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type, schema::AnyOf},
    };

    use super::ModuleFeatureId;
    use crate::{
        features::{
            FEATURE_ID_MAX_LENGTH, FEATURE_ID_MIN_LENGTH, FEATURE_ID_SCHEMA_CHARS_REGEX, FeatureId,
        },
        modules::{MODULE_ID_MAX_LENGTH, MODULE_ID_MIN_LENGTH, MODULE_ID_SCHEMA_CHARS_REGEX},
        utils::ExampleData,
    };

    impl PartialSchema for ModuleFeatureId {
        fn schema() -> RefOr<Schema> {
            use serde_json::json;

            let module_id_regex_snippet = format!(
                "{MODULE_ID_SCHEMA_CHARS_REGEX}{{{MODULE_ID_MIN_LENGTH},{MODULE_ID_MAX_LENGTH}}}"
            );
            let feature_id_regex_snippet = format!(
                "{FEATURE_ID_SCHEMA_CHARS_REGEX}{{{FEATURE_ID_MIN_LENGTH},{FEATURE_ID_MAX_LENGTH}}}"
            );

            Schema::AnyOf(AnyOf {
                items: vec![
                    FeatureId::schema(),
                    ObjectBuilder::new()
                        .schema_type(Type::String)
                        .description(Some("A module feature identifier"))
                        .pattern(Some(format!(
                            "^{module_id_regex_snippet}::{feature_id_regex_snippet}$",
                        )))
                        .examples([json!(ModuleFeatureId::example_data())])
                        .into(),
                ],
                description: None,
                default: None,
                example: Some(json!(Self::example_data())),
                discriminator: None,
                ..Default::default()
            })
            .into()
        }
    }

    impl ToSchema for ModuleFeatureId {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}

impl Display for ModuleFeatureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{NAMESPACE_SEPARATOR}{}", self.module, self.feature)
    }
}

impl From<(ModuleId, FeatureId)> for ModuleFeatureId {
    fn from((module, feature): (ModuleId, FeatureId)) -> Self {
        Self { module, feature }
    }
}

impl ExampleData for ModuleFeatureId {
    fn example_data() -> Self {
        Self {
            module: ModuleId::example_data(),
            feature: FeatureId::example_data(),
        }
    }
}

/// The error that is returned by [ModuleFeatureId::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseModuleFeatureIdError {
    /// Invalid string pattern. The string must be either a [ModuleId] and a [FeatureId] separated by [NAMESPACE_SEPARATOR], or a [FeatureId].
    #[snafu(display(
        "Feature id must either be \"<feature>\" or \"<module>{separator}<feature>\""
    ))]
    InvalidPattern {
        /// The expected separator string
        separator: &'static str,
    },

    /// Module id could not be parsed.
    #[snafu(display("Could not parse module id {found:?}"))]
    ModuleIdParsing {
        /// The found module id string which couldn't be parsed
        found: String,

        /// The source error
        source: ParseModuleIdError,
    },

    /// Feature id could not be parsed.
    #[snafu(display("Could not parse feature id {found:?}"))]
    FeatureIdParsing {
        /// The found feature id string which couldn't be parsed
        found: String,

        /// The source error
        source: ParseFeatureIdError,
    },
}

impl FromStr for ModuleFeatureId {
    type Err = ParseModuleFeatureIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(NAMESPACE_SEPARATOR);

        match (parts.next(), parts.next()) {
            (Some(feature_id), None) => {
                let module = ModuleId::default();
                let feature = feature_id.parse().with_context(|_| FeatureIdParsingSnafu {
                    found: feature_id.to_string(),
                })?;

                Ok(Self { module, feature })
            }
            (Some(module_id), Some(feature_id)) => {
                ensure!(
                    parts.next().is_none(),
                    InvalidPatternSnafu {
                        separator: NAMESPACE_SEPARATOR
                    }
                );
                let module = module_id.parse().with_context(|_| ModuleIdParsingSnafu {
                    found: module_id.to_string(),
                })?;

                let feature = feature_id.parse().with_context(|_| FeatureIdParsingSnafu {
                    found: feature_id.to_string(),
                })?;

                Ok(Self { module, feature })
            }
            _ => InvalidPatternSnafu {
                separator: NAMESPACE_SEPARATOR,
            }
            .fail(),
        }
    }
}

#[cfg(feature = "diesel")]
mod diesel_traits {
    use std::{
        io::Write,
        str::{FromStr, from_utf8},
    };

    use diesel::{
        backend::Backend,
        deserialize::{self, FromSql},
        pg::Pg,
        serialize::{self, IsNull, Output, ToSql},
    };

    use super::*;

    impl ToSql<diesel::sql_types::Text, Pg> for ModuleFeatureId {
        fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
            write!(out, "{self}")?;
            Ok(IsNull::No)
        }
    }

    impl FromSql<diesel::sql_types::Text, Pg> for ModuleFeatureId {
        fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
            let s = from_utf8(bytes.as_bytes())?;
            let module_feature_id = Self::from_str(s)?;

            Ok(module_feature_id)
        }
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{ModuleFeatureId, NAMESPACE_SEPARATOR};

    impl Serialize for ModuleFeatureId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_string().serialize(serializer)
        }
    }

    struct ModuleFeatureIdVisitor;

    impl serde::de::Visitor<'_> for ModuleFeatureIdVisitor {
        type Value = ModuleFeatureId;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                formatter,
                "module id and namespace id separated by {NAMESPACE_SEPARATOR}"
            )
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let module_feature_id = v.parse().map_err(|_e| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)
            })?;
            Ok(module_feature_id)
        }
    }

    impl<'de> Deserialize<'de> for ModuleFeatureId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(ModuleFeatureIdVisitor)
        }
    }
}
