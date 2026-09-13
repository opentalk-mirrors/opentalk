// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! The kind of an asset, and some extra functionality around it.

use std::str::FromStr;

use snafu::{Snafu, ensure};

use crate::identifier::Identifier;

/// The minimum length for a file kind
pub const ASSET_FILE_KIND_MIN_LENGTH: usize = 1;
/// The maximum length for a file kind
pub const ASSET_FILE_KIND_MAX_LENGTH: usize = 50;

/// The kind of asset
///
/// Can be parsed using [`std::str::FromStr`].
/// May contain alphanumeric ascii characters and underscores only, and is restricted to a
/// length of [`ASSET_FILE_KIND_MIN_LENGTH`] up to [`ASSET_FILE_KIND_MAX_LENGTH`] characters.
/// This serves as sanitization measure for user input.
#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde_with::DeserializeFromStr)
)]
pub struct AssetFileKind(Identifier);

impl AssetFileKind {
    /// Get the `&str` reference to the module id
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    // Only for use in the `module_id` macro
    #[doc(hidden)]
    pub const fn __new_borrowed(value: &'static str) -> Self {
        Self(Identifier::new_borrowed(value))
    }

    /// Get an example instance of the [`AssetFileKind`].
    pub const fn example_data() -> Self {
        Self::__new_borrowed("mykind")
    }
}

impl TryFrom<&'static str> for AssetFileKind {
    type Error = ParseAssetFileKindError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        ensure_is_valid(value)?;
        Ok(Self::__new_borrowed(value))
    }
}

impl TryFrom<String> for AssetFileKind {
    type Error = ParseAssetFileKindError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/// The error that is returned by [AssetFileKind::from_str] on failure.
#[derive(Debug, Snafu)]
pub enum ParseAssetFileKindError {
    /// The input string was shorter than the minimum length [ASSET_FILE_KIND_MIN_LENGTH].
    #[snafu(display("AssetFileKind must be at least {min_length} characters long"))]
    TooShort {
        /// The minimum allowed length.
        min_length: usize,
    },

    /// The input string was longer than the maximum length [ASSET_FILE_KIND_MAX_LENGTH].
    #[snafu(display("AssetFileKind must not be longer than {max_length} characters"))]
    TooLong {
        /// The maximum allowed length.
        max_length: usize,
    },

    /// Invalid characters were found in the input data.
    #[snafu(display("AssetFileKind only allows alphanumeric ascii characters or '_'"))]
    InvalidCharacters,
}

fn ensure_is_valid(s: &str) -> Result<(), ParseAssetFileKindError> {
    ensure!(
        s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
        InvalidCharactersSnafu
    );
    ensure!(
        s.len() >= ASSET_FILE_KIND_MIN_LENGTH,
        TooShortSnafu {
            min_length: ASSET_FILE_KIND_MIN_LENGTH
        }
    );
    ensure!(
        s.len() <= ASSET_FILE_KIND_MAX_LENGTH,
        TooLongSnafu {
            max_length: ASSET_FILE_KIND_MAX_LENGTH
        }
    );
    Ok(())
}

impl FromStr for AssetFileKind {
    type Err = ParseAssetFileKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ensure_is_valid(s)?;
        Ok(Self(Identifier::new_owned(s.to_string())))
    }
}

#[cfg(feature = "utoipa")]
mod impl_utoipa {
    use serde_json::json;
    use utoipa::{
        PartialSchema, ToSchema,
        openapi::{ObjectBuilder, RefOr, Schema, Type},
    };

    use super::{ASSET_FILE_KIND_MAX_LENGTH, ASSET_FILE_KIND_MIN_LENGTH, AssetFileKind};

    impl PartialSchema for AssetFileKind {
        fn schema() -> RefOr<Schema> {
            ObjectBuilder::new()
                .schema_type(Type::String)
                .min_length(Some(ASSET_FILE_KIND_MIN_LENGTH))
                .max_length(Some(ASSET_FILE_KIND_MAX_LENGTH))
                .pattern(Some("^[0-9a-zA-Z_]*$".to_string()))
                .description(Some("An asset file kind"))
                .examples([json!(AssetFileKind::example_data())])
                .into()
        }
    }

    impl ToSchema for AssetFileKind {
        fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
            schemas.push((Self::name().into(), Self::schema()));
        }
    }
}
