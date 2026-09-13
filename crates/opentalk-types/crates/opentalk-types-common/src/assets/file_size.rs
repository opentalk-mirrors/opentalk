// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

const DEFAULT_VALUE: i64 = 0;
const ZERO_VALUE: i64 = 0;
const MIN_VALUE: i64 = 0;
const MAX_VALUE: i64 = i64::MAX;

/// Error when parsing an [`FileSize`].
#[derive(Debug, Snafu)]
pub enum TryFromFileSizeError {
    /// File size is larger than the maximum file size allowed.
    #[snafu(display("File size is larger than the maximum file size allowed ({file_size_max})."))]
    ValueTooLarge {
        /// The maximum file size
        file_size_max: i64,
    },
    /// File size is negative.
    #[snafu(display("File size is negative"))]
    ValueNegative,
}

/// The size of a file.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    derive_more::Display,
    derive_more::AsRef,
    derive_more::Into,
)]
#[cfg_attr(
    feature = "diesel",
    derive(
        opentalk_diesel_newtype::DieselNewtype,
        diesel::expression::AsExpression,
        diesel::deserialize::FromSqlRow
    )
)]
#[cfg_attr(feature="diesel", diesel(sql_type = diesel::sql_types::BigInt))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "i64")
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(FileSize::example_data())))]
pub struct FileSize(i64);

impl FileSize {
    /// The minimum value of the type.
    pub const MIN: Self = Self(MIN_VALUE);

    /// The maximum value of the type.
    pub const MAX: Self = Self(MAX_VALUE);

    /// The default value of the type.
    pub const DEFAULT: Self = Self(DEFAULT_VALUE);

    /// The zero value of the type.
    pub const ZERO: Self = Self(ZERO_VALUE);

    /// Add two [`FileSize`] values, saturating.
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Subtract an [`FileSize`] from another, saturating.
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl TryFrom<u64> for FileSize {
    type Error = TryFromFileSizeError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromFileSizeError::ValueTooLarge {
            file_size_max: MAX_VALUE,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<usize> for FileSize {
    type Error = TryFromFileSizeError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromFileSizeError::ValueTooLarge {
            file_size_max: MAX_VALUE,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<i64> for FileSize {
    type Error = TryFromFileSizeError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        ensure!(value >= 0, ValueNegativeSnafu);
        Ok(Self(value))
    }
}

impl From<u32> for FileSize {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<FileSize> for usize {
    fn from(value: FileSize) -> Self {
        value.0 as usize
    }
}

impl ExampleData for FileSize {
    fn example_data() -> Self {
        Self(17)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_matches};

    use super::{FileSize, MAX_VALUE, TryFromFileSizeError};

    #[test]
    fn saturating_add() {
        assert_eq!(
            FileSize::from(5423).saturating_add(FileSize::from(3)),
            FileSize::from(5426)
        );
        assert_eq!(
            FileSize::MAX.saturating_add(FileSize::from(3)),
            FileSize::MAX
        );
    }

    #[test]
    fn try_from() {
        assert_eq!(FileSize::try_from(0usize).unwrap(), FileSize::ZERO);
        assert_eq!(FileSize::try_from(0i64).unwrap(), FileSize::ZERO);
        assert_eq!(FileSize::try_from(0u64).unwrap(), FileSize::ZERO);

        assert_eq!(FileSize::try_from(14usize).unwrap(), FileSize(14));
        assert_eq!(FileSize::try_from(16i64).unwrap(), FileSize(16));
        assert_eq!(FileSize::try_from(18u64).unwrap(), FileSize(18));

        assert_matches!(
            FileSize::try_from(-42i64),
            Err(TryFromFileSizeError::ValueNegative)
        );
        assert_matches!(
            FileSize::try_from((i64::MAX as usize) + 1),
            Err(TryFromFileSizeError::ValueTooLarge {
                file_size_max: MAX_VALUE
            })
        );
        assert_matches!(
            FileSize::try_from((i64::MAX as u64) + 1),
            Err(TryFromFileSizeError::ValueTooLarge {
                file_size_max: MAX_VALUE
            })
        );
    }

    #[test]
    fn from_u32() {
        assert_eq!(FileSize::from(0u32), FileSize::ZERO);
        assert_eq!(FileSize::from(42u32), FileSize(42));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::FileSize;

    #[test]
    fn serialize_default() {
        let example = FileSize::default();
        assert_eq!(json!(example), json!(0));
    }

    #[test]
    fn serialize() {
        let example = FileSize::from(423);
        assert_eq!(json!(example), json!(423));
    }

    #[test]
    fn deserialize_default() {
        let example = FileSize::default();
        assert_eq!(example, serde_json::from_value(json!(0)).unwrap());
    }

    #[test]
    fn deserialize() {
        let example = FileSize::from(64);
        assert_eq!(example, serde_json::from_value(json!(64)).unwrap());
    }
}
