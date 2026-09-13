// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// An API response that can contain paging information which will be added in a `Link` header in the response.
#[derive(Debug, Clone)]
pub struct PagedApiResponse<T, P> {
    inner: T,
    paging: P,
}

impl<T, P> PagedApiResponse<T, P> {
    /// Create a new paged API response from another API response and the paging information.
    pub fn new(inner: T, paging: P) -> Self {
        Self { inner, paging }
    }

    /// Get a reference to the inner response.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get a reference to the paging information.
    pub fn paging(&self) -> &P {
        &self.paging
    }
}

/// A trait for adding pagination to a response.
pub trait WithPagination: Sized {
    /// Build the paged API response based on the original API response and the paging information.
    fn with_pagination<P>(self, pagination: P) -> PagedApiResponse<Self, P>;
}

#[cfg(feature = "actix")]
mod actix_impls {
    use actix_web::{
        HttpResponse, Responder,
        body::{BoxBody, MessageBody},
    };
    use http0::header::LINK;
    use snafu::{ResultExt as _, Whatever};
    use url::Url;

    use crate::pagination::{
        PagedApiResponse, PagingLinkHeader, paged_api_response::WithPagination,
    };

    impl<T: Responder<Body = B>, B: MessageBody + 'static> WithPagination for T {
        fn with_pagination<P>(self, pagination: P) -> PagedApiResponse<Self, P> {
            PagedApiResponse::new(self, pagination)
        }
    }

    impl<T: Responder<Body = B>, P: PagingLinkHeader, B: MessageBody + 'static> Responder
        for PagedApiResponse<T, P>
    {
        type Body = BoxBody;

        fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse {
            let Ok(url) = extract_full_url_from_request(req)
                .inspect_err(|e| log::warn!("Error extracting full URL from HTTP request: {e}"))
            else {
                return HttpResponse::InternalServerError().finish();
            };

            let mut response = self.inner.respond_to(req).map_into_boxed_body();

            if !response.status().is_success() {
                return response;
            }

            _ = response
                .headers_mut()
                .insert(LINK, self.paging.build_paging_link_header(&url));
            response
        }
    }

    fn extract_full_url_from_request(req: &actix_web::HttpRequest) -> Result<Url, Whatever> {
        let conn = req.connection_info();

        let url = Url::parse(&format!(
            "{scheme}://{host}/",
            scheme = conn.scheme(),
            host = conn.host()
        ))
        .whatever_context("Failed to parse URL")?;

        url.join(&req.uri().to_string())
            .whatever_context("Failed to build URL")
    }
}
