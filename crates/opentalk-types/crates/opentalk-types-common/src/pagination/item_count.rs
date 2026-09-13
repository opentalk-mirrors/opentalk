// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::cmp::Ordering;

use snafu::{Snafu, ensure};

use crate::{pagination::PageSize, utils::ExampleData};

/// The maximum number for an item count to be valid.
pub const ITEM_COUNT_MAX: ItemCount = ItemCount(MAX_VALUE);

/// The default value for ItemCount.
pub const ITEM_COUNT_DEFAULT: ItemCount = ItemCount(DEFAULT_VALUE);

const DEFAULT_VALUE: i64 = 0;
const ZERO_VALUE: i64 = 0;
const MIN_VALUE: i64 = 0;
const MAX_VALUE: i64 = i64::MAX;

/// Error when parsing an [`ItemCount`].
#[derive(Debug, Snafu)]
pub enum TryFromItemCountError {
    /// Item count is larger than the maximum item count allowed.
    #[snafu(display(
        "Item count is larger than the maximum item count allowed ({item_count_max})."
    ))]
    ValueTooLarge {
        /// The maximum item count
        item_count_max: i64,
    },
    /// Item count is negative.
    #[snafu(display("Item count is negative"))]
    ValueNegative,
}

/// The number of items, e.g. .
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
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(ItemCount::example_data())))]
pub struct ItemCount(i64);

impl ItemCount {
    /// The minimum value of the type.
    pub const MIN: Self = Self(MIN_VALUE);

    /// The maximum value of the type.
    pub const MAX: Self = Self(MAX_VALUE);

    /// The default value of the type.
    pub const DEFAULT: Self = Self(DEFAULT_VALUE);

    /// The zero value of the type.
    pub const ZERO: Self = Self(ZERO_VALUE);

    /// Add two [`ItemCount`] values, saturating.
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Subtract an [`ItemCount`] from another, saturating.
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    /// Tell whether the value is the minimum value.
    pub const fn is_min(&self) -> bool {
        self.0 == MIN_VALUE
    }

    /// Tell whether the value is the maximum value.
    pub const fn is_max(&self) -> bool {
        self.0 == MAX_VALUE
    }

    /// Tell whether the value is zero.
    pub const fn is_zero(&self) -> bool {
        self.0 == ZERO_VALUE
    }

    /// Tell whether the value is the default value.
    pub const fn is_default(&self) -> bool {
        self.0 == DEFAULT_VALUE
    }
}

impl TryFrom<u64> for ItemCount {
    type Error = TryFromItemCountError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromItemCountError::ValueTooLarge {
            item_count_max: MAX_VALUE,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<usize> for ItemCount {
    type Error = TryFromItemCountError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let value = i64::try_from(value).map_err(|_e| TryFromItemCountError::ValueTooLarge {
            item_count_max: MAX_VALUE,
        })?;
        Self::try_from(value)
    }
}

impl TryFrom<i64> for ItemCount {
    type Error = TryFromItemCountError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        ensure!(value >= 0, ValueNegativeSnafu);
        Ok(Self(value))
    }
}

impl From<u32> for ItemCount {
    fn from(value: u32) -> Self {
        Self(value.into())
    }
}

impl From<ItemCount> for usize {
    fn from(value: ItemCount) -> Self {
        value.0 as usize
    }
}

impl From<PageSize> for ItemCount {
    fn from(value: PageSize) -> Self {
        Self(value.into())
    }
}

impl ExampleData for ItemCount {
    fn example_data() -> Self {
        Self(17)
    }
}

impl PartialEq<PageSize> for ItemCount {
    fn eq(&self, other: &PageSize) -> bool {
        self.0 == i64::from(*other)
    }
}

impl PartialOrd<PageSize> for ItemCount {
    fn partial_cmp(&self, other: &PageSize) -> Option<Ordering> {
        self.0.partial_cmp(&i64::from(*other))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_matches};

    use super::{ItemCount, MAX_VALUE, TryFromItemCountError};

    #[test]
    fn saturating_add() {
        assert_eq!(
            ItemCount::from(5423).saturating_add(ItemCount::from(3)),
            ItemCount::from(5426)
        );
        assert_eq!(
            ItemCount::MAX.saturating_add(ItemCount::from(3)),
            ItemCount::MAX
        );
    }

    #[test]
    fn try_from() {
        assert_eq!(ItemCount::try_from(0usize).unwrap(), ItemCount::ZERO);
        assert_eq!(ItemCount::try_from(0i64).unwrap(), ItemCount::ZERO);
        assert_eq!(ItemCount::try_from(0u64).unwrap(), ItemCount::ZERO);

        assert_eq!(ItemCount::try_from(14usize).unwrap(), ItemCount(14));
        assert_eq!(ItemCount::try_from(16i64).unwrap(), ItemCount(16));
        assert_eq!(ItemCount::try_from(18u64).unwrap(), ItemCount(18));

        assert_matches!(
            ItemCount::try_from(-42i64),
            Err(TryFromItemCountError::ValueNegative)
        );
        assert_matches!(
            ItemCount::try_from((i64::MAX as usize) + 1),
            Err(TryFromItemCountError::ValueTooLarge {
                item_count_max: MAX_VALUE
            })
        );
        assert_matches!(
            ItemCount::try_from((i64::MAX as u64) + 1),
            Err(TryFromItemCountError::ValueTooLarge {
                item_count_max: MAX_VALUE
            })
        );
    }

    #[test]
    fn from_u32() {
        assert_eq!(ItemCount::from(0u32), ItemCount::ZERO);
        assert_eq!(ItemCount::from(42u32), ItemCount(42));
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::ItemCount;

    #[test]
    fn serialize_default() {
        let example = ItemCount::default();
        assert_eq!(json!(example), json!(0));
    }

    #[test]
    fn serialize() {
        let example = ItemCount::from(423);
        assert_eq!(json!(example), json!(423));
    }

    #[test]
    fn deserialize_default() {
        let example = ItemCount::default();
        assert_eq!(
            example,
            serde_json::from_value::<ItemCount>(json!(0)).unwrap()
        );
    }

    #[test]
    fn deserialize() {
        let example = ItemCount::from(64);
        assert_eq!(
            example,
            serde_json::from_value::<ItemCount>(json!(64)).unwrap()
        );
    }
}
