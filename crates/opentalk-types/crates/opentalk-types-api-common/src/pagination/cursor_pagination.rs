// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Page pagination links to be placed in a response header.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CursorPagination {
    /// The cursor identifying the previous page.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    before: Option<String>,

    /// The cursor identifying the next page.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    after: Option<String>,
}

#[cfg(feature = "actix")]
impl super::PagingLinkHeader for CursorPagination {
    fn build_paging_link_header(&self, url: &url::Url) -> http0::HeaderValue {
        super::paging_link_header::vec_to_header_link_value(self.build_links_vec(url))
            .expect("vec_to_header_value failed")
    }
}

impl CursorPagination {
    /// Create a new [`CursorPagination`] struct.
    pub fn new(before: Option<String>, after: Option<String>) -> Self {
        Self { before, after }
    }

    /// Get the cursor identifying the previous page.
    pub fn before(&self) -> Option<&str> {
        self.before.as_deref()
    }

    /// Get the cursor identifying the next page.
    pub fn after(&self) -> Option<&str> {
        self.after.as_deref()
    }

    #[cfg(feature = "actix")]
    /// Convert to a list of HTTP headers and their values.
    fn build_links_vec(&self, url: &url::Url) -> Vec<(String, String)> {
        use std::collections::HashMap;

        let mut headers = Vec::new();
        let mut query = url
            .query_pairs()
            .into_owned()
            .collect::<HashMap<String, String>>();
        _ = query.remove("page");
        _ = query.remove("before");
        _ = query.remove("after");

        let mut url = url.clone();
        let base = url
            .query_pairs_mut()
            .clear()
            .extend_pairs(query.iter())
            .finish();
        if let Some(before) = &self.before {
            let before = base
                .clone()
                .query_pairs_mut()
                .append_pair("before", before)
                .finish()
                .to_string();
            headers.push(("before".to_string(), before));
        }

        if let Some(after) = &self.after {
            let after = base
                .clone()
                .query_pairs_mut()
                .append_pair("after", after)
                .finish()
                .to_string();
            headers.push(("after".to_string(), after));
        }
        headers
    }
}
