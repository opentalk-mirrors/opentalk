// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

// SPDX-License-Identifier: EUPL-1.2

use std::cmp::Ordering;

use snafu::{Snafu, ensure};

use crate::{
    pagination::{ItemCount, ItemIndex, Page},
    utils::ExampleData,
};

const DEFAULT_VALUE: i64 = 30;
const ONE_VALUE: i64 = 1;
const MIN_VALUE: i64 = 1;
const MAX_VALUE: i64 = 100;

/// Error when parsing a [`Page`].
#[derive(Debug, Snafu)]
pub enum TryFromPageSizeError {
    /// Page size is outside of the allowed range.
    #[snafu(display("Page size is outside the allowed range ({min}..={max})"))]
    OutOfRange {
        /// The minimum page size
        min: i64,
        /// The minimum page size
        max: i64,
    },
}

/// A page number used for paging in requests.
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(PageSize::example_data())))]
pub struct PageSize(i64);

impl PageSize {
    /// The minimum value of the type.
    pub const MIN: Self = Self(MIN_VALUE);

    /// The maximum value of the type.
    pub const MAX: Self = Self(MAX_VALUE);

    /// The default value of the type.
    pub const DEFAULT: Self = Self(DEFAULT_VALUE);

    /// The page with number 1.
    pub const ONE: Self = Self(ONE_VALUE);

    /// Create a new [`PageSize`] from an i64 value, clamped to the valid range.
    pub const fn from_i64_clamped(value: i64) -> Self {
        let value = if value < MIN_VALUE { MIN_VALUE } else { value };
        let value = if value > MAX_VALUE { MAX_VALUE } else { value };
        Self(value)
    }

    /// Add two [`PageSize`] values, saturating.
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self::from_i64_clamped(self.0.saturating_add(rhs.0))
    }

    /// Subtract a [`PageSize`] value from another, saturating.
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

    /// Get the first index of a page.
    ///
    /// If the value would exceed the maximum allowed value, the first index of
    /// the last possible page is used, not the maximum value of [`ItemIndex`].
    pub fn first_index_on_page_saturating(&self, page: Page) -> ItemIndex {
        let index = self.0.saturating_mul(page.as_zero_based_i64());
        // In case the value saturated, we want to go back down to the start of the last page instead.
        let index = index.saturating_sub(index % self.0);
        // It should be impossible that `unwrap_or_default()` falls back to calling `default()`,
        // because based on the calculation above `index` is valid for `ItemIndex.
        ItemIndex::try_from(index).unwrap_or_default()
    }

    /// Get the last page calculated from the the overall item count.
    pub fn last_page_for_item_count(&self, item_count: ItemCount) -> Page {
        let quotient = usize::from(item_count) / usize::from(*self);
        let remainder = usize::from(item_count) % usize::from(*self);
        let last_page = if remainder > 0 && usize::from(*self) > 0 {
            quotient.saturating_add(1)
        } else {
            quotient
        };
        Page::try_from(last_page).unwrap()
    }
}

impl Default for PageSize {
    fn default() -> Self {
        Self(DEFAULT_VALUE)
    }
}

impl TryFrom<u64> for PageSize {
    type Error = TryFromPageSizeError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromPageSizeError::OutOfRange {
            min: MIN_VALUE,
            max: MAX_VALUE,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<usize> for PageSize {
    type Error = TryFromPageSizeError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromPageSizeError::OutOfRange {
            min: MIN_VALUE,
            max: MAX_VALUE,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<i32> for PageSize {
    type Error = TryFromPageSizeError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::try_from(i64::from(value))
    }
}

impl TryFrom<i64> for PageSize {
    type Error = TryFromPageSizeError;

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

impl TryFrom<ItemCount> for PageSize {
    type Error = TryFromPageSizeError;

    fn try_from(value: ItemCount) -> Result<Self, Self::Error> {
        Self::try_from(i64::from(value))
    }
}

impl From<PageSize> for usize {
    fn from(PageSize(value): PageSize) -> Self {
        usize::try_from(value).unwrap()
    }
}

impl PartialEq<ItemCount> for PageSize {
    fn eq(&self, other: &ItemCount) -> bool {
        self.0 == i64::from(*other)
    }
}

impl PartialOrd<ItemCount> for PageSize {
    fn partial_cmp(&self, other: &ItemCount) -> Option<Ordering> {
        self.0.partial_cmp(&i64::from(*other))
    }
}

impl ExampleData for PageSize {
    fn example_data() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_matches};

    use super::{MAX_VALUE, MIN_VALUE, PageSize, TryFromPageSizeError};

    #[test]
    fn saturating_add() {
        assert_eq!(
            PageSize::try_from(42)
                .unwrap()
                .saturating_add(PageSize::try_from(3).unwrap()),
            PageSize::try_from(45).unwrap()
        );
        assert_eq!(
            PageSize::MAX.saturating_add(PageSize::try_from(3).unwrap()),
            PageSize::MAX
        );
    }

    #[test]
    fn try_from() {
        assert_eq!(PageSize::try_from(1usize).unwrap(), PageSize::ONE);
        assert_eq!(PageSize::try_from(1i32).unwrap(), PageSize::ONE);
        assert_eq!(PageSize::try_from(1i64).unwrap(), PageSize::ONE);
        assert_eq!(PageSize::try_from(1u64).unwrap(), PageSize::ONE);

        assert_eq!(PageSize::try_from(14usize).unwrap(), PageSize(14));
        assert_eq!(PageSize::try_from(15i32).unwrap(), PageSize(15));
        assert_eq!(PageSize::try_from(16i64).unwrap(), PageSize(16));
        assert_eq!(PageSize::try_from(18u64).unwrap(), PageSize(18));

        assert_matches!(
            PageSize::try_from(-42i64),
            Err(TryFromPageSizeError::OutOfRange {
                min: MIN_VALUE,
                max: MAX_VALUE
            })
        );
        assert_matches!(
            PageSize::try_from((i64::MAX as usize) + 1),
            Err(TryFromPageSizeError::OutOfRange {
                min: MIN_VALUE,
                max: MAX_VALUE
            })
        );
        assert_matches!(
            PageSize::try_from((i64::MAX as u64) + 1),
            Err(TryFromPageSizeError::OutOfRange {
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

    use super::PageSize;

    #[test]
    fn serialize_default() {
        let example = PageSize::default();
        assert_eq!(json!(example), json!(30));
    }

    #[test]
    fn serialize() {
        let example = PageSize::try_from(23).unwrap();
        assert_eq!(json!(example), json!(23));
    }

    #[test]
    fn deserialize_invalid_zero() {
        assert!(serde_json::from_value::<PageSize>(json!(0)).is_err());
    }

    #[test]
    fn deserialize_default() {
        let example = PageSize::default();
        assert_eq!(
            example,
            serde_json::from_value::<PageSize>(json!(30)).unwrap()
        );
    }

    #[test]
    fn deserialize() {
        let example = PageSize::try_from(64).unwrap();
        assert_eq!(
            example,
            serde_json::from_value::<PageSize>(json!(64)).unwrap()
        );
    }
}
