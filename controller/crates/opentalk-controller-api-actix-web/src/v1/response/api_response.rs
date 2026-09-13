// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! API response type for paginated responses.
//!
//! The current Pagination support follows the GitHub REST APIv3, i.e. page
//! hints are included inside the Link HTTP header.

use actix_web::{
    HttpResponse, Responder,
    body::BoxBody,
    http::header::{self, HeaderMap},
    mime,
};
use either::Either;
use opentalk_types_api_v1::pagination::{CursorPagination, PagePagination, PagingLinkHeader as _};
use opentalk_types_common::pagination::{ItemCount, Page, PageSize};
use serde::Serialize;
use url::Url;

use crate::v1::response::ApiOutputLinkHeader;

/// An API response with pagination link headers.
#[derive(Debug, Clone)]
pub struct ApiResponse<T: Serialize> {
    links: ApiOutputLinkHeader,
    data: T,
}

impl<T: Serialize> ApiResponse<T> {
    /// Creates new [`ApiResponse`]
    pub fn new(data: T) -> Self {
        Self {
            links: ApiOutputLinkHeader { pagination: None },
            data,
        }
    }

    /// Transforms [`ApiResponse`] to also return page based pagination links.
    ///
    /// This is mutually exclusive to [ApiResponse::with_cursor_pagination]
    pub fn with_page_pagination(
        mut self,
        per_page: PageSize,
        page: Page,
        total: ItemCount,
    ) -> Self {
        self.links.pagination = Some(Either::Left(PagePagination::new(per_page, page, total)));

        self
    }

    /// Transforms [`ApiResponse`] to also return cursor based pagination links
    ///
    /// This is mutually exclusive to [ApiResponse::with_page_pagination]
    pub fn with_cursor_pagination(mut self, before: Option<String>, after: Option<String>) -> Self {
        if before.is_some() || after.is_some() {
            self.links.pagination = Some(Either::Right(CursorPagination::new(before, after)));
        }

        self
    }
}

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse {
        match serde_json::to_string(&self.data) {
            Ok(body) => {
                let url = extract_full_url_from_request(req);

                let mut headers = HeaderMap::new();
                if let Some(links) = match url {
                    Some(url) => self.links.pagination.map(|links| {
                        links.either(
                            |l| l.build_paging_link_header(&url),
                            |r| r.build_paging_link_header(&url),
                        )
                    }),
                    None => return HttpResponse::InternalServerError().finish(),
                } {
                    _ = headers.insert(header::LINK, links);
                }

                let mut response = HttpResponse::Ok();
                _ = response.content_type(mime::APPLICATION_JSON);

                for pair in headers {
                    _ = response.insert_header(pair);
                }

                response.body(body)
            }
            Err(err) => {
                HttpResponse::from_error(actix_web::error::JsonPayloadError::Serialize(err))
            }
        }
    }
}

fn extract_full_url_from_request(req: &actix_web::HttpRequest) -> Option<Url> {
    let conn = req.connection_info();

    let url = Url::parse(&format!(
        "{scheme}://{host}/",
        scheme = conn.scheme(),
        host = conn.host()
    ))
    .inspect_err(|e| tracing::warn!("Failed to extract full url for API response: {e}"))
    .ok()?;

    url.join(&req.uri().to_string())
        .inspect_err(|e| tracing::warn!("Failed to extract full url for API response: {e}"))
        .ok()
}
