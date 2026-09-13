// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Pagination Query types

use opentalk_types_common::pagination::{Page, PageSize};

/// Page-based pagination query
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams, utoipa::ToSchema))]
pub struct PagePaginationQuery {
    /// The number of entries per page
    #[cfg_attr(feature = "serde", serde(default))]
    pub per_page: PageSize,

    /// The number of the page
    #[cfg_attr(feature = "serde", serde(default,))]
    pub page: Page,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn pagination_query() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let paging = PagePaginationQuery {
            per_page: 12.try_into().unwrap(),
            page: 2i64.try_into().unwrap(),
        };

        let paging_urlencoded = "per_page=12&page=2";

        let serialize_output: String = serde_urlencoded::to_string(&paging)?;
        assert_eq!(paging_urlencoded, serialize_output);

        let deserialized = serde_urlencoded::from_str(paging_urlencoded)?;
        assert_eq!(paging, deserialized);

        Ok(())
    }

    #[test]
    fn pagination_query_out_of_bounds() {
        use serde::de::Error;

        assert_eq!(
            serde_urlencoded::from_str::<PagePaginationQuery>("per_page=12&page=-2"),
            Err(serde_urlencoded::de::Error::custom(format!(
                "Page number is outside the allowed range (1..={})",
                i64::MAX
            ))),
        );
        assert_eq!(
            serde_urlencoded::from_str::<PagePaginationQuery>("per_page=-12&page=2"),
            Err(serde_urlencoded::de::Error::custom(
                "Page size is outside the allowed range (1..=100)"
            )),
        );
        assert_eq!(
            serde_urlencoded::from_str::<PagePaginationQuery>("per_page=101&page=2"),
            Err(serde_urlencoded::de::Error::custom(
                "Page size is outside the allowed range (1..=100)"
            )),
        );
    }

    #[test]
    fn default_values() {
        let default_page = PagePaginationQuery {
            per_page: 12.try_into().unwrap(),
            page: Page::default(),
        };

        assert_eq!(
            Ok(default_page),
            serde_urlencoded::from_str::<PagePaginationQuery>("per_page=12")
        );

        let default_per_page = PagePaginationQuery {
            per_page: PageSize::default(),
            page: 13i64.try_into().unwrap(),
        };

        assert_eq!(
            Ok(default_per_page),
            serde_urlencoded::from_str::<PagePaginationQuery>("page=13")
        );

        let default_query = PagePaginationQuery {
            per_page: PageSize::default(),
            page: Page::default(),
        };
        assert_eq!(
            Ok(default_query),
            serde_urlencoded::from_str::<PagePaginationQuery>("")
        );
    }
}
