// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Pagination types
//!
//! Great blogposts are:
//! - <https://phauer.com/2015/restful-api-design-best-practices/>
//! - <https://phauer.com/2018/web-api-pagination-timestamp-id-continuation-token/>

mod cursor;
mod cursor_pagination;
mod page_pagination;
mod page_pagination_query;
mod paged_api_response;
#[cfg(feature = "actix")]
mod paging_link_header;

pub use cursor::Cursor;
pub use cursor_pagination::CursorPagination;
pub use page_pagination::PagePagination;
pub use page_pagination_query::PagePaginationQuery;
pub use paged_api_response::{PagedApiResponse, WithPagination};
#[cfg(feature = "actix")]
pub use paging_link_header::PagingLinkHeader;
