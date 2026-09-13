// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::{Snafu, ensure};

use crate::utils::ExampleData;

const DEFAULT_VALUE: i64 = 1;
const FIRST_VALUE: i64 = 1;
const ONE_VALUE: i64 = 1;
const MIN_VALUE: i64 = 1;
const MAX_VALUE: i64 = i64::MAX;

/// Error when parsing a [`Page`].
#[derive(Debug, Snafu)]
pub enum TryFromPageError {
    /// Page number is outside of the allowed range.
    #[snafu(display("Page number is outside the allowed range ({min}..={max})"))]
    OutOfRange {
        /// The minimum page number
        min: i64,
        /// The maximum page number
        max: i64,
    },
}

/// A page number used for paging in requests.
///
/// This is `1`-based, so a `Page` with a value of `0` can not be created.
#[derive(
    Debug,
    Clone,
    Copy,
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(Page::example_data())))]
pub struct Page(i64);

impl Page {
    /// The minimum value of the type.
    pub const MIN: Self = Self(MIN_VALUE);

    /// The maximum value of the type.
    pub const MAX: Self = Self(MAX_VALUE);

    /// The default value of the type.
    pub const DEFAULT: Self = Self(DEFAULT_VALUE);

    /// The first page.
    pub const FIRST: Self = Self(FIRST_VALUE);

    /// The page with number 1.
    pub const ONE: Self = Self(ONE_VALUE);

    /// Create a new [`Page`] from an i64 value, clamped to the valid range.
    pub const fn from_i64_clamped(value: i64) -> Self {
        let value = if value < MIN_VALUE { MIN_VALUE } else { value };
        Self(value)
    }

    /// Add two [`Page`] values, saturating.
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self::from_i64_clamped(self.0.saturating_add(rhs.0))
    }

    /// Subtract a [`Page`] value from another, saturating.
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self::from_i64_clamped(self.0.saturating_sub(rhs.0))
    }

    /// Get the next page number. Will return itself if this is already the highest possible value.
    pub fn saturating_next(&self) -> Self {
        self.saturating_add(Self::ONE)
    }

    /// Get the previous page number. Will return itself if this is already the lowest possible value.
    pub fn saturating_previous(&self) -> Self {
        self.saturating_sub(Self::ONE)
    }

    /// Get the zero-based index value as i64.
    pub fn as_zero_based_i64(&self) -> i64 {
        self.0.saturating_sub(1).clamp(0, i64::MAX)
    }

    /// Get the zero-based index value as usize.
    pub fn as_zero_based_usize(&self) -> usize {
        self.0.saturating_sub(1) as usize
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl TryFrom<u64> for Page {
    type Error = TryFromPageError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromPageError::OutOfRange {
            min: MIN_VALUE,
            max: MAX_VALUE,
        })?;
        Ok(Self(value))
    }
}

impl TryFrom<i32> for Page {
    type Error = TryFromPageError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::try_from(i64::from(value))
    }
}

impl TryFrom<usize> for Page {
    type Error = TryFromPageError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromPageError::OutOfRange {
            min: MIN_VALUE,
            max: MAX_VALUE,
        })?;
        Ok(Self(value))
    }
}

impl TryFrom<i64> for Page {
    type Error = TryFromPageError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        ensure!(
            (MIN_VALUE..=MAX_VALUE).contains(&value),
            OutOfRangeSnafu {
                min: MIN_VALUE,
                max: MAX_VALUE
            }
        );
        Ok(Self(value))
    }
}

impl From<Page> for usize {
    fn from(Page(value): Page) -> Self {
        usize::try_from(value).unwrap()
    }
}

impl ExampleData for Page {
    fn example_data() -> Self {
        Self(5)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_matches};

    use super::{MAX_VALUE, MIN_VALUE, Page, TryFromPageError};

    #[test]
    fn saturating_add() {
        assert_eq!(
            Page::try_from(5423)
                .unwrap()
                .saturating_add(Page::try_from(3).unwrap()),
            Page::try_from(5426).unwrap()
        );
        assert_eq!(
            Page::MAX.saturating_add(Page::try_from(3).unwrap()),
            Page::MAX
        );
    }

    #[test]
    fn try_from() {
        assert_eq!(Page::try_from(1usize).unwrap(), Page::FIRST);
        assert_eq!(Page::try_from(1i32).unwrap(), Page::FIRST);
        assert_eq!(Page::try_from(1i64).unwrap(), Page::FIRST);
        assert_eq!(Page::try_from(1u64).unwrap(), Page::FIRST);

        assert_eq!(Page::try_from(14usize).unwrap(), Page(14));
        assert_eq!(Page::try_from(15i32).unwrap(), Page(15));
        assert_eq!(Page::try_from(16i64).unwrap(), Page(16));
        assert_eq!(Page::try_from(18u64).unwrap(), Page(18));

        assert_matches!(
            Page::try_from(-42i64),
            Err(TryFromPageError::OutOfRange {
                min: MIN_VALUE,
                max: MAX_VALUE
            })
        );
        assert_matches!(
            Page::try_from((i64::MAX as usize) + 1),
            Err(TryFromPageError::OutOfRange {
                min: MIN_VALUE,
                max: MAX_VALUE
            })
        );
        assert_matches!(
            Page::try_from((i64::MAX as u64) + 1),
            Err(TryFromPageError::OutOfRange {
                min: MIN_VALUE,
                max: MAX_VALUE
            })
        );
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::Page;

    #[test]
    fn serialize_default() {
        let example = Page::default();
        assert_eq!(json!(example), json!(1));
    }

    #[test]
    fn serialize() {
        let example = Page::try_from(423).unwrap();
        assert_eq!(json!(example), json!(423));
    }

    #[test]
    fn deserialize_invalid_zero() {
        assert!(serde_json::from_value::<Page>(json!(0)).is_err());
    }

    #[test]
    fn deserialize_default() {
        let example = Page::default();
        assert_eq!(example, serde_json::from_value(json!(1)).unwrap());
    }

    #[test]
    fn deserialize() {
        let example = Page::try_from(64).unwrap();
        assert_eq!(example, serde_json::from_value(json!(64)).unwrap());
    }
}
