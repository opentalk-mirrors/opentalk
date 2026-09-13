// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use http0::{HeaderValue, header::InvalidHeaderValue};
use url::Url;

/// A trait to implement for types that can build a `Link` header for a HTTP response.
pub trait PagingLinkHeader {
    /// Build the link header, based on the request url.
    fn build_paging_link_header(&self, url: &Url) -> HeaderValue;
}

pub(super) fn vec_to_header_link_value(
    vec: Vec<(String, String)>,
) -> Result<HeaderValue, InvalidHeaderValue> {
    let buf = vec
        .iter()
        .map(|(rel, url)| format!("<{url}>; rel=\"{rel}\""))
        .collect::<Vec<_>>()
        .join(",");

    HeaderValue::from_str(&buf)
}
