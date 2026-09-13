// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::pagination::{ItemCount, Page, PageSize};

/// Page pagination links to be placed in a response header.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PagePagination {
    /// The number of the current page.
    pub page: Page,

    /// The page size.
    pub per_page: PageSize,

    /// The number of the first page.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub first: Option<Page>,

    /// The number of the previous page.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub prev: Option<Page>,

    /// The number of the next page.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub next: Option<Page>,

    /// The number of the last page.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub last: Option<Page>,
}

#[cfg(feature = "actix")]
impl super::PagingLinkHeader for PagePagination {
    fn build_paging_link_header(&self, url: &url::Url) -> http0::HeaderValue {
        super::paging_link_header::vec_to_header_link_value(self.build_links_vec(url))
            .expect("vec_to_header_value failed")
    }
}

impl PagePagination {
    /// Create a new [`PagePagination`] struct.
    pub fn new(per_page: PageSize, page: Page, total: ItemCount) -> Self {
        let first = (page != Page::FIRST).then_some(Page::FIRST);
        let prev = (page != Page::FIRST).then_some(page.saturating_previous());

        let last_page = per_page.last_page_for_item_count(total);

        let next = (page < last_page).then_some(page.saturating_next());
        let last = (page < last_page).then_some(last_page);

        Self {
            page,
            per_page,
            first,
            prev,
            next,
            last,
        }
    }

    #[cfg(feature = "actix")]
    /// Convert to a list of HTTP headers and their values.
    fn build_links_vec(&self, url: &url::Url) -> Vec<(String, String)> {
        let mut headers = Vec::new();
        let mut query = url
            .query_pairs()
            .into_owned()
            .collect::<std::collections::BTreeMap<String, String>>();
        _ = query.remove("page");
        let mut url = url.clone();
        let base = url
            .query_pairs_mut()
            .clear()
            .extend_pairs(query.iter())
            .finish();

        if let Some(first) = self.first {
            let first = base
                .clone()
                .query_pairs_mut()
                .append_pair("page", &first.to_string())
                .finish()
                .to_string();
            headers.push(("first".to_string(), first));
        }

        if let Some(prev) = self.prev {
            let prev = base
                .clone()
                .query_pairs_mut()
                .append_pair("page", &prev.to_string())
                .finish()
                .to_string();
            headers.push(("prev".to_string(), prev));
        }

        if let Some(next) = self.next {
            let next = base
                .clone()
                .query_pairs_mut()
                .append_pair("page", &next.to_string())
                .finish()
                .to_string();
            headers.push(("next".to_string(), next));
        }
        if let Some(last) = self.last {
            let last = base
                .clone()
                .query_pairs_mut()
                .append_pair("page", &last.to_string())
                .finish()
                .to_string();
            headers.push(("last".to_string(), last));
        }
        headers
    }
}
